//! AuthServer/GMA-обработчики `asmessage.cpp`.
//!
//! Закрытие Auth-соединения предшествует запуску управляемого reconnect; успешная
//! замена возвращается в ту же FIFO-позицию. `0xCF302` остаётся только
//! внутрипроцессным pointer-event оригинала и отвергается на wire.
//! Ответ Auth удаляет pending quest до синхронного `AuthListener` callback.
//! GMA-ветви переиспользуют сообщение с новым opcode и фактические Auth/World
//! send-owner-ы. Ограниченные строки сохраняют сдвиг курсора и пустой результат
//! при отсутствии NUL.

use std::error::Error;
use std::fmt;

use super::authmanager::{AuthManager, AuthResponseOutcome};
use super::game::{AuthLifecycleError, CGame, GameRouteError};
use nebokrai_shared::network::CBaseMessage;
use crate::app::login_message::CMessage;

const GMA_KICK_PLAYER: i32 = 0x000C_F701;
const GMA_KICK_RESPONSE_TO_AUTH: i32 = 0x000C_F801;
const GMA_KICK_FORWARD_TO_WORLD: i32 = 0x0004_FD01;
const GMA_GET_SERVER_INFO: i32 = 0x0002_0101;
const GMA_GET_SERVER_INFO_TO_AUTH: i32 = 0x000C_F801;
const GMA_UPDATE_SERVER_INFO: i32 = 0x0002_0104;
const GMA_UPDATE_SERVER_INFO_TO_AUTH: i32 = 0x000C_F802;
const GMA_BROADCAST_FROM_AUTH: i32 = 0x000C_F702;
const GMA_BROADCAST_TO_WORLD: i32 = 0x0004_FD04;
const AUTH_CONNECTION_CLOSED: i32 = 0x000C_F301;
const LEGACY_RECONNECTED_POINTER: i32 = 0x000C_F302;
const AUTH_RESPONSE: i32 = 0x000C_F601;
const GMA_AUTH_RANGE_START: u32 = 0x000C_F700;
const GMA_AUTH_RANGE_END: u32 = 0x000C_F8FF;
const LEGACY_STRING_LIMIT: usize = 0x100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnknownAsMessageOwner {
    Auth,
    Gma,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AsMessageOutcome {
    Handled,
    AuthResponse(AuthResponseOutcome),
    Unknown {
        owner: UnknownAsMessageOwner,
        message_type: i32,
    },
}

#[derive(Debug)]
pub enum AsMessageError {
    Route(GameRouteError),
    Reconnect(AuthLifecycleError),
    LegacyReconnectPointerOnWire,
}

impl fmt::Display for AsMessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Route(error) => error.fmt(formatter),
            Self::Reconnect(error) => error.fmt(formatter),
            Self::LegacyReconnectPointerOnWire => formatter
                .write_str("wire 0xCF302 не является допустимым reconnect-событием LoginServer"),
        }
    }
}

impl Error for AsMessageError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Route(error) => Some(error),
            Self::Reconnect(error) => Some(error),
            Self::LegacyReconnectPointerOnWire => None,
        }
    }
}

impl From<GameRouteError> for AsMessageError {
    fn from(error: GameRouteError) -> Self {
        Self::Route(error)
    }
}

impl From<AuthLifecycleError> for AsMessageError {
    fn from(error: AuthLifecycleError) -> Self {
        Self::Reconnect(error)
    }
}

pub struct AsMessageHandlers<'a> {
    game: &'a mut CGame,
    auth_manager: &'a mut AuthManager,
}

impl<'a> AsMessageHandlers<'a> {
    pub fn new(game: &'a mut CGame, auth_manager: &'a mut AuthManager) -> Self {
        Self { game, auth_manager }
    }

    pub async fn on_as_message(
        &mut self,
        message: &mut CMessage,
    ) -> Result<AsMessageOutcome, AsMessageError> {
        let message_type = message.message_type();
        let opcode = message_type as u32;
        if GMA_AUTH_RANGE_START < opcode && opcode < GMA_AUTH_RANGE_END {
            return self.on_gma_message(message);
        }

        match message_type {
            AUTH_CONNECTION_CLOSED => {
                self.game.disconnect_as();
                self.game.start_reconnect_thread().await?;
                Ok(AsMessageOutcome::Handled)
            }
            LEGACY_RECONNECTED_POINTER => Err(AsMessageError::LegacyReconnectPointerOnWire),
            AUTH_RESPONSE => Ok(AsMessageOutcome::AuthResponse(
                self.auth_manager.on_response_auth(message, self.game),
            )),
            _ => Ok(AsMessageOutcome::Unknown {
                owner: UnknownAsMessageOwner::Auth,
                message_type,
            }),
        }
    }

    pub fn on_gma_message(
        &mut self,
        message: &mut CMessage,
    ) -> Result<AsMessageOutcome, AsMessageError> {
        match message.message_type() {
            GMA_KICK_PLAYER => self.on_gma_kick_player(message),
            GMA_GET_SERVER_INFO => {
                message.set_message_type(GMA_GET_SERVER_INFO_TO_AUTH);
                self.game.send_to_auth(message)?;
                Ok(AsMessageOutcome::Handled)
            }
            GMA_UPDATE_SERVER_INFO => {
                message.set_message_type(GMA_UPDATE_SERVER_INFO_TO_AUTH);
                message.base_mut().add_long(self.game.area_id());
                self.game.send_to_auth(message)?;
                Ok(AsMessageOutcome::Handled)
            }
            GMA_BROADCAST_FROM_AUTH => {
                message.set_message_type(GMA_BROADCAST_TO_WORLD);
                self.game.send_all_world(message)?;
                Ok(AsMessageOutcome::Handled)
            }
            message_type => Ok(AsMessageOutcome::Unknown {
                owner: UnknownAsMessageOwner::Gma,
                message_type,
            }),
        }
    }

    fn on_gma_kick_player(
        &self,
        message: &mut CMessage,
    ) -> Result<AsMessageOutcome, AsMessageError> {
        let request_id = message.base_mut().get_long().unwrap_or(0);
        let world_name = message
            .base_mut()
            .get_str_bytes(LEGACY_STRING_LIMIT)
            .expect("ненулевая граница GetStr задана константой");
        let reason = message.base_mut().get_char().unwrap_or(0);
        let target = message
            .base_mut()
            .get_str_bytes(LEGACY_STRING_LIMIT)
            .expect("ненулевая граница GetStr задана константой");
        let world_id = self.game.world_id_by_name(&world_name);

        if world_id == -1 {
            let mut response = CMessage::new(GMA_KICK_RESPONSE_TO_AUTH);
            response.base_mut().add_long(request_id);
            response.base_mut().add_char(0);
            add_legacy_string(response.base_mut(), &target);

            let mut diagnostic = b"Login Server : Invalid world server name : ".to_vec();
            diagnostic.extend_from_slice(&world_name);
            diagnostic.push(b'!');
            add_legacy_string(response.base_mut(), &diagnostic);
            self.game.send_to_auth(&response)?;
        } else {
            let mut forward = CMessage::new(GMA_KICK_FORWARD_TO_WORLD);
            forward.base_mut().add_long(request_id);
            forward.base_mut().add_char(reason);
            add_legacy_string(forward.base_mut(), &target);
            self.game.send_msg_to_world(&forward, world_id)?;
        }
        Ok(AsMessageOutcome::Handled)
    }
}

fn add_legacy_string(message: &mut CBaseMessage, value: &[u8]) {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    message.add(&value[..end]);
    message.add_byte(0);
}
