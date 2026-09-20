//! Исполняемый owner клиентского container-move `0xC0101` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/message packaging/cs2ccontainerobjectmove.cpp`. Реализация
//! сохраняет полный fixed prefix, self-move normalization и разный tail:
//! delete пишет только source amount, move/switch — обе amount, new — длину
//! и old-client stream. Ground-goods owner использует тот же собранный
//! `CMessage` для spatial around-send; выбор получателей остаётся у
//! `GameServerAroundRuntime`. `SendToSession` разрешает player-owner-ов через
//! ordered plug registry `CSessionFactory`. Rust `Drop` заменяет технический
//! MSVC destructor без отдельного adapter-а.

use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

const CONTAINER_OBJECT_MOVE_MESSAGE: i32 = 0x000c_0101;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum ContainerObjectMoveOperation {
    #[default]
    RollBack = 0,
    MoveObject = 1,
    NewObject = 2,
    DeleteObject = 3,
    SwitchObject = 4,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CS2CContainerObjectMove {
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
    pub(crate) fn set_operation(&mut self, operation: ContainerObjectMoveOperation) {
        self.operation = operation;
    }

    pub(crate) fn set_source_container(&mut self, object_type: i32, object_id: i32, position: u32) {
        self.source_container_type = object_type;
        self.source_container_id = object_id;
        self.source_container_position = position;
    }

    pub(crate) fn set_destination_container(
        &mut self,
        object_type: i32,
        object_id: i32,
        position: u32,
    ) {
        self.destination_container_type = object_type;
        self.destination_container_id = object_id;
        self.destination_container_position = position;
    }

    pub(crate) fn set_source_container_extend_id(&mut self, extend_id: i32) {
        self.source_container_extend_id = extend_id;
    }

    pub(crate) fn set_destination_container_extend_id(&mut self, extend_id: i32) {
        self.destination_container_extend_id = extend_id;
    }

    pub(crate) fn set_source_object(&mut self, object_type: i32, object_id: CGuid, amount: u32) {
        self.source_object_type = object_type;
        self.source_object_id = object_id;
        self.source_object_amount = amount;
    }

    pub(crate) fn set_destination_object(&mut self, object_type: i32, object_id: CGuid) {
        self.destination_object_type = object_type;
        self.destination_object_id = object_id;
    }

    pub(crate) fn set_destination_object_amount(&mut self, amount: u32) {
        self.destination_object_amount = amount;
    }

    pub(crate) fn set_object_stream(&mut self, object_stream: Vec<u8>) {
        self.object_stream = object_stream;
    }

    pub(crate) fn send_to_player(mut self, game: &CGame, player_id: i32) -> i32 {
        if player_id == 0 {
            return 0;
        }
        self.normalize_self_move();
        self.message().send_to_player(game.net_server(), player_id)
    }

    pub(crate) fn send_to_session(mut self, game: &CGame, session_id: i32) -> Vec<i32> {
        self.normalize_self_move();
        game.session_player_ids(session_id)
            .into_iter()
            .map(|player_id| self.message().send_to_player(game.net_server(), player_id))
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

    pub(crate) fn message(&self) -> CMessage {
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
