//! Аукционный журнал Realm: контракт записей `CAuctionLog::stLogNode`.
//!
//! Источник — точная пара `Nworldserver.exe`/`WorldServer.pdb`
//! (RSDS 289F1FB3-96A0-4FF4-8B5D-1FD17B50B751), PDB-путь владельца
//! `e:\svn\...\public\auctionroom\auctionlog.cpp`. `#[repr(C)]` layout 0x150
//! байт сохранён.

use chrono::NaiveDate;

use nebokrai_shared::values::CGuid;

pub const AUCTION_LOG_NODE_SIZE: usize = 0x150;
pub const AUCTION_LOG_DESCRIPTION_SIZE: usize = 0x100;

/// Оригинал Windows `SYSTEMTIME` аукционного журнала.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct AuctionLogSystemTime {
    pub year: u16,
    pub month: u16,
    pub day_of_week: u16,
    pub day: u16,
    pub hour: u16,
    pub minute: u16,
    pub second: u16,
    pub milliseconds: u16,
}

impl AuctionLogSystemTime {
    pub fn to_legacy_bytes(self) -> [u8; 0x10] {
        let mut bytes = [0; 0x10];
        for (index, value) in [
            self.year,
            self.month,
            self.day_of_week,
            self.day,
            self.hour,
            self.minute,
            self.second,
            self.milliseconds,
        ]
        .into_iter()
        .enumerate()
        {
            let offset = index * 2;
            bytes[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    pub fn is_after_zero_baseline(self) -> Result<bool, AuctionLogTimeBlock> {
        if self.year == 0
            && self.month == 0
            && self.day == 0
            && self.hour == 0
            && self.minute == 0
            && self.second == 0
        {
            return Ok(false);
        }
        let Some(date) = NaiveDate::from_ymd_opt(
            i32::from(self.year),
            u32::from(self.month),
            u32::from(self.day),
        ) else {
            return Err(AuctionLogTimeBlock { time: self });
        };
        let Some(time) = date.and_hms_opt(
            u32::from(self.hour),
            u32::from(self.minute),
            u32::from(self.second),
        ) else {
            return Err(AuctionLogTimeBlock { time: self });
        };

        Ok(time.and_utc().timestamp() > -1)
    }
}

/// Невоспроизводимая безопасно `_mktime`-нормализация повреждённой DB-даты.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuctionLogTimeBlock {
    pub time: AuctionLogSystemTime,
}

/// Полный PDB-layout `CAuctionLog::stLogNode`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct AuctionLogNode {
    pub base_id: i32,
    pub operation_type: i32,
    pub money_type: i32,
    pub money_num: i32,
    pub player_id: i32,
    pub amount: i32,
    pub fee: i32,
    pub notice: i32,
    pub time: AuctionLogSystemTime,
    pub description: [u8; AUCTION_LOG_DESCRIPTION_SIZE],
    pub guid: CGuid,
    pub guid_key: CGuid,
}

impl Default for AuctionLogNode {
    fn default() -> Self {
        Self {
            base_id: 0,
            operation_type: 0,
            money_type: 0,
            money_num: 0,
            player_id: 0,
            amount: 0,
            fee: 0,
            notice: 0,
            time: Default::default(),
            description: [0; AUCTION_LOG_DESCRIPTION_SIZE],
            guid: CGuid::GUID_INVALID,
            guid_key: CGuid::GUID_INVALID,
        }
    }
}

impl AuctionLogNode {
    /// Декодирует точный raw `stLogNode[0x150]`, который World handler копировал
    /// через `_GetBufferFromByteArray` без field-wise wire преобразований.
    pub fn from_legacy_bytes(bytes: &[u8; AUCTION_LOG_NODE_SIZE]) -> Self {
        let read_i32 = |offset: usize| {
            i32::from_le_bytes(
                bytes[offset..offset + 4]
                    .try_into()
                    .expect("поле stLogNode помещается в fixed layout"),
            )
        };
        let read_u16 = |offset: usize| {
            u16::from_le_bytes(
                bytes[offset..offset + 2]
                    .try_into()
                    .expect("SYSTEMTIME поле помещается в fixed layout"),
            )
        };
        let mut description = [0; AUCTION_LOG_DESCRIPTION_SIZE];
        description.copy_from_slice(&bytes[0x30..0x130]);
        let guid = CGuid::from_legacy_bytes(
            bytes[0x130..0x140]
                .try_into()
                .expect("auction GUID имеет 16 байт"),
        );
        let guid_key = CGuid::from_legacy_bytes(
            bytes[0x140..0x150]
                .try_into()
                .expect("auction GUID key имеет 16 байт"),
        );
        Self {
            base_id: read_i32(0x00),
            operation_type: read_i32(0x04),
            money_type: read_i32(0x08),
            money_num: read_i32(0x0c),
            player_id: read_i32(0x10),
            amount: read_i32(0x14),
            fee: read_i32(0x18),
            notice: read_i32(0x1c),
            time: AuctionLogSystemTime {
                year: read_u16(0x20),
                month: read_u16(0x22),
                day_of_week: read_u16(0x24),
                day: read_u16(0x26),
                hour: read_u16(0x28),
                minute: read_u16(0x2a),
                second: read_u16(0x2c),
                milliseconds: read_u16(0x2e),
            },
            description,
            guid,
            guid_key,
        }
    }

