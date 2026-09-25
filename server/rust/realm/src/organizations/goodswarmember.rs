//! Участники Goods War `CGoodsWarMember` из WorldServer, перенесённые в
//! Realm `organizations/`, подтверждённые `worldserver.exe` и
//! `worldserver.pdb`.
//!
//! Player->faction map и faction set сохраняют signed order; count-list —
//! отдельный порядок по убыванию count. Замена записи вставляет новую до первой
//! с меньшим либо равным count и только затем удаляет старое имя. Первая запись
//! имени публикует top-five, замена существующей — нет.
//!
//! Wire `0x7FF20` сохраняет operations и terminal `(0,0)`; `0x7FF21` содержит
//! не более пяти fixed-name records. `FactionWin` вставляет только неизвестных
//! players, отрицательный master ID остаётся marker-ом.
//!
//! DB reload не очищает прежний count-list и добавляет TOP 5 в DB order;
//! поздняя ошибка сохраняет префикс. Имена длиннее 19 байт отклоняются вместо
//! переполнения `char[20]`; хвост fixed поля обнулён.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::OpenOptions;
use std::io::Write;

use chrono::{Datelike, Local, Timelike};
use encoding_rs::WINDOWS_1251;
use futures_util::TryStreamExt;
use tiberius::Row;

use crate::app::world_message::CMessage;
use crate::persistence::rssetup::WorldTdsClient;

