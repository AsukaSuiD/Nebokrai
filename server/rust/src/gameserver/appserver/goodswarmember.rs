//! Состояние участников GoodsWar исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный владелец —
//! `appserver/goodswarmember.cpp/.h`. Она подтверждает две упорядоченные
//! коллекции: соответствие ID участника и ID фракции, а также множество ID
//! участвующих фракций; отдельно хранятся пять записей счётчика. Ограничение
//! знакового значения применяется только сверху, изменения передаются World
//! сообщением `0x60139`.
//! Конструктор после очистки списка участников немедленно запрашивает полный
//! снимок с `subtype = 4`; удаление положительного ID отправляет `subtype = 2`,
//! а отрицательная парная запись удаляется без сообщения.
//!
//! `BTreeMap` и `BTreeSet` заменяют внутренние деревья MSVC с тем же порядком
//! ключей. Имена в счётчике сохраняют точные 20 байт протокола; клиентский
//! снимок имеет opcode `0xC0316` и порядок: `count`, затем `AddEx(20 bytes)` и
//! знаковый счётчик. Маршрут страны `0x7FF20/0x7FF21` проходит живую очередь
//! всего семейства: чтение до сигнального значения, очистка перед чтением,
//! публикация принятого префикса, сообщение World об удалении и ограничение
//! счётчика меняют того же владельца, который выдаёт клиентский снимок.

use std::collections::{BTreeMap, BTreeSet};

use crate::gameserver::gameserver::game::CGame;
use nebokrai_shared::protocol::LegacyReader;
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

    /// Запрос полного снимка участников у WorldServer при создании владельца.
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

    /// Исходный владелец ограничивает только верхнюю границу; отрицательное
    /// значение сохраняется.
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

    /// Сохраняет точный порядок частичных эффектов: удаление положительной
    /// записи и сообщение World предшествуют безусловной попытке удалить
    /// отрицательную парную запись.
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

    /// Существующее соответствие участника не перезаписывается.
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
/// чтение до сигнального значения, очистку перед чтением и публикацию принятого
/// префикса при коротком сообщении.
pub(crate) fn dispatch_game_goods_war_message(
    message: &mut CMessage,
    game: &mut CGame,
) -> Option<Result<(), GameGoodsWarMessageError>> {
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
    Some(result)
}

fn decode_goods_war_members(
    source: &[u8],
    cursor: &mut usize,
    owner: &mut CGoodsWarMember,
    game: &CGame,
) -> Result<(), GameGoodsWarMessageError> {
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
            tracing::trace!(subtype, inserted, "участники GoodsWar добавлены");
            Ok(())
        }
        2 => {
            let member_id = read_goods_war_i32(source, cursor, "deleted member ID")?;
            let notification = owner.delete_one_member(member_id, game);
            tracing::trace!(subtype, member_id, ?notification, "участник GoodsWar удалён");
            Ok(())
        }
        3 => {
            let faction_id = read_goods_war_i32(source, cursor, "deleted faction ID")?;
            owner.delete_members_by_faction_id(faction_id);
            tracing::trace!(subtype, faction_id, "участники фракции GoodsWar удалены");
            Ok(())
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
            tracing::trace!(subtype, inserted, "снимок участников GoodsWar применён");
            Ok(())
        }
        5 => {
            owner.clear_members();
            tracing::trace!(subtype, "участники GoodsWar очищены");
            Ok(())
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
            let retained = owner.faction_ids().len();
            tracing::trace!(subtype, declared, retained, "список фракций GoodsWar заменён");
            Ok(())
        }
        _ => {
            tracing::trace!(subtype, "неизвестный подтип GoodsWar проигнорирован");
            Ok(())
        }
    }
}

fn decode_goods_war_counts(
    source: &[u8],
    cursor: &mut usize,
    owner: &mut CGoodsWarMember,
) -> Result<(), GameGoodsWarMessageError> {
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
    tracing::trace!(declared, effective, decoded, "счётчики GoodsWar заменены");
    Ok(())
}

fn read_goods_war_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, GameGoodsWarMessageError> {
    let mut reader = goods_war_reader(source, *cursor, field, 4)?;
    let value = reader.read_i32().map_err(|block| goods_war_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_goods_war_array<const N: usize>(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<[u8; N], GameGoodsWarMessageError> {
    let mut reader = goods_war_reader(source, *cursor, field, N)?;
    let bytes = reader
        .read_bytes(N)
        .map_err(|block| goods_war_error(field, block))?;
    *cursor = reader.position();
    Ok(bytes.try_into().expect("прочитано точное число байт"))
}

fn goods_war_reader<'source>(
    source: &'source [u8],
    cursor: usize,
    field: &'static str,
    needed: usize,
) -> Result<LegacyReader<'source>, GameGoodsWarMessageError> {
    LegacyReader::at(source, cursor).map_err(|block| {
        GameGoodsWarMessageError::Input(GoodsWarMessageInputBlock {
            field,
            offset: block.offset,
            needed,
            available: block.available,
        })
    })
}

fn goods_war_error(
    field: &'static str,
    block: nebokrai_shared::protocol::LegacyReadBlock,
) -> GameGoodsWarMessageError {
    GameGoodsWarMessageError::Input(GoodsWarMessageInputBlock {
        field,
        offset: block.offset,
        needed: block.needed,
        available: block.available,
    })
}