    pub fn description(&self) -> Option<&[u8]> {
        self.description
            .iter()
            .position(|byte| *byte == 0)
            .map(|terminator| &self.description[..terminator])
    }

    pub fn to_legacy_bytes(&self) -> [u8; AUCTION_LOG_NODE_SIZE] {
        let mut bytes = [0; AUCTION_LOG_NODE_SIZE];
        for (offset, value) in [
            (0x00, self.base_id),
            (0x04, self.operation_type),
            (0x08, self.money_type),
            (0x0c, self.money_num),
            (0x10, self.player_id),
            (0x14, self.amount),
            (0x18, self.fee),
            (0x1c, self.notice),
        ] {
            bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes[0x20..0x30].copy_from_slice(&self.time.to_legacy_bytes());
        bytes[0x30..0x130].copy_from_slice(&self.description);
        bytes[0x130..0x140].copy_from_slice(self.guid.as_legacy_bytes());
        bytes[0x140..0x150].copy_from_slice(self.guid_key.as_legacy_bytes());
        bytes
    }

    pub fn description_wire_bytes(&self) -> Option<&[u8]> {
        let terminator = self.description.iter().position(|byte| *byte == 0)?;
        Some(&self.description[..=terminator])
    }
}

const _: () = {
    assert!(std::mem::size_of::<AuctionLogSystemTime>() == 0x10);
    assert!(std::mem::size_of::<AuctionLogNode>() == AUCTION_LOG_NODE_SIZE);
    assert!(std::mem::offset_of!(AuctionLogNode, player_id) == 0x10);
    assert!(std::mem::offset_of!(AuctionLogNode, time) == 0x20);
    assert!(std::mem::offset_of!(AuctionLogNode, description) == 0x30);
    assert!(std::mem::offset_of!(AuctionLogNode, guid) == 0x130);
    assert!(std::mem::offset_of!(AuctionLogNode, guid_key) == 0x140);
};

// Runtime-семья `CAuctionLog` (world-владелец `public/auctionroom/auctionlog.cpp`)
// ниже — единый владелец auction log вместе с data-контрактом выше.

// Журнал аукциона WorldServer из точной пары EXE/PDB, перенесённый в Realm `auction/`.
//
// `BTreeMap<i32, Vec<_>>` сохраняет numeric key-order и insertion order равных
// ключей. Страница содержит до 17 записей; page transitions и bool results
// сохраняют исходные асимметрии. Goods-log выбирает последнюю вставленную
// подходящую запись, а `CollectNoNotice` не фильтрует уже отмеченные records:
// он меняет live flag и ставит SQL в FIFO по одному в исходном порядке.
// Owned records и chrono заменяют MSVC/CRT без batch transaction, claim или
// нормализации повреждённых дат.

use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDateTime, Timelike};
use encoding_rs::WINDOWS_1251;
use futures_util::TryStreamExt;
use tiberius::Row;

use crate::persistence::rssetup::WorldTdsClient;
use crate::app::world_message::{CMessage, SendMessageError};
use nebokrai_shared::network::ServerCommandHandle;

const AUCTION_LOG_RECORDS_PER_PAGE: i32 = 17;

/// Оригинал 12-байтовый элемент рассылки `AuctionMostGoods`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(C)]
pub struct AuctionBangNode {
    pub base_id: u32,
    pub count: u32,
    pub operation: u32,
}

const _: () = assert!(std::mem::size_of::<AuctionBangNode>() == 0x0c);

/// Первая UB-граница C-string сериализации одной страницы.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuctionLogPageBlock {
    pub player_id: i32,
    pub record_index: usize,
}

