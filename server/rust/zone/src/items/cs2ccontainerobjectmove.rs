//! Клиентский container-move `0xC0101` GameServer: подкоды `SetOperation`,
//! self-move normalization и build-only кодек кадра.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` (SHA-256 EXE
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`, RSDS
//! `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2), исходный owner
//! `appserver/message packaging/cs2ccontainerobjectmove.cpp`. Кодековая форма
//! перенесена буквально: enum операции — он же подкод
//! `SetOperation` первого байта (`RollBack` 0, `MoveObject` 1, `NewObject` 2,
//! `DeleteObject` 3, `SwitchObject` 4), self-move normalization перед
//! публикацией и объектный кодек с разным tail по операции: rollback-кадр
//! состоит из одного байта операции, delete пишет только source amount,
//! move/switch — обе amount, new — длину и old-client stream.
//!
//! Build-only граница: здесь собирается кадр (`message`) и выполняется
//! normalization; сама доставка остаётся у старого пакета за швом
//! [`ContainerObjectMessageSender`] — RLE send-family одного player и
//! разрешение player-owner-ов session через ordered plug registry
//! `CSessionFactory`, реализованные `CGame`. Ground-goods owner старого
//! пакета использует тот же собранный `CMessage` для spatial around-send
//! без normalization; выбор получателей остаётся у
//! `GameServerAroundRuntime`. Tombstone `cc2scontainerobjectmove` (C2S
//! deserializer) — отдельный decode-owner, этим переносом не затрагивается.

use nebokrai_shared::values::CGuid;

use crate::app::game_message::CMessage;

const CONTAINER_OBJECT_MOVE_MESSAGE: i32 = 0x000c_0101;

/// Доставка собранных container-object кадров, остающаяся у старого пакета:
/// RLE send-family `CMessage` одному player и разрешение player-owner-ов
/// session реестром `CSessionFactory` (impl — у `CGame`).
pub trait ContainerObjectMessageSender {
    /// Доставляет собранный кадр одному player; возвращает queue result.
    fn send_container_object_message(&self, message: &CMessage, player_id: i32) -> i32;
    /// Player-owner-ы session в порядке ordered plug registry.
    fn session_player_ids(&self, session_id: i32) -> Vec<i32>;
}

/// Подкод операции `SetOperation` первого байта `0xC0101`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ContainerObjectMoveOperation {
    #[default]
    RollBack = 0,
    MoveObject = 1,
    NewObject = 2,
    DeleteObject = 3,
    SwitchObject = 4,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CS2CContainerObjectMove {
    operation: ContainerObjectMoveOperation,
    source_container_type: i32,
    source_container_id: i32,
    source_container_extend_id: i32,
    source_container_position: u32,
    destination_container_type: i32,
    destination_container_id: i32,
    destination_container_extend_id: i32,
    destination_container_position: u32,
    source_object_type: i32,
    source_object_id: CGuid,
    source_object_amount: u32,
    destination_object_type: i32,
    destination_object_id: CGuid,
    destination_object_amount: u32,
    object_stream: Vec<u8>,
}

impl Default for CS2CContainerObjectMove {
    fn default() -> Self {
        Self {
            operation: ContainerObjectMoveOperation::RollBack,
            source_container_type: 0,
            source_container_id: 0,
            source_container_extend_id: 0,
            source_container_position: 0,
            destination_container_type: 0,
            destination_container_id: 0,
            destination_container_extend_id: 0,
            destination_container_position: 0,
            source_object_type: 0,
            source_object_id: CGuid::GUID_INVALID,
            source_object_amount: 0,
            destination_object_type: 0,
            destination_object_id: CGuid::GUID_INVALID,
            destination_object_amount: 0,
            object_stream: Vec::new(),
        }
    }
}

impl CS2CContainerObjectMove {
    pub fn set_operation(&mut self, operation: ContainerObjectMoveOperation) {
        self.operation = operation;
    }

