//! Сообщения AuthServer и GMA LoginServer из `asmessage.cpp`.
//!
//! Статус всех трёх функций файла — `IMPLEMENTED`: `OnGMAKickPlayer` RVA
//! `0x0007F110`, `OnGMAMessage` `0x0007F2A0`, `OnASMessage` `0x0007F320`.
//!
//! Точная пара: `LoginServer/loginserver.exe + LoginServer/LoginServer.pdb`;
//! SHA-256 EXE
//! `1C84006DF612053B007D69E0243497A8DA85E10FB1D825D0B462F016747E7876`,
//! SHA-256 PDB
//! `FBBCEB3B18F72DECB57B2178063E946233703DD7C298738DE929E9A1C98A902C`.
//! Исходный путь PDB:
//! `d:\complite_version\fengyun_russia\trunk\server\loginserver\applogin\message\asmessage.cpp`.
//!
//! `0xCF301` сначала явно закрывает текущий Auth client и только затем заменяет
//! прежний reconnect-thread управляемой Tokio-задачей. Она немедленно пробует
//! весь `aslist.ini`, после неуспеха ждёт исходные пять секунд и при успехе
//! публикует owned replacement в ту же FIFO прежнего клиента. Поэтому
//! `ReassignAS` всё ещё исполняется в исходной позиции очереди, а Win32
//! `PostThreadMessage(0x464)`, handle и передача указателя через `0xCF302` не
//! получают небезопасных Rust-аналогов. Wire-пакет `0xCF302` отвергается
//! отдельно: в оригинале этот тип был только внутрипроцессным pointer-event.
//!
//! `0xCF601` передаётся фактическому `AuthManager`, который удаляет pending
//! quest и синхронно вызывает точную `AuthListener`-границу реализованного
//! embedded `AuthHandler` через `CGame`; результат также возвращается как outcome
//! для logging/runtime-owner. GMA ветви меняют opcode того же сообщения и
//! используют реальные Auth/World send-владельцы `CGame`. `GetWorldIDByName`
//! сохраняет byte-exact имя, connected-state и sentinel `-1`. Для отсутствующего
//! мира ответ `0xCF801` содержит request ID, нулевой result, operator/account и
//! diagnostic с исходным world name.
//!
//! Пропущенный декомпилятором vararg этой diagnostic-строки точечно подтверждён
//! машинным кодом LoginServer `0x0047F1CF..0x0047F1E4`: перед `sprintf` в стек
//! кладётся адрес буфера world name. Ограничение обоих `GetStr` равно `0x100`;
//! отсутствие NUL в пределах буфера сохраняет пустой результат и уже
//! сдвинутый курсор. SEH, stack cookie, временные C-массивы и ручные
//! деструкторы заменены владеющими `Vec`/`CMessage`.

use std::error::Error;
use std::fmt;

use crate::loginserver::loginserver::authmanager::{AuthManager, AuthResponseOutcome};
use crate::loginserver::loginserver::game::{AuthLifecycleError, CGame, GameRouteError};
use crate::nets::basemessage::CBaseMessage;
use crate::nets::netlogin::message::CMessage;

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

/// Какой исходный обработчик встретил неизвестный opcode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum UnknownAsMessageOwner {
    /// `OnASMessage` после исключения доказанных ветвей.
    Auth,
    /// `OnGMAMessage` после исключения четырёх доказанных ветвей.
    Gma,
}

/// Наблюдаемый результат одного обработчика без прежнего `AddLogText`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum AsMessageOutcome {
    /// Доказанная ветвь полностью выполнила свои side effects.
    Handled,
    /// `AuthManager` разобрал ответ и синхронно вызвал `AuthListener`.
    AuthResponse(AuthResponseOutcome),
    /// Старый warning представлен структурированно будущему logging-owner.
    Unknown {
        /// Обработчик, владевший warning.
        owner: UnknownAsMessageOwner,
        /// Полный исходный тип сообщения.
        message_type: i32,
    },
}

/// Ошибка безопасной границы доказанных AS/GMA маршрутов.
#[derive(Debug)]
pub(crate) enum AsMessageError {
    /// Фактический Auth/World transport-owner недоступен либо не собрал frame.
    Route(GameRouteError),
    /// Управляемый reconnect-owner не смог быть запущен.
    Reconnect(AuthLifecycleError),
    /// В wire пришёл исторический тип, содержавший только process-local pointer.
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

/// Конкретная композиция `g_pGame` и `gAuthMgr` исходного message-файла.
pub(crate) struct AsMessageHandlers<'a> {
    game: &'a mut CGame,
    auth_manager: &'a mut AuthManager,
}

impl<'a> AsMessageHandlers<'a> {
    /// Связывает обработчик с `CGame`, содержащим embedded listener-owner.
    pub(crate) fn new(game: &'a mut CGame, auth_manager: &'a mut AuthManager) -> Self {
        Self { game, auth_manager }
    }

    /// Выполняет `OnASMessage`, включая упорядоченный close/reconnect путь.
    pub(crate) async fn on_as_message(
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

    /// Выполняет `OnGMAMessage` для сообщений как от Auth, так и от GM-входа.
    pub(crate) fn on_gma_message(
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