/// Наблюдаемый результат `AddByteAtCurPage` до возможной string-границы.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuctionLogPageWriteDisposition {
    MissingMessage,
    MissingPage,
    Written { page: i32, record_count: usize },
}

/// Наблюдаемый результат `AddByteGoodsLog`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuctionGoodsLogWriteOutcome {
    MissingMessage,
    Written { found: bool },
    BlockedMissingFact(AuctionLogTimeBlock),
}

/// Граница исходной `CWriteLogQueue::PushWriteLogData`.
pub trait AuctionNoticeWriteQueue {
    fn push_auction_notice_sql(&mut self, sql: String);
}

/// Итог `CollectNoNotice` после публикации SQL и полного wire payload.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuctionNoticeCollection {
    pub player_id: i32,
    pub record_count: usize,
}

/// Две recordset-стадии оригинал `LoadItem`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuctionLogLoadStage {
    History,
    Ranking,
}

/// Причина исходного `false` без публикации credentials или SQL values.
#[derive(Debug)]
pub enum AuctionLogLoadFailure {
    MissingConnection,
    Database {
        stage: AuctionLogLoadStage,
        row_index: Option<usize>,
        source: tiberius::error::Error,
    },
    MissingRequiredValue {
        stage: AuctionLogLoadStage,
        row_index: usize,
        column: &'static str,
    },
}

/// Локальная UB/partial-value граница одной history-строки.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuctionLogLoadBlockSource {
    DescriptionOverflow { encoded_length: usize },
    MalformedGuid { column: &'static str },
    CalendarOutsideSystemTime,
}

/// Уже действующий prefix перед безопасно неразрешимой строкой.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuctionLogLoadBlock {
    pub row_index: usize,
    pub source: AuctionLogLoadBlockSource,
}

/// Полный доказанный итог `CAuctionLog::LoadItem`.
#[derive(Debug)]
pub enum AuctionLogLoadOutcome {
    ReturnedTrue,
    ReturnedFalse(AuctionLogLoadFailure),
    BlockedMissingFact(AuctionLogLoadBlock),
}

/// Этап оригинал неатомарного `CAuctionLog::UpdateAuctionBangDB`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuctionBangUpdateStage {
    MostMoneyQuery,
    MostMoneyRow,
    MostMoneyUpdate,
    MostCountQuery,
    MostCountRow,
    MostCountUpdate,
}

/// Причина исходного `false` ежедневного обновления ranking-а.
#[derive(Debug)]
pub enum AuctionBangUpdateFailure {
    MissingConnection,
    Database {
        stage: AuctionBangUpdateStage,
        source: tiberius::error::Error,
    },
    MissingRow {
        stage: AuctionBangUpdateStage,
    },
    MissingRequiredValue {
        stage: AuctionBangUpdateStage,
        column: &'static str,
    },
}

/// Полный доказанный boolean-итог `UpdateAuctionBangDB`.
#[derive(Debug)]
pub enum AuctionBangUpdateOutcome {
    ReturnedTrue,
    ReturnedFalse(AuctionBangUpdateFailure),
}

