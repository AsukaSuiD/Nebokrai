//! Изменение количества container-object `0xC0102` GameServer: build-only
//! кодек кадра source container/object/amount.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` (SHA-256 EXE
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`, RSDS
//! `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2), исходный owner
//! `appserver/message packaging/cs2ccontainerobjectamountchange.cpp`.
//! Сохраняется полный wire: source container type/id/extend/position, object
//! type/GUID и итоговое amount.
//!
//! Build-only граница: кадр собирается здесь (`message`); прямая доставка
//! одному player и разрешение получателей `SendToSession` через ordered plug
//! registry `CSessionFactory` остаются у старого пакета за общим швом
//! [`ContainerObjectMessageSender`][super::cs2ccontainerobjectmove::ContainerObjectMessageSender].

use nebokrai_shared::values::CGuid;

use crate::app::game_message::CMessage;

use super::cs2ccontainerobjectmove::ContainerObjectMessageSender;

const CONTAINER_OBJECT_AMOUNT_CHANGE_MESSAGE: i32 = 0x000c_0102;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CS2CContainerObjectAmountChange {
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
    pub fn set_source_container(&mut self, object_type: i32, object_id: i32, position: u32) {
        self.source_container_type = object_type;
        self.source_container_id = object_id;
        self.source_container_position = position;
    }

    pub fn set_source_container_extend_id(&mut self, extend_id: i32) {
        self.source_container_extend_id = extend_id;
    }

    pub fn set_object(&mut self, object_type: i32, object_id: CGuid) {
        self.object_type = object_type;
        self.object_id = object_id;
    }

    pub fn set_object_amount(&mut self, amount: u32) {
        self.amount = amount;
    }

    /// Собранный кадр `0xC0102`: source container type/id/extend/position,
    /// object type/GUID и итоговое amount.
    pub fn message(&self) -> CMessage {
        let mut message = CMessage::new(CONTAINER_OBJECT_AMOUNT_CHANGE_MESSAGE);
        message.add_long(self.source_container_type);
        message.add_long(self.source_container_id);
        message.add_long(self.source_container_extend_id);
        message.add_ulong(self.source_container_position);
        message.add_long(self.object_type);
        message.base_mut().add_guid(self.object_id);
        message.add_ulong(self.amount);
        message
    }

    pub fn send_to_player<Sender: ContainerObjectMessageSender>(
        &self,
        sender: &Sender,
        player_id: i32,
    ) -> i32 {
        if player_id == 0 {
            return 0;
        }
        sender.send_container_object_message(&self.message(), player_id)
    }

    pub fn send_to_session<Sender: ContainerObjectMessageSender>(
        &self,
        sender: &Sender,
        session_id: i32,
    ) -> Vec<i32> {
        sender
            .session_player_ids(session_id)
            .into_iter()
            .map(|player_id| self.send_to_player(sender, player_id))
            .collect()
    }
}
