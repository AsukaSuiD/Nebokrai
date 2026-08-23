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

use std::collections::{BTreeMap, BTreeSet};

use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::{CMessage, SendMessageError};

const GOODS_WAR_WORLD_MESSAGE: i32 = 0x0006_0139;
const GOODS_WAR_CLIENT_COUNTS: i32 = 0x000C_0316;
const COUNT_CAPACITY: usize = 5;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GoodsWarCount {
    pub(crate) faction_name: [u8; 20],
    pub(crate) count: i32,
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