enum AuctionHistoryRowDecode {
    Database(tiberius::error::Error),
    MissingRequiredValue(&'static str),
    Blocked(AuctionLogLoadBlockSource),
}

/// Owned-состояние исходного `CAuctionLog` без process-static singleton-а.
pub struct CAuctionLog {
    /// Нулевой sentinel гарантирует первый daily-ranking проход: допустимый
    /// `tm_mday` лежит в диапазоне `1..=31`. Это безопасная замена чтения
    /// неинициализированного слова в исходном constructor-е.
    old_auction_day: i32,
    log_list: BTreeMap<i32, Vec<AuctionLogNode>>,
    player_pages: BTreeMap<i32, i32>,
    goods_list: Vec<AuctionBangNode>,
}

impl Default for CAuctionLog {
    fn default() -> Self {
        Self::new()
    }
}

impl CAuctionLog {
    /// Создаёт только доказанные constructor-ом пустые контейнеры.
    pub const fn new() -> Self {
        Self {
            old_auction_day: 0,
            log_list: BTreeMap::new(),
            player_pages: BTreeMap::new(),
            goods_list: Vec::new(),
        }
    }

    pub const fn old_auction_day(&self) -> i32 {
        self.old_auction_day
    }

    pub fn set_old_auction_day(&mut self, day: i32) {
        self.old_auction_day = day;
    }

    /// Multimap insert всегда принимает ещё одну запись и возвращает `true`.
    pub fn add_item(&mut self, item: AuctionLogNode) -> bool {
        self.log_list.entry(item.player_id).or_default().push(item);
        true
    }

    /// Потоково загружает live history, затем очищает и загружает ranking.
    pub async fn load_item(
        &mut self,
        active_connection: Option<&mut WorldTdsClient>,
        increment_log_days: u32,
    ) -> AuctionLogLoadOutcome {
        let Some(active_connection) = active_connection else {
            return AuctionLogLoadOutcome::ReturnedFalse(AuctionLogLoadFailure::MissingConnection);
        };

        let history_sql = format!(
            "SELECT * FROM AuctionLog WHERE DATEDIFF(day,log_time,GETDATE())<={} ORDER BY PlayerID, log_time",
            increment_log_days as i32,
        );
        let mut history = match active_connection.simple_query(history_sql).await {
            Ok(history) => history,
            Err(source) => {
                return AuctionLogLoadOutcome::ReturnedFalse(AuctionLogLoadFailure::Database {
                    stage: AuctionLogLoadStage::History,
                    row_index: None,
                    source,
                });
            }
        };
        let mut history_row_index = 0usize;
        loop {
            let item = match history.try_next().await {
                Ok(Some(item)) => item,
                Ok(None) => break,
                Err(source) => {
                    return AuctionLogLoadOutcome::ReturnedFalse(AuctionLogLoadFailure::Database {
                        stage: AuctionLogLoadStage::History,
                        row_index: Some(history_row_index),
                        source,
                    });
                }
            };
            let Some(row) = item.into_row() else {
                continue;
            };
            let node = match decode_auction_history_row(&row) {
                Ok(node) => node,
                Err(AuctionHistoryRowDecode::Database(source)) => {
                    return AuctionLogLoadOutcome::ReturnedFalse(AuctionLogLoadFailure::Database {
                        stage: AuctionLogLoadStage::History,
                        row_index: Some(history_row_index),
                        source,
                    });
                }
                Err(AuctionHistoryRowDecode::MissingRequiredValue(column)) => {
                    return AuctionLogLoadOutcome::ReturnedFalse(
                        AuctionLogLoadFailure::MissingRequiredValue {
                            stage: AuctionLogLoadStage::History,
                            row_index: history_row_index,
                            column,
                        },
                    );
                }
                Err(AuctionHistoryRowDecode::Blocked(source)) => {
                    return AuctionLogLoadOutcome::BlockedMissingFact(AuctionLogLoadBlock {
                        row_index: history_row_index,
                        source,
                    });
                }
            };
            let _legacy_result = self.add_item(node);
            history_row_index += 1;
        }
        drop(history);

        let mut ranking = match active_connection
            .simple_query("SELECT * FROM auctionmostgoods ")
            .await
        {
            Ok(ranking) => ranking,
            Err(source) => {
                return AuctionLogLoadOutcome::ReturnedFalse(AuctionLogLoadFailure::Database {
                    stage: AuctionLogLoadStage::Ranking,
                    row_index: None,
                    source,
                });
            }
        };
        self.goods_list.clear();
        let mut ranking_row_index = 0usize;
        loop {
            let item = match ranking.try_next().await {
                Ok(Some(item)) => item,
                Ok(None) => break,
                Err(source) => {
                    return AuctionLogLoadOutcome::ReturnedFalse(AuctionLogLoadFailure::Database {
                        stage: AuctionLogLoadStage::Ranking,
                        row_index: Some(ranking_row_index),
                        source,
                    });
                }
            };
            let Some(row) = item.into_row() else {
                continue;
            };
            let node = match decode_auction_bang_row(&row) {
                Ok(node) => node,
                Err(AuctionHistoryRowDecode::Database(source)) => {
                    return AuctionLogLoadOutcome::ReturnedFalse(AuctionLogLoadFailure::Database {
                        stage: AuctionLogLoadStage::Ranking,
                        row_index: Some(ranking_row_index),
                        source,
                    });
                }
                Err(AuctionHistoryRowDecode::MissingRequiredValue(column)) => {
                    return AuctionLogLoadOutcome::ReturnedFalse(
                        AuctionLogLoadFailure::MissingRequiredValue {
                            stage: AuctionLogLoadStage::Ranking,
                            row_index: ranking_row_index,
                            column,
                        },
                    );
                }
                Err(AuctionHistoryRowDecode::Blocked(_)) => {
                    unreachable!("ranking row не содержит локальных raw-buffer границ")
                }
            };
            self.goods_list.push(node);
            ranking_row_index += 1;
        }

        AuctionLogLoadOutcome::ReturnedTrue
    }

