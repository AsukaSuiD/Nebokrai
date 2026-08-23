//! GameServer lifecycle-ветви `appbilling/servermessage.cpp`, подтверждённые
//! `billingserver.exe` и `billingserver.pdb`.
//!
//! Connect сначала назначает socket как map ID, затем публикует событие.
//! Disconnect всегда проверяет переданный адрес и `port as u16`, независимо от
//! admission-флага allow-list; signed payload сохраняется, хотя старый журнал
//! показывал те же биты как unsigned. Nullable server-owner выражен `Option`.

use std::net::Ipv4Addr;

use crate::nets::netbilling::message::CMessage;
use crate::nets::netbilling::serverforgs::CServerForGS;

const GAME_SERVER_CONNECTED_MESSAGE_TYPE: i32 = 0x000E_F101;
const GAME_SERVER_DISCONNECTED_MESSAGE_TYPE: i32 = 0x0010_EF01;
const GAME_SERVER_ADDRESS_LIMIT: usize = 0x20;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum ServerMessageOutcome {
    Connected {
        socket_id: i32,
        address: Ipv4Addr,
        map_assignment: Option<i32>,
    },
    Disconnected {
        map_id: i32,
        address: Vec<u8>,
        port: i32,
        allowed: bool,
    },
    Unsupported { message_type: i32 },
}

pub(crate) struct ServerMessageHandler<'a> {
    server: Option<&'a CServerForGS>,
}

impl<'a> ServerMessageHandler<'a> {
    pub(crate) const fn new(server: Option<&'a CServerForGS>) -> Self {
        Self { server }
    }

    pub(crate) fn on_server_message(&self, message: &mut CMessage) -> ServerMessageOutcome {
        match message.message_type() {
            GAME_SERVER_CONNECTED_MESSAGE_TYPE => self.on_connected(message),
            GAME_SERVER_DISCONNECTED_MESSAGE_TYPE => self.on_disconnected(message),
            message_type => ServerMessageOutcome::Unsupported { message_type },
        }
    }

    fn on_connected(&self, message: &CMessage) -> ServerMessageOutcome {
        let socket_id = message.socket_id();
        let map_assignment = self.server.map(|server| {
            server
                .command_handle()
                .set_client_map_id(socket_id, socket_id)
        });
        let address = Ipv4Addr::from(message.ip().to_le_bytes());

        ServerMessageOutcome::Connected {
            socket_id,
            address,
            map_assignment,
        }
    }

    fn on_disconnected(&self, message: &mut CMessage) -> ServerMessageOutcome {
        let map_id = message.base_mut().get_long().unwrap_or(0);
        let address = message
            .base_mut()
            .get_str_bytes(GAME_SERVER_ADDRESS_LIMIT)
            .expect("ненулевая GetStr-граница задана константой");
        let port = message.base_mut().get_long().unwrap_or(0);
        let allowed = self
            .server
            .is_some_and(|server| server.is_allowed_address(&address, port as u16));

        ServerMessageOutcome::Disconnected {
            map_id,
            address,
            port,
            allowed,
        }
    }
}
