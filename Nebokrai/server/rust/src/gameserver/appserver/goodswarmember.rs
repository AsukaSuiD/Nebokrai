//! Состояние участников GoodsWar исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/goodswarmember.cpp/.h`, подтверждает две ordered collections:
//! member ID -> faction ID и множество участвующих faction ID, пять записей
//! счётчика, signed clamp только сверху и World-сообщения `0x60139`.
//! Constructor после очистки member map немедленно запрашивает полный список
//! subtype `4`; удаление положительного member отправляет subtype `2`, а
//! отрицательная парная запись удаляется без сообщения.
//!
//! `BTreeMap/BTreeSet` заменяют MSVC tree plumbing с тем же sorted key-order.
//! Имена счётчика остаются точными 20 wire bytes; client snapshot имеет opcode
//! `0xC0316` и порядок `count`, затем `AddEx(20 bytes) + signed count`.
//! Country-route `0x7FF20/0x7FF21` проходит живой FIFO целой family: subtype
//! sentinel loops, clear-before-read, prefix publication, World delete-send и
//! count clamp меняют тот же owner, который обслуживает client snapshot.

use std::collections::{BTreeMap, BTreeSet};

use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::{CMessage, SendMessageError};

const GOODS_WAR_WORLD_MESSAGE: i32 = 0x0006_0139;
const GOODS_WAR_CLIENT_COUNTS: i32 = 0x000C_0316;
const COUNT_CAPACITY: usize = 5;
const GOODS_WAR_MEMBER_MESSAGE: u32 = 0x0007_FF20;
const GOODS_WAR_COUNT_MESSAGE: u32 = 0x0007_FF21;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GoodsWarCount {
    pub(crate) faction_name: [u8; 20],
    pub(crate) count: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct GoodsWarMessageInputBlock {
    pub(crate) field: &'static str,
    pub(crate) offset: usize,
    pub(crate) needed: usize,
    pub(crate) available: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameGoodsWarMessageError {
    OwnerUnavailable,
    Input(GoodsWarMessageInputBlock),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GameGoodsWarMutationReport {
    IgnoredSubtype {
        subtype: i32,
    },
    MembersAdded {
        subtype: i32,
        inserted: usize,
    },
    MemberDeleted {
        notification: Option<Result<i32, SendMessageError>>,
    },
    FactionMembersDeleted {
        faction_id: i32,
    },
    MembersCleared,
    FactionIdsReplaced {
        declared: i32,
        retained: usize,
    },
    CountsReplaced {
        declared: i32,
        effective: i32,
        decoded: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GameGoodsWarMessageReport {
    pub(crate) opcode: u32,
    pub(crate) mutation: GameGoodsWarMutationReport,
}

#[derive(Debug, Default)]
pub(crate) struct CGoodsWarMember {
    members: BTreeMap<i32, i32>,
    faction_ids: BTreeSet<i32>,
    counts: [GoodsWarCount; COUNT_CAPACITY],
    count_size: i32,
}

impl CGoodsWarMember {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Constructor-side request полного member snapshot у WorldServer.
    pub(crate) fn request_initial_state(&self, game: &CGame) -> Result<i32, SendMessageError> {
        let mut message = CMessage::new(GOODS_WAR_WORLD_MESSAGE);
        message.add_long(4);
        message.send(game, false)
    }

    pub(crate) fn insert_count(&mut self, index: i32, faction_name: [u8; 20], count: i32) {
        let Ok(index) = usize::try_from(index) else {
            return;
        };
        if let Some(slot) = self.counts.get_mut(index) {
            *slot = GoodsWarCount {
                faction_name,
                count,
            };
        }
    }

    /// Исходный owner ограничивает только верхнюю границу; negative сохраняется.
    pub(crate) fn set_max_count(&mut self, count: i32) -> i32 {
        self.count_size = count.min(COUNT_CAPACITY as i32);
        self.count_size
    }

    /// Возвращает 0, 1 либо 2 за наличие ключей `member_id` и `-member_id`.
    pub(crate) fn is_goods_war_member(&self, member_id: i32) -> i32 {
        i32::from(self.members.contains_key(&member_id))
            + i32::from(self.members.contains_key(&member_id.wrapping_neg()))
    }

    pub(crate) fn clear_all_faction_ids(&mut self) {
        self.faction_ids.clear();
    }

    pub(crate) fn contains_faction_id(&self, faction_id: i32) -> bool {
        self.faction_ids.contains(&faction_id)
    }

    /// Сохраняет exact partial effects: positive erase и World send идут раньше
    /// безусловной попытки удалить отрицательную парную запись.
    pub(crate) fn delete_one_member(
        &mut self,
        member_id: i32,
        game: &CGame,
    ) -> Option<Result<i32, SendMessageError>> {
        let notification = if self.members.remove(&member_id).is_some() {
            let mut message = CMessage::new(GOODS_WAR_WORLD_MESSAGE);
            message.add_long(2);
            message.add_long(member_id);
            Some(message.send(game, false))
        } else {
            None
        };
        self.members.remove(&member_id.wrapping_neg());
        notification
    }

    pub(crate) fn delete_members_by_faction_id(&mut self, faction_id: i32) {
        self.members
            .retain(|_, member_faction_id| *member_faction_id != faction_id);
    }

    pub(crate) fn insert_faction_id(&mut self, faction_id: i32) -> bool {
        self.faction_ids.insert(faction_id)
    }

    /// Existing member mapping не перезаписывается.
    pub(crate) fn insert_one_member(&mut self, member_id: i32, faction_id: i32) {
        self.members.entry(member_id).or_insert(faction_id);
    }

    pub(crate) fn clear_members(&mut self) {
        self.members.clear();
    }

    pub(crate) fn request_goods_war_list(&self, player_id: i32, game: &CGame) -> Option<i32> {
        game.find_player(player_id)?;
        let mut message = CMessage::new(GOODS_WAR_CLIENT_COUNTS);
        message.add_long(self.count_size);
        if self.count_size > 0 {
            for entry in self.counts.iter().take(self.count_size as usize) {
                message.base_mut().add_ex(&entry.faction_name);
                message.add_long(entry.count);
            }
        }
        Some(message.send_to_player(game.net_server(), player_id))
    }

    pub(crate) fn members(&self) -> &BTreeMap<i32, i32> {
        &self.members
    }

    pub(crate) fn faction_ids(&self) -> &BTreeSet<i32> {
        &self.faction_ids
    }

    pub(crate) fn counts(&self) -> &[GoodsWarCount; COUNT_CAPACITY] {
        &self.counts
    }

    pub(crate) const fn count_size(&self) -> i32 {
        self.count_size
    }
}

/// Обрабатывает обе достигнутые GoodsWar country-route ветви, сохраняя
/// sentinel loops, clear-before-read и prefix publication при short payload.
pub(crate) fn dispatch_game_goods_war_message(
    message: &mut CMessage,
    game: &mut CGame,
) -> Option<Result<GameGoodsWarMessageReport, GameGoodsWarMessageError>> {
    let opcode = message.message_type() as u32;
    if !matches!(opcode, GOODS_WAR_MEMBER_MESSAGE | GOODS_WAR_COUNT_MESSAGE) {
        return None;
    }
    let Some(mut owner) = game.take_goods_war() else {
        return Some(Err(GameGoodsWarMessageError::OwnerUnavailable));
    };
    let result = {
        let (source, cursor) = message.base_mut().wire_bytes_and_cursor_mut();
        if opcode == GOODS_WAR_MEMBER_MESSAGE {
            decode_goods_war_members(source, cursor, &mut owner, game)
        } else {
            decode_goods_war_counts(source, cursor, &mut owner)
        }
    };
    game.restore_goods_war(owner);
    Some(result.map(|mutation| GameGoodsWarMessageReport { opcode, mutation }))
}

fn decode_goods_war_members(
    source: &[u8],
    cursor: &mut usize,
    owner: &mut CGoodsWarMember,
    game: &CGame,
) -> Result<GameGoodsWarMutationReport, GameGoodsWarMessageError> {
    let subtype = read_goods_war_i32(source, cursor, "subtype")?;
    match subtype {
        1 => {
            let faction_id = read_goods_war_i32(source, cursor, "faction ID")?;
            let mut inserted = 0;
            if 0 < faction_id {
                loop {
                    let member_id = read_goods_war_i32(source, cursor, "member ID")?;
                    if member_id == 0 {
                        break;
                    }
                    owner.insert_one_member(member_id, faction_id);
                    inserted += 1;
                }
            }
            Ok(GameGoodsWarMutationReport::MembersAdded { subtype, inserted })
        }
        2 => {
            let member_id = read_goods_war_i32(source, cursor, "deleted member ID")?;
            Ok(GameGoodsWarMutationReport::MemberDeleted {
                notification: owner.delete_one_member(member_id, game),
            })
        }
        3 => {
            let faction_id = read_goods_war_i32(source, cursor, "deleted faction ID")?;
            owner.delete_members_by_faction_id(faction_id);
            Ok(GameGoodsWarMutationReport::FactionMembersDeleted { faction_id })
        }
        4 => {
            let mut inserted = 0;
            loop {
                let member_id = read_goods_war_i32(source, cursor, "snapshot member ID")?;
                if member_id == 0 {
                    break;
                }
                let faction_id = read_goods_war_i32(source, cursor, "snapshot faction ID")?;
                owner.insert_one_member(member_id, faction_id);
                inserted += 1;
            }
            Ok(GameGoodsWarMutationReport::MembersAdded { subtype, inserted })
        }
        5 => {
            owner.clear_members();
            Ok(GameGoodsWarMutationReport::MembersCleared)
        }
        0x10 => {
            owner.clear_all_faction_ids();
            let declared = read_goods_war_i32(source, cursor, "faction count")?;
            if 0 < declared {
                for _ in 0..declared {
                    let faction_id = read_goods_war_i32(source, cursor, "faction list ID")?;
                    owner.insert_faction_id(faction_id);
                }
            }
            Ok(GameGoodsWarMutationReport::FactionIdsReplaced {
                declared,
                retained: owner.faction_ids().len(),
            })
        }
        _ => Ok(GameGoodsWarMutationReport::IgnoredSubtype { subtype }),
    }
}

fn decode_goods_war_counts(
    source: &[u8],
    cursor: &mut usize,
    owner: &mut CGoodsWarMember,
) -> Result<GameGoodsWarMutationReport, GameGoodsWarMessageError> {
    let declared = read_goods_war_i32(source, cursor, "count size")?;
    let effective = owner.set_max_count(declared);
    let mut decoded = 0;
    if 0 < effective {
        for index in 0..effective {
            let faction_name = read_goods_war_array::<20>(source, cursor, "faction name")?;
            let count = read_goods_war_i32(source, cursor, "faction count value")?;
            owner.insert_count(index, faction_name, count);
            decoded += 1;
        }
    }
    Ok(GameGoodsWarMutationReport::CountsReplaced {
        declared,
        effective,
        decoded,
    })
}

fn read_goods_war_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, GameGoodsWarMessageError> {
    Ok(i32::from_le_bytes(read_goods_war_array::<4>(
        source, cursor, field,
    )?))
}

fn read_goods_war_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], GameGoodsWarMessageError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(N) else {
        return Err(GameGoodsWarMessageError::Input(GoodsWarMessageInputBlock {
            field,
            offset,
            needed: N,
            available,
        }));
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(GameGoodsWarMessageError::Input(GoodsWarMessageInputBlock {
            field,
            offset,
            needed: N,
            available,
        }));
    };
    *cursor = end;
    Ok(bytes.try_into().expect("длина GoodsWar field проверена"))
}