    /// Неатомарно пересчитывает две оригинал строки `AuctionMostGoods`.
    pub async fn update_auction_bang_db(
        &mut self,
        active_connection: Option<&mut WorldTdsClient>,
    ) -> AuctionBangUpdateOutcome {
        let Some(active_connection) = active_connection else {
            return AuctionBangUpdateOutcome::ReturnedFalse(
                AuctionBangUpdateFailure::MissingConnection,
            );
        };

        self.goods_list.clear();

        let most_money = match query_first_auction_bang(
            active_connection,
            "select dwbaseid,Moneynum from Auctionlog where MoneyNum=(select max(moneynum)from auctionlog where moneytype = 1 and opttype = -1 and DateDiff(day, log_time, getdate())< 30) ",
            "moneynum",
            2,
            AuctionBangUpdateStage::MostMoneyQuery,
            AuctionBangUpdateStage::MostMoneyRow,
        )
        .await
        {
            Ok(node) => node,
            Err(failure) => {
                return AuctionBangUpdateOutcome::ReturnedFalse(failure);
            }
        };
        self.goods_list.push(most_money);
        let most_money_update = format!(
            "update AuctionMostGoods SET dwbaseid = {},dwNum = {} where dwOpt = 2",
            most_money.base_id as i32, most_money.count as i32,
        );
        if let Err(source) = active_connection
            .execute(most_money_update.as_str(), &[])
            .await
        {
            return AuctionBangUpdateOutcome::ReturnedFalse(AuctionBangUpdateFailure::Database {
                stage: AuctionBangUpdateStage::MostMoneyUpdate,
                source,
            });
        }

        let most_count = match query_first_auction_bang(
            active_connection,
            "SELECT dwbaseid,SUM(Amount) as dwCount FROM AuctionLog where opttype = -1 AND DateDiff(day, log_time, getdate())< 30 GROUP BY dwbaseid ORDER BY dwCount DESC",
            "dwCount",
            1,
            AuctionBangUpdateStage::MostCountQuery,
            AuctionBangUpdateStage::MostCountRow,
        )
        .await
        {
            Ok(node) => node,
            Err(failure) => {
                return AuctionBangUpdateOutcome::ReturnedFalse(failure);
            }
        };
        self.goods_list.push(most_count);
        let most_count_update = format!(
            "update AuctionMostGoods SET dwbaseid = {},dwNum = {} where dwOpt = 1",
            most_count.base_id as i32, most_count.count as i32,
        );
        if let Err(source) = active_connection
            .execute(most_count_update.as_str(), &[])
            .await
        {
            return AuctionBangUpdateOutcome::ReturnedFalse(AuctionBangUpdateFailure::Database {
                stage: AuctionBangUpdateStage::MostCountUpdate,
                source,
            });
        }

        AuctionBangUpdateOutcome::ReturnedTrue
    }

