//! WorldServer dispatcher-owner `OnGMAMessage`.
//!
//! Статус `IMPLEMENTED`: exact `0x004A5DB0..0x004A609F` материализован для
//! kick-player `0x4FD01` и transport branches `0x4FD04`, `0x60401`, `0x60402`.
//! Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, исходный owner
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\gmamessage.cpp`.
//! Kick-player сохраняет `_strcmpi` lookup аккаунта, online-list gate, точные
//! payload-ы ошибок LoginServer, отсутствие Login-ответа на успешном пути и
//! два `AddLogText` в исходном порядке. Небезопасные `char[256]`, `strcpy` и
//! `sprintf` заменены bounded bytes/CString без изменения wire и log-текста.
//! `0x4FD04 -> 0x80002` вызывает `SendAll`; `0x60401 -> 0x20101` вызывает
//! неприоритетный `Send(false)` в LoginServer; `0x60402 -> 0x20104` дописывает
//! signed source map ID, 32-битные биты `dwNumber` и также вызывает
//! `Send(false)`. Payload не читается, `Update` и ownership/tail gates
//! отсутствуют. До setup `dwNumber` в оригинале был неинициализирован; Rust не
//! выбирает произвольные биты и возвращает typed safe-block без внешнего send.
//! Любой opcode вне четырёх exact case завершает dispatcher без чтения,
//! отправки и fallback-маршрута; Rust представляет это `NoOp`.
//!
//! Декомпилятор: Ghidra 12.1.2. Сырой C++ ниже сохранён как локальная
//! документация, а не как Rust-реализация.

use std::ffi::CString;

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::worldserver::game::CGame;
use crate::worldserver::worldserver::worldserver::AddLogTextDisposition;

const KICK_PLAYER_REQUEST: i32 = 0x0004_FD01;
const KICK_PLAYER_RESPONSE: i32 = 0x0002_0101;
const KICK_PLAYER_COMMAND: i32 = 0x0008_0001;
const ROLE_NAME_SELECTOR: i8 = b'r' as i8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WorldGmaKickPlayerTargetKind {
    RoleName,
    Account,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGmaKickPlayerDisposition {
    InvalidPlayer {
        response_type: i32,
        error_text: Vec<u8>,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    MissingGameServer {
        response_type: i32,
        error_text: Vec<u8>,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    Routed {
        command_type: i32,
        game_server_id: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
        route_log: AddLogTextDisposition,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGmaMessageOutcome {
    /// Default полного exact `OnGMAMessage` без side effects.
    NoOp {
        request_type: i32,
    },
    KickPlayer {
        request_type: i32,
        request_id: i32,
        request_id_complete: bool,
        selector: i8,
        selector_complete: bool,
        target_kind: WorldGmaKickPlayerTargetKind,
        target: Vec<u8>,
        resolved_player_id: u32,
        resolved_player_name: Vec<u8>,
        receive_log: AddLogTextDisposition,
        disposition: WorldGmaKickPlayerDisposition,
    },
    LoginRelay {
        request_type: i32,
        response_type: i32,
        appended_map_id: Option<i32>,
        appended_world_number: Option<u32>,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    GameServerBroadcast {
        request_type: i32,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    MissingWorldNumber {
        request_type: i32,
        response_type: i32,
        map_id: i32,
    },
}

pub(crate) enum WorldGmaMessageDispatch {
    Handled(WorldGmaMessageOutcome),
    Pending(CMessage),
}

/// Исполняет полный exact `OnGMAMessage` и достигнутый kick-player helper.
pub(crate) fn on_gma_message(
    game: &CGame,
    mut message: CMessage,
    add_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
) -> WorldGmaMessageDispatch {
    let request_type = message.message_type();
    match request_type {
        KICK_PLAYER_REQUEST => on_kick_player(game, message, add_log_text),
        0x0004_FD04 => {
            let response_type = 0x0008_0002;
            message.set_message_type(response_type);
            let wire = message.as_wire_bytes().to_vec();
            let delivery = message.send_all(game.current_game_server_sender().as_ref());
            WorldGmaMessageDispatch::Handled(WorldGmaMessageOutcome::GameServerBroadcast {
                request_type,
                response_type,
                wire,
                delivery,
            })
        }
        0x0006_0401 => send_login_relay(game, message, request_type, 0x0002_0101, None),
        0x0006_0402 => {
            let response_type = 0x0002_0104;
            let map_id = message.map_id();
            let Some(world_number) = game.configured_world_number() else {
                return WorldGmaMessageDispatch::Handled(
                    WorldGmaMessageOutcome::MissingWorldNumber {
                        request_type,
                        response_type,
                        map_id,
                    },
                );
            };
            message.set_message_type(response_type);
            message.base_mut().add_long(map_id);
            message.base_mut().add_ulong(world_number);
            send_login_relay(
                game,
                message,
                request_type,
                response_type,
                Some((map_id, world_number)),
            )
        }
        _ => WorldGmaMessageDispatch::Handled(WorldGmaMessageOutcome::NoOp { request_type }),
    }
}

fn on_kick_player(
    game: &CGame,
    mut message: CMessage,
    add_log_text: &mut dyn FnMut(&[u8]) -> AddLogTextDisposition,
) -> WorldGmaMessageDispatch {
    let decoded_request_id = message.base_mut().get_long();
    let request_id = decoded_request_id.unwrap_or(0);
    let decoded_selector = message.base_mut().get_char();
    let selector = decoded_selector.unwrap_or(0);
    let target = message
        .base_mut()
        .get_str_bytes(0x100)
        .expect("literal 0x100 исключает zero-capacity GetStr");

    let mut receive_log_text = b"Receive KICK_PLAYER command [type : ".to_vec();
    receive_log_text.push(selector as u8);
    receive_log_text.extend_from_slice(b", name : ");
    receive_log_text.extend_from_slice(&target);
    receive_log_text.extend_from_slice(b"].");
    let receive_log = add_log_text(&receive_log_text);

    let target_kind = if selector == ROLE_NAME_SELECTOR {
        WorldGmaKickPlayerTargetKind::RoleName
    } else {
        WorldGmaKickPlayerTargetKind::Account
    };
    let (resolved_player_id, resolved_player_name) = match target_kind {
        WorldGmaKickPlayerTargetKind::RoleName => {
            (game.online_player_id_by_name(&target), target.clone())
        }
        WorldGmaKickPlayerTargetKind::Account => game
            .online_player_by_cdkey(&target)
            .map(|player| {
                (
                    player.get_id() as u32,
                    legacy_c_string_prefix(player.get_name()).to_vec(),
                )
            })
            .unwrap_or((0, Vec::new())),
    };

    let disposition = if resolved_player_id == 0 {
        let error_text = match target_kind {
            WorldGmaKickPlayerTargetKind::RoleName => {
                b"WorldServer : Invalid player role name !".to_vec()
            }
            WorldGmaKickPlayerTargetKind::Account => {
                b"WorldServer : Invalid player account !".to_vec()
            }
        };
        let (wire, delivery) = send_kick_failure(game, request_id, &target, &error_text);
        WorldGmaKickPlayerDisposition::InvalidPlayer {
            response_type: KICK_PLAYER_RESPONSE,
            error_text,
            wire,
            delivery,
        }
    } else {
        let game_server_id = game.game_server_number_by_player_id(resolved_player_id as i32);
        if game_server_id == 0 {
            let error_text =
                b"WorldServer : Can NOT get the game server the player is on !".to_vec();
            let (wire, delivery) = send_kick_failure(game, request_id, &target, &error_text);
            WorldGmaKickPlayerDisposition::MissingGameServer {
                response_type: KICK_PLAYER_RESPONSE,
                error_text,
                wire,
                delivery,
            }
        } else {
            let mut command = CMessage::new(KICK_PLAYER_COMMAND);
            command.base_mut().add_long(request_id);
            add_c_string(&mut command, &resolved_player_name);
            let wire = command.as_wire_bytes().to_vec();
            let delivery = command.send_to_map_id(
                game.current_game_server_sender().as_ref(),
                game_server_id,
            );
            let route_log_text =
                format!("Send KICK_PLAYER command to game server[{game_server_id}].").into_bytes();
            let route_log = add_log_text(&route_log_text);
            WorldGmaKickPlayerDisposition::Routed {
                command_type: KICK_PLAYER_COMMAND,
                game_server_id,
                wire,
                delivery,
                route_log,
            }
        }
    };

    WorldGmaMessageDispatch::Handled(WorldGmaMessageOutcome::KickPlayer {
        request_type: KICK_PLAYER_REQUEST,
        request_id,
        request_id_complete: decoded_request_id.is_some(),
        selector,
        selector_complete: decoded_selector.is_some(),
        target_kind,
        target,
        resolved_player_id,
        resolved_player_name,
        receive_log,
        disposition,
    })
}

fn send_kick_failure(
    game: &CGame,
    request_id: i32,
    target: &[u8],
    error_text: &[u8],
) -> (Vec<u8>, Result<i32, SendMessageError>) {
    let mut response = CMessage::new(KICK_PLAYER_RESPONSE);
    response.base_mut().add_long(request_id);
    response.base_mut().add_char(0);
    add_c_string(&mut response, target);
    add_c_string(&mut response, error_text);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    (wire, delivery)
}

fn add_c_string(message: &mut CMessage, bytes: &[u8]) {
    let value = CString::new(legacy_c_string_prefix(bytes))
        .expect("legacy C-string prefix не содержит внутреннего NUL");
    message.base_mut().add_str(Some(&value));
}

fn legacy_c_string_prefix(bytes: &[u8]) -> &[u8] {
    let length = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    &bytes[..length]
}

fn send_login_relay(
    game: &CGame,
    mut message: CMessage,
    request_type: i32,
    response_type: i32,
    appended: Option<(i32, u32)>,
) -> WorldGmaMessageDispatch {
    message.set_message_type(response_type);
    let wire = message.as_wire_bytes().to_vec();
    let delivery = message.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    WorldGmaMessageDispatch::Handled(WorldGmaMessageOutcome::LoginRelay {
        request_type,
        response_type,
        appended_map_id: appended.map(|value| value.0),
        appended_world_number: appended.map(|value| value.1),
        wire,
        delivery,
    })
}
