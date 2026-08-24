//! Исполняемый owner изменения количества container-object `0xC0102` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message packaging/cs2ccontainerobjectamountchange.cpp`.
//! Сохраняется полный wire: source container type/id/extend/position, object
//! type/GUID и итоговое amount. Текущий достигнутый caller отправляет packet
//! одному player; `SendToSession` остаётся за session fan-out owner-ом.

use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

const CONTAINER_OBJECT_AMOUNT_CHANGE_MESSAGE: i32 = 0x000c_0102;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CS2CContainerObjectAmountChange {
    source_container_type: i32,
    source_container_id: i32,
    source_container_extend_id: i32,
    source_container_position: u32,
    object_type: i32,
    object_id: CGuid,
    amount: u32,
}

impl Default for CS2CContainerObjectAmountChange {
    fn default() -> Self {
        Self {
            source_container_type: 0,
            source_container_id: 0,
            source_container_extend_id: 0,
            source_container_position: 0,
            object_type: 0,
            object_id: CGuid::GUID_INVALID,
            amount: 0,
        }
    }
}

impl CS2CContainerObjectAmountChange {
    pub(crate) fn set_source_container(&mut self, object_type: i32, object_id: i32, position: u32) {
        self.source_container_type = object_type;
        self.source_container_id = object_id;
        self.source_container_position = position;
    }

    pub(crate) fn set_source_container_extend_id(&mut self, extend_id: i32) {
        self.source_container_extend_id = extend_id;
    }

    pub(crate) fn set_object(&mut self, object_type: i32, object_id: CGuid) {
        self.object_type = object_type;
        self.object_id = object_id;
    }

    pub(crate) fn set_object_amount(&mut self, amount: u32) {
        self.amount = amount;
    }

    pub(crate) fn send_to_player(&self, game: &CGame, player_id: i32) -> i32 {
        if player_id == 0 {
            return 0;
        }
        let mut message = CMessage::new(CONTAINER_OBJECT_AMOUNT_CHANGE_MESSAGE);
        message.add_long(self.source_container_type);
        message.add_long(self.source_container_id);
        message.add_long(self.source_container_extend_id);
        message.add_ulong(self.source_container_position);
        message.add_long(self.object_type);
        message.base_mut().add_guid(self.object_id);
        message.add_ulong(self.amount);
        message.send_to_player(game.net_server(), player_id)
    }
}