    /// Сохраняет оригинал переход page и исторически странный bool результата.
    pub fn compute_page(&mut self, direction: i32, player_id: i32) -> bool {
        let Some(page) = self.player_pages.get_mut(&player_id) else {
            self.player_pages.insert(player_id, 0);
            return true;
        };

        match direction {
            1 => {
                let count = self.log_list.get(&player_id).map_or(0, Vec::len) as u32 as i32;
                *page = page.wrapping_add(1);
                if count < page.wrapping_mul(AUCTION_LOG_RECORDS_PER_PAGE) {
                    *page = page.wrapping_sub(1);
                }
                false
            }
            2 if *page > 0 => {
                *page -= 1;
                true
            }
            2 => false,
            _ => {
                *page = 0;
                true
            }
        }
    }

    /// Добавляет player/page и до семнадцати сокращённых записей.
    pub fn add_byte_at_current_page(
        &self,
        player_id: i32,
        message: Option<&mut CMessage>,
    ) -> Result<AuctionLogPageWriteDisposition, AuctionLogPageBlock> {
        let Some(message) = message else {
            return Ok(AuctionLogPageWriteDisposition::MissingMessage);
        };
        let Some(&page) = self.player_pages.get(&player_id) else {
            return Ok(AuctionLogPageWriteDisposition::MissingPage);
        };

        let records = self
            .log_list
            .get(&player_id)
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let skip = page.wrapping_mul(AUCTION_LOG_RECORDS_PER_PAGE);
        let page_records = if skip < 0 {
            &[][..]
        } else {
            records
                .get(usize::try_from(skip).expect("неотрицательный i32 помещается в usize")..)
                .unwrap_or(&[])
        };
        let page_records = &page_records[..page_records
            .len()
            .min(AUCTION_LOG_RECORDS_PER_PAGE as usize)];

        message.base_mut().add_long(player_id);
        message.base_mut().add_long(page_records.len() as i32);
        for (record_index, record) in page_records.iter().enumerate() {
            message.base_mut().add(&record.time.to_legacy_bytes());
            message.base_mut().add_long(record.operation_type);
            message.base_mut().add_long(record.money_type);
            message.base_mut().add_long(record.money_num);
            message.base_mut().add_long(record.amount);
            message.base_mut().add_long(record.fee);
            let Some(description) = record.description_wire_bytes() else {
                return Err(AuctionLogPageBlock {
                    player_id,
                    record_index,
                });
            };
            message.base_mut().add(description);
        }

        Ok(AuctionLogPageWriteDisposition::Written {
            page,
            record_count: page_records.len(),
        })
    }

    /// Добавляет последнюю вставленную подходящую запись одного GUID.
    pub fn add_byte_goods_log(
        &self,
        player_id: i32,
        guid: CGuid,
        message: Option<&mut CMessage>,
    ) -> AuctionGoodsLogWriteOutcome {
        let Some(message) = message else {
            return AuctionGoodsLogWriteOutcome::MissingMessage;
        };

        let mut selected = None;
        if let Some(records) = self.log_list.get(&player_id) {
            for record in records {
                if record.player_id != player_id || record.guid != guid {
                    continue;
                }
                match record.time.is_after_zero_baseline() {
                    Ok(true) => selected = Some(record),
                    Ok(false) => {}
                    Err(block) => {
                        return AuctionGoodsLogWriteOutcome::BlockedMissingFact(block);
                    }
                }
            }
        }

        let Some(record) = selected else {
            message.base_mut().add_long(0);
            return AuctionGoodsLogWriteOutcome::Written { found: false };
        };
        message.base_mut().add_long(1);
        message.base_mut().add_long(record.money_type);
        message.base_mut().add(&record.time.to_legacy_bytes());
        message.base_mut().add_long(record.money_num);
        message.base_mut().add_long(record.amount);
        AuctionGoodsLogWriteOutcome::Written { found: true }
    }