const GOODS_WAR_STATE_MESSAGE_TYPE: i32 = 0x7FF20;
const GOODS_WAR_COUNT_MESSAGE_TYPE: i32 = 0x7FF21;
const FACTION_NAME_CAPACITY: usize = 20;
const MAX_PUBLISHED_COUNTS: usize = 5;
const LOAD_GOODS_WAR_COUNTS_SQL: &str = concat!(
    "SELECT TOP 5 Name,GoodsWarCount FROM CSL_FACTION_BaseProperty ",
    "WHERE GoodsWarCount > 0 ",
    "ORDER BY GoodsWarCount DESC, GoodsWarLastTime DESC",
);
const GOODS_WAR_DATABASE_ERROR_TEXT: &[u8] = b"ERR:  GoodsWarCount ....failed!";

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoodsWarFactionSnapshot {
    pub name: Vec<u8>,
    pub goods_war_count: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoodsWarFactionWinSnapshot {
    pub faction_id: i32,
    pub name: Vec<u8>,
    pub master_id: i32,
    pub member_ids: Vec<i32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoodsWarAuditPlayer {
    pub account: Vec<u8>,
    pub player_id: i32,
    pub name: Vec<u8>,
    pub level: u8,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GoodsWarAuditEnvironment {
    pub login_server_id: i32,
    pub world_number: u32,
}

pub trait GoodsWarDeliveryContext {
    fn send_all(&mut self, message: &CMessage) -> i32;
}

pub trait GoodsWarMemberContext: GoodsWarDeliveryContext {
    type Block;

    fn faction_snapshot(
        &mut self,
        faction_id: i32,
    ) -> Result<Option<GoodsWarFactionSnapshot>, Self::Block>;

 /// Выполняет concrete `CFaction::SetGoodsWarCount` и возвращает
 /// нормализованное сохранённое значение.
    fn set_faction_goods_war_count(
        &mut self,
        faction_id: i32,
        count: i32,
    ) -> Result<i32, Self::Block>;

 /// Возвращает runtime-поля для legacy `bzhsmd.txt`; `None`
 /// означает, что setup ещё не достиг назначенного `dwNumber`.
    fn faction_win_audit_environment(&mut self) -> Option<GoodsWarAuditEnvironment>;

    fn faction_win_audit_player(&mut self, player_id: i32) -> Option<GoodsWarAuditPlayer>;
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct GoodsWarFactionCount {
    name: [u8; FACTION_NAME_CAPACITY],
    count: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GoodsWarMutationReport {
    pub target_found: bool,
    pub state_changed: bool,
    pub delivery: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoodsWarRefreshReport {
    pub members_delivery: i32,
    pub counts_delivery: i32,
    pub faction_ids_delivery: i32,
}

#[derive(Debug)]
pub enum GoodsWarDatabaseLoadFailure {
    MissingConnection,
    Database {
        row_index: Option<usize>,
        source: tiberius::error::Error,
    },
    MissingRequiredValue {
        row_index: usize,
        column: &'static str,
    },
    NumericOutsideRange {
        row_index: usize,
        column: &'static str,
        value: i64,
    },
    FactionNameWouldOverflow {
        row_index: usize,
        visible_len: usize,
        capacity: usize,
    },
}

#[derive(Debug)]
pub enum GoodsWarDatabaseLoadDisposition {
    Complete,
    Failed(GoodsWarDatabaseLoadFailure),
}

#[derive(Debug)]
pub struct GoodsWarDatabaseLoadReport {
    pub previous_count: usize,
    pub appended_count: usize,
    pub total_count: usize,
    pub disposition: GoodsWarDatabaseLoadDisposition,
    pub error_text: Option<&'static [u8]>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GoodsWarFactionWinAuditReport {
    pub environment_available: bool,
    pub file_opened: bool,
    pub header_written: bool,
    pub player_lookups_attempted: usize,
    pub player_rows_attempted: usize,
    pub player_rows_written: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GoodsWarFactionWinReport {
    pub inserted_members: usize,
    pub delivery: i32,
    pub audit: GoodsWarFactionWinAuditReport,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GoodsWarMemberBlock<ContextBlock> {
    Context(ContextBlock),
    FactionNameWouldOverflow {
        faction_id: i32,
        visible_len: usize,
        capacity: usize,
    },
}

#[derive(Default)]
pub struct CGoodsWarMember {
    lifecycle_live: bool,
    members: BTreeMap<i32, i32>,
    faction_ids: BTreeSet<i32>,
    counts: Vec<GoodsWarFactionCount>,
}

impl CGoodsWarMember {
    pub const fn with_reached_empty_state() -> Self {
        Self {
            lifecycle_live: false,
            members: BTreeMap::new(),
            faction_ids: BTreeSet::new(),
            counts: Vec::new(),
        }
    }

    pub const fn begin_lifecycle(&mut self) {
        self.lifecycle_live = true;
    }

    pub fn release_lifecycle(&mut self) -> bool {
        let was_live = self.lifecycle_live;
        *self = Self::with_reached_empty_state();
        was_live
    }

 /// Потоково дописывает `TOP 5` count-list из World DB.
 ///
 /// Ошибка не откатывает уже прочитанный prefix: исходный COM catch также
 /// завершал `reInitDB`, оставляя ранее присоединённые list-node-ы.
    pub async fn reinitialize_database(
        &mut self,
        active_connection: Option<&mut WorldTdsClient>,
    ) -> GoodsWarDatabaseLoadReport {
        let previous_count = self.counts.len();
        macro_rules! failed {
            ($failure:expr) => {
                return GoodsWarDatabaseLoadReport {
                    previous_count,
                    appended_count: self.counts.len() - previous_count,
                    total_count: self.counts.len(),
                    disposition: GoodsWarDatabaseLoadDisposition::Failed($failure),
                    error_text: Some(GOODS_WAR_DATABASE_ERROR_TEXT),
                }
            };
        }

        let Some(active_connection) = active_connection else {
            failed!(GoodsWarDatabaseLoadFailure::MissingConnection);
        };
        let mut rows = match active_connection
            .simple_query(LOAD_GOODS_WAR_COUNTS_SQL)
            .await
        {
            Ok(rows) => rows,
            Err(source) => failed!(GoodsWarDatabaseLoadFailure::Database {
                row_index: None,
                source,
            }),
        };
        let mut row_index = 0usize;
        loop {
            let item = match rows.try_next().await {
                Ok(Some(item)) => item,
                Ok(None) => break,
                Err(source) => failed!(GoodsWarDatabaseLoadFailure::Database {
                    row_index: Some(row_index),
                    source,
                }),
            };
            let Some(row) = item.into_row() else {
                continue;
            };

            let name = match row.try_get::<&str, _>("Name") {
                Ok(Some(name)) => name,
                Ok(None) => failed!(GoodsWarDatabaseLoadFailure::MissingRequiredValue {
                    row_index,
                    column: "Name",
                }),
                Err(source) => failed!(GoodsWarDatabaseLoadFailure::Database {
                    row_index: Some(row_index),
                    source,
                }),
            };
            let (encoded_name, _, _) = WINDOWS_1251.encode(name);
            let visible_name = legacy_c_string_prefix(encoded_name.as_ref());
            if visible_name.len() >= FACTION_NAME_CAPACITY {
                failed!(GoodsWarDatabaseLoadFailure::FactionNameWouldOverflow {
                    row_index,
                    visible_len: visible_name.len(),
                    capacity: FACTION_NAME_CAPACITY,
                });
            }
            let count = match read_ado_long(&row, "GoodsWarCount") {
                Ok(Some(count)) => count,
                Ok(None) => failed!(GoodsWarDatabaseLoadFailure::MissingRequiredValue {
                    row_index,
                    column: "GoodsWarCount",
                }),
                Err(ReadGoodsWarLongError::Database(source)) => {
                    failed!(GoodsWarDatabaseLoadFailure::Database {
                        row_index: Some(row_index),
                        source,
                    })
                }
                Err(ReadGoodsWarLongError::OutsideRange(value)) => {
                    failed!(GoodsWarDatabaseLoadFailure::NumericOutsideRange {
                        row_index,
                        column: "GoodsWarCount",
                        value,
                    })
                }
            };

            let mut stored_name = [0_u8; FACTION_NAME_CAPACITY];
            stored_name[..visible_name.len()].copy_from_slice(visible_name);
            self.counts.push(GoodsWarFactionCount {
                name: stored_name,
                count,
            });
            row_index += 1;
        }

        GoodsWarDatabaseLoadReport {
            previous_count,
            appended_count: self.counts.len() - previous_count,
            total_count: self.counts.len(),
            disposition: GoodsWarDatabaseLoadDisposition::Complete,
            error_text: None,
        }
    }

    pub fn contains_faction_id(&self, faction_id: i32) -> bool {
        self.faction_ids.contains(&faction_id)
    }

    fn send_members<Context: GoodsWarDeliveryContext + ?Sized>(
        &self,
        context: &mut Context,
    ) -> i32 {
        let mut message = CMessage::new(GOODS_WAR_STATE_MESSAGE_TYPE);
        if self.members.is_empty() {
            message.base_mut().add_long(5);
        } else {
            message.base_mut().add_long(4);
            for (&player_id, &faction_id) in &self.members {
                message.base_mut().add_long(player_id);
                message.base_mut().add_long(faction_id);
            }
            message.base_mut().add_long(0);
            message.base_mut().add_long(0);
        }
        context.send_all(&message)
    }

    fn send_count_list<Context: GoodsWarDeliveryContext + ?Sized>(
        &self,
        context: &mut Context,
    ) -> i32 {
        let published_count = self.counts.len().min(MAX_PUBLISHED_COUNTS);
        let mut message = CMessage::new(GOODS_WAR_COUNT_MESSAGE_TYPE);
        message.base_mut().add_long(published_count as i32);
        for record in self.counts.iter().take(published_count) {
            message.base_mut().add(&record.name);
            message.base_mut().add_long(record.count);
        }
        context.send_all(&message)
    }

    fn send_faction_ids<Context: GoodsWarDeliveryContext + ?Sized>(
        &self,
        context: &mut Context,
    ) -> i32 {
        let mut message = CMessage::new(GOODS_WAR_STATE_MESSAGE_TYPE);
        message.base_mut().add_long(0x10);
        message.base_mut().add_long(self.faction_ids.len() as i32);
        for &faction_id in &self.faction_ids {
            message.base_mut().add_long(faction_id);
        }
        context.send_all(&message)
    }

    pub fn delete_one_member<Context: GoodsWarDeliveryContext + ?Sized>(
        &mut self,
        player_id: i32,
        context: &mut Context,
    ) -> GoodsWarMutationReport {
        if self.members.remove(&player_id).is_none() {
            return GoodsWarMutationReport::default();
        }
        let mut message = CMessage::new(GOODS_WAR_STATE_MESSAGE_TYPE);
        message.base_mut().add_long(2);
        message.base_mut().add_long(player_id);
        GoodsWarMutationReport {
            target_found: true,
            state_changed: true,
            delivery: Some(context.send_all(&message)),
        }
    }

    pub fn delete_members_by_faction_id<Context: GoodsWarDeliveryContext + ?Sized>(
        &mut self,
        faction_id: i32,
        context: &mut Context,
    ) -> GoodsWarMutationReport {
        let previous_len = self.members.len();
        self.members
            .retain(|_, member_faction_id| *member_faction_id != faction_id);
        if self.members.len() == previous_len {
            return GoodsWarMutationReport::default();
        }

        let mut message = CMessage::new(GOODS_WAR_STATE_MESSAGE_TYPE);
        message.base_mut().add_long(3);
        message.base_mut().add_long(faction_id);
        GoodsWarMutationReport {
            target_found: true,
            state_changed: true,
            delivery: Some(context.send_all(&message)),
        }
    }

    pub fn delete_one_faction_count<Context>(
        &mut self,
        faction_id: i32,
        context: &mut Context,
    ) -> Result<GoodsWarMutationReport, GoodsWarMemberBlock<Context::Block>>
    where
        Context: GoodsWarMemberContext + ?Sized,
    {
        let Some(snapshot) = context
            .faction_snapshot(faction_id)
            .map_err(GoodsWarMemberBlock::Context)?
        else {
            return Ok(GoodsWarMutationReport::default());
        };
        Ok(self.delete_one_faction_count_by_name(&snapshot.name, context))
    }

    pub fn delete_one_faction_count_by_name<Context>(
        &mut self,
        faction_name: &[u8],
        context: &mut Context,
    ) -> GoodsWarMutationReport
    where
        Context: GoodsWarDeliveryContext + ?Sized,
    {
        let faction_name = legacy_c_string_prefix(faction_name);
        let Some(position) = self
            .counts
            .iter()
            .position(|entry| legacy_c_string_prefix(&entry.name) == faction_name)
        else {
            return GoodsWarMutationReport {
                target_found: true,
                ..GoodsWarMutationReport::default()
            };
        };
        self.counts.remove(position);
        GoodsWarMutationReport {
            target_found: true,
            state_changed: true,
            delivery: Some(self.send_count_list(context)),
        }
    }

    pub fn insert_one_faction<Context>(
        &mut self,
        faction_id: i32,
        context: &mut Context,
    ) -> Result<GoodsWarMutationReport, GoodsWarMemberBlock<Context::Block>>
    where
        Context: GoodsWarMemberContext + ?Sized,
    {
        if context
            .faction_snapshot(faction_id)
            .map_err(GoodsWarMemberBlock::Context)?
            .is_none()
        {
            return Ok(GoodsWarMutationReport::default());
        }
        if !self.faction_ids.insert(faction_id) {
            return Ok(GoodsWarMutationReport {
                target_found: true,
                ..GoodsWarMutationReport::default()
            });
        }
        Ok(GoodsWarMutationReport {
            target_found: true,
            state_changed: true,
            delivery: Some(self.send_faction_ids(context)),
        })
    }

 /// Увеличивает count и переставляет запись; публикация есть только при
 /// первом появлении имени, как в раннем return старого owner-а.
    pub fn append_one_faction_to_count<Context>(
        &mut self,
        faction_id: i32,
        context: &mut Context,
    ) -> Result<GoodsWarMutationReport, GoodsWarMemberBlock<Context::Block>>
    where
        Context: GoodsWarMemberContext + ?Sized,
    {
        let Some(snapshot) = context
            .faction_snapshot(faction_id)
            .map_err(GoodsWarMemberBlock::Context)?
        else {
            return Ok(GoodsWarMutationReport::default());
        };
        let visible_name = legacy_c_string_prefix(&snapshot.name);
        if visible_name.len() >= FACTION_NAME_CAPACITY {
            return Err(GoodsWarMemberBlock::FactionNameWouldOverflow {
                faction_id,
                visible_len: visible_name.len(),
                capacity: FACTION_NAME_CAPACITY,
            });
        }
        let mut name = [0_u8; FACTION_NAME_CAPACITY];
        name[..visible_name.len()].copy_from_slice(visible_name);

        let old_position = self
            .counts
            .iter()
            .position(|entry| legacy_c_string_prefix(&entry.name) == visible_name);
        let next_count = snapshot.goods_war_count.wrapping_add(1);
        let saved_count = context
            .set_faction_goods_war_count(faction_id, next_count)
            .map_err(GoodsWarMemberBlock::Context)?;
        let insert_position = self
            .counts
            .iter()
            .position(|entry| entry.count <= saved_count)
            .unwrap_or(self.counts.len());
        self.counts.insert(
            insert_position,
            GoodsWarFactionCount {
                name,
                count: saved_count,
            },
        );
        if let Some(old_position) = old_position {
            let shifted_old_position = if old_position >= insert_position {
                old_position + 1
            } else {
                old_position
            };
            self.counts.remove(shifted_old_position);
        }
        Ok(GoodsWarMutationReport {
            target_found: true,
            state_changed: true,
            delivery: old_position
                .is_none()
                .then(|| self.send_count_list(context)),
        })
    }

    pub fn refresh_all<Context: GoodsWarDeliveryContext + ?Sized>(
        &self,
        context: &mut Context,
    ) -> GoodsWarRefreshReport {
        GoodsWarRefreshReport {
            members_delivery: self.send_members(context),
            counts_delivery: self.send_count_list(context),
            faction_ids_delivery: self.send_faction_ids(context),
        }
    }

 /// Выполняет `FactionWin`: insert-only member delta, wire sentinel,
 /// общий send и только затем best-effort legacy file audit.
    pub fn faction_win<Context: GoodsWarMemberContext + ?Sized>(
        &mut self,
        winner: &GoodsWarFactionWinSnapshot,
        context: &mut Context,
    ) -> GoodsWarFactionWinReport {
        let mut message = CMessage::new(GOODS_WAR_STATE_MESSAGE_TYPE);
        message.base_mut().add_long(1);
        message.base_mut().add_long(winner.faction_id);

        let mut inserted_members = 0usize;
        for &player_id in &winner.member_ids {
            if self.members.contains_key(&player_id) {
                continue;
            }
            self.members.insert(player_id, winner.faction_id);
            message.base_mut().add_long(player_id);
            inserted_members += 1;
        }
        message.base_mut().add_long(winner.master_id.wrapping_neg());
        message.base_mut().add_long(0);

        let delivery = context.send_all(&message);
        let audit = context
            .faction_win_audit_environment()
            .map_or_else(GoodsWarFactionWinAuditReport::default, |environment| {
                append_faction_win_audit(winner, &environment, context)
            });
        GoodsWarFactionWinReport {
            inserted_members,
            delivery,
            audit,
        }
    }
}

enum ReadGoodsWarLongError {
    Database(tiberius::error::Error),
    OutsideRange(i64),
}

fn read_ado_long(
    row: &Row,
    column: &'static str,
) -> Result<Option<i32>, ReadGoodsWarLongError> {
    let first_error = match row.try_get::<i32, _>(column) {
        Ok(value) => return Ok(value),
        Err(error) => error,
    };
    if let Ok(value) = row.try_get::<u8, _>(column) {
        return Ok(value.map(i32::from));
    }
    if let Ok(value) = row.try_get::<i16, _>(column) {
        return Ok(value.map(i32::from));
    }
    if let Ok(value) = row.try_get::<i64, _>(column) {
        return value
            .map(|value| {
                i32::try_from(value).map_err(|_| ReadGoodsWarLongError::OutsideRange(value))
            })
            .transpose();
    }
    Err(ReadGoodsWarLongError::Database(first_error))
}

fn legacy_c_string_prefix(value: &[u8]) -> &[u8] {
    &value[..value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len())]
}

fn append_faction_win_audit<Context: GoodsWarMemberContext + ?Sized>(
    winner: &GoodsWarFactionWinSnapshot,
    environment: &GoodsWarAuditEnvironment,
    context: &mut Context,
) -> GoodsWarFactionWinAuditReport {
    let mut report = GoodsWarFactionWinAuditReport {
        environment_available: true,
        ..GoodsWarFactionWinAuditReport::default()
    };
    let now = Local::now();
    let Ok(mut file) = OpenOptions::new()
        .read(true)
        .append(true)
        .create(true)
        .open("bzhsmd.txt")
    else {
        return report;
    };
    report.file_opened = true;

    let mut header = Vec::new();
    let _ = write!(
        header,
        "\r\n{}-{} {:04}{:02}{:02} {:02}:{:02}\r\n>>>FName:",
        environment.login_server_id,
        environment.world_number as i32,
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
    );
    header.extend_from_slice(legacy_c_string_prefix(&winner.name));
    let _ = write!(
        header,
        "\tMasterID:{}\tCount:{}\r\n",
        winner.master_id,
        winner.member_ids.len() as u32 as i32,
    );
    report.header_written = file.write_all(&header).is_ok();

    for &player_id in &winner.member_ids {
        report.player_lookups_attempted += 1;
        let Some(player) = context.faction_win_audit_player(player_id) else {
            continue;
        };
        report.player_rows_attempted += 1;
        let mut row = Vec::new();
        row.extend_from_slice(legacy_c_string_prefix(&player.account));
        let _ = write!(row, "\t{}\t", player.player_id);
        row.extend_from_slice(legacy_c_string_prefix(&player.name));
        let _ = write!(row, "\t{}\r\n", player.level);
        if file.write_all(&row).is_ok() {
            report.player_rows_written += 1;
        }
    }
    report
}