    pub fn set_source_container(&mut self, object_type: i32, object_id: i32, position: u32) {
        self.source_container_type = object_type;
        self.source_container_id = object_id;
        self.source_container_position = position;
    }

    pub fn set_destination_container(
        &mut self,
        object_type: i32,
        object_id: i32,
        position: u32,
    ) {
        self.destination_container_type = object_type;
        self.destination_container_id = object_id;
        self.destination_container_position = position;
    }

    pub fn set_source_container_extend_id(&mut self, extend_id: i32) {
        self.source_container_extend_id = extend_id;
    }

    pub fn set_destination_container_extend_id(&mut self, extend_id: i32) {
        self.destination_container_extend_id = extend_id;
    }

    pub fn set_source_object(&mut self, object_type: i32, object_id: CGuid, amount: u32) {
        self.source_object_type = object_type;
        self.source_object_id = object_id;
        self.source_object_amount = amount;
    }

    pub fn set_destination_object(&mut self, object_type: i32, object_id: CGuid) {
        self.destination_object_type = object_type;
        self.destination_object_id = object_id;
    }

    pub fn set_destination_object_amount(&mut self, amount: u32) {
        self.destination_object_amount = amount;
    }

    pub fn set_object_stream(&mut self, object_stream: Vec<u8>) {
        self.object_stream = object_stream;
    }

    pub fn send_to_player<Sender: ContainerObjectMessageSender>(
        mut self,
        sender: &Sender,
        player_id: i32,
    ) -> i32 {
        if player_id == 0 {
            return 0;
        }
        self.normalize_self_move();
        sender.send_container_object_message(&self.message(), player_id)
    }

    pub fn send_to_session<Sender: ContainerObjectMessageSender>(
        mut self,
        sender: &Sender,
        session_id: i32,
    ) -> Vec<i32> {
        self.normalize_self_move();
        sender
            .session_player_ids(session_id)
            .into_iter()
            .map(|player_id| sender.send_container_object_message(&self.message(), player_id))
            .collect()
    }

    fn normalize_self_move(&mut self) {
        if self.source_object_type != self.destination_object_type
            || self.source_object_id != self.destination_object_id
        {
            return;
        }
        self.destination_object_type = 0;
        self.destination_object_id = CGuid::GUID_INVALID;
        if self.source_container_type == self.destination_container_type
            && self.source_container_id == self.destination_container_id
            && self.source_container_extend_id == self.destination_container_extend_id
            && self.source_container_position == self.destination_container_position
        {
            self.operation = ContainerObjectMoveOperation::RollBack;
        }
    }

    /// Собранный кадр `0xC0101` без normalization: rollback — один байт
    /// операции; delete — только source amount; move/switch — обе amount;
    /// new — длина и old-client object stream.
    pub fn message(&self) -> CMessage {
        let mut message = CMessage::new(CONTAINER_OBJECT_MOVE_MESSAGE);
        message.add_byte(self.operation as u8);
        if self.operation == ContainerObjectMoveOperation::RollBack {
            return message;
        }
        message.add_long(self.source_container_type);
        message.add_long(self.source_container_id);
        message.add_long(self.source_container_extend_id);
        message.add_ulong(self.source_container_position);
        message.add_long(self.destination_container_type);
        message.add_long(self.destination_container_id);
        message.add_long(self.destination_container_extend_id);
        message.add_ulong(self.destination_container_position);
        message.add_long(self.source_object_type);
        message.base_mut().add_guid(self.source_object_id);
        message.add_long(self.destination_object_type);
        message.base_mut().add_guid(self.destination_object_id);
        if self.operation != ContainerObjectMoveOperation::NewObject {
            message.add_ulong(self.source_object_amount);
            if self.operation != ContainerObjectMoveOperation::DeleteObject {
                message.add_ulong(self.destination_object_amount);
            }
            return message;
        }
        message.add_ulong(self.object_stream.len() as u32);
        message.base_mut().add(&self.object_stream);
        message
    }
}