    /// Публикует все записи игрока, включая ранее отмеченные, в оригинал порядке.
    pub fn collect_no_notice<Q: AuctionNoticeWriteQueue>(
        &mut self,
        player_id: i32,
        message: &mut CMessage,
        write_queue: &mut Q,
    ) -> AuctionNoticeCollection {
        let mut snapshots = Vec::new();
        if let Some(records) = self.log_list.get_mut(&player_id) {
            snapshots.reserve(records.len());
            for record in records {
                let snapshot = record.clone();
                record.notice = 1;
                write_queue.push_auction_notice_sql(format!(
                    "UPDATE AuctionLog set bNotice = 1 where guidkey = '{}' and guid = '{}' and opttype = {}",
                    snapshot.guid_key, snapshot.guid, snapshot.operation_type,
                ));
                snapshots.push(snapshot);
            }
        }

        message.base_mut().add_long(player_id);
        message.base_mut().add_long(snapshots.len() as i32);
        for snapshot in &snapshots {
            message.base_mut().add(&snapshot.to_legacy_bytes());
        }

        AuctionNoticeCollection {
            player_id,
            record_count: snapshots.len(),
        }
    }

    /// Строит `0x8040C` и сохраняет исходную отправку по numeric map ID.
    pub fn send_auction_msg_to_game_server(
        &self,
        player_id: u32,
        map_id: u32,
        sender: Option<&ServerCommandHandle>,
    ) -> Result<i32, SendMessageError> {
        let mut message = CMessage::new(0x0008_040c);
        message.base_mut().add_ulong(player_id);
        for goods in &self.goods_list {
            message.base_mut().add_ulong(goods.base_id);
            message.base_mut().add_ulong(goods.count);
            message.base_mut().add_ulong(goods.operation);
        }
        message.send_to_map_id(sender, map_id as i32)
    }
}

fn decode_auction_history_row(row: &Row) -> Result<AuctionLogNode, AuctionHistoryRowDecode> {
    macro_rules! required {
        ($type:ty, $column:literal) => {
            match row.try_get::<$type, _>($column) {
                Ok(Some(value)) => value,
                Ok(None) => {
                    return Err(AuctionHistoryRowDecode::MissingRequiredValue($column));
                }
                Err(error) => return Err(AuctionHistoryRowDecode::Database(error)),
            }
        };
    }

    let time = required!(NaiveDateTime, "log_time");
    let year = u16::try_from(time.year()).map_err(|_| {
        AuctionHistoryRowDecode::Blocked(AuctionLogLoadBlockSource::CalendarOutsideSystemTime)
    })?;
    let time = AuctionLogSystemTime {
        year,
        month: u16::try_from(time.month()).expect("chrono month помещается в SYSTEMTIME"),
        day_of_week: u16::try_from(time.weekday().num_days_from_sunday())
            .expect("chrono weekday помещается в SYSTEMTIME"),
        day: u16::try_from(time.day()).expect("chrono day помещается в SYSTEMTIME"),
        hour: u16::try_from(time.hour()).expect("chrono hour помещается в SYSTEMTIME"),
        minute: u16::try_from(time.minute()).expect("chrono minute помещается в SYSTEMTIME"),
        second: u16::try_from(time.second()).expect("chrono second помещается в SYSTEMTIME"),
        milliseconds: u16::try_from(time.nanosecond() / 1_000_000)
            .expect("миллисекунды помещаются в SYSTEMTIME"),
    };
    let base_id = required!(i32, "dwbaseid");
    let money_num = required!(i32, "moneynum");
    let money_type = required!(i32, "moneytype");
    let operation_type = required!(i32, "optType");
    let player_id = required!(i32, "playerid");
    let amount = required!(i32, "amount");
    let fee = required!(i32, "sxf");
    let notice = required!(i32, "bNotice");

    let guid_text = row
        .try_get::<&str, _>("guid")
        .map_err(AuctionHistoryRowDecode::Database)?;
    let guid = CGuid::from_legacy_text(guid_text).map_err(|_| {
        AuctionHistoryRowDecode::Blocked(AuctionLogLoadBlockSource::MalformedGuid {
            column: "guid",
        })
    })?;
    let guid_key_text = row
        .try_get::<&str, _>("guidKey")
        .map_err(AuctionHistoryRowDecode::Database)?;
    let guid_key = CGuid::from_legacy_text(guid_key_text).map_err(|_| {
        AuctionHistoryRowDecode::Blocked(AuctionLogLoadBlockSource::MalformedGuid {
            column: "guidKey",
        })
    })?;

    let description_text = required!(&str, "strdescri");
    let (description_text, _, _) = WINDOWS_1251.encode(description_text);
    if description_text.len() >= AUCTION_LOG_DESCRIPTION_SIZE {
        return Err(AuctionHistoryRowDecode::Blocked(
            AuctionLogLoadBlockSource::DescriptionOverflow {
                encoded_length: description_text.len(),
            },
        ));
    }
    let mut description = [0; AUCTION_LOG_DESCRIPTION_SIZE];
    description[..description_text.len()].copy_from_slice(&description_text);

    Ok(AuctionLogNode {
        base_id,
        operation_type,
        money_type,
        money_num,
        player_id,
        amount,
        fee,
        notice,
        time,
        description,
        guid,
        guid_key,
    })
}

fn decode_auction_bang_row(row: &Row) -> Result<AuctionBangNode, AuctionHistoryRowDecode> {
    fn required_i32(row: &Row, column: &'static str) -> Result<i32, AuctionHistoryRowDecode> {
        row.try_get::<i32, _>(column)
            .map_err(AuctionHistoryRowDecode::Database)?
            .ok_or(AuctionHistoryRowDecode::MissingRequiredValue(column))
    }

    Ok(AuctionBangNode {
        base_id: required_i32(row, "dwbaseid")? as u32,
        count: required_i32(row, "dwnum")? as u32,
        operation: required_i32(row, "dwopt")? as u32,
    })
}

async fn query_first_auction_bang(
    active_connection: &mut WorldTdsClient,
    sql: &'static str,
    count_column: &'static str,
    operation: u32,
    query_stage: AuctionBangUpdateStage,
    row_stage: AuctionBangUpdateStage,
) -> Result<AuctionBangNode, AuctionBangUpdateFailure> {
    let mut rows = active_connection
        .simple_query(sql)
        .await
        .map_err(|source| AuctionBangUpdateFailure::Database {
            stage: query_stage,
            source,
        })?;
    loop {
        let item = rows
            .try_next()
            .await
            .map_err(|source| AuctionBangUpdateFailure::Database {
                stage: row_stage,
                source,
            })?;
        let Some(item) = item else {
            return Err(AuctionBangUpdateFailure::MissingRow { stage: row_stage });
        };
        let Some(row) = item.into_row() else {
            continue;
        };
        let base_id = row
            .try_get::<i32, _>("dwbaseid")
            .map_err(|source| AuctionBangUpdateFailure::Database {
                stage: row_stage,
                source,
            })?
            .ok_or(AuctionBangUpdateFailure::MissingRequiredValue {
                stage: row_stage,
                column: "dwbaseid",
            })? as u32;
        let count = row
            .try_get::<i32, _>(count_column)
            .map_err(|source| AuctionBangUpdateFailure::Database {
                stage: row_stage,
                source,
            })?
            .ok_or(AuctionBangUpdateFailure::MissingRequiredValue {
                stage: row_stage,
                column: count_column,
            })? as u32;
        return Ok(AuctionBangNode {
            base_id,
            count,
            operation,
        });
    }
}
