//! WorldServer dispatcher-owner `OnGMMessage`.
//!
//! Статус `IMPLEMENTED_PARTIAL`: exact ветви `0x5FF01` RVA
//! `0x000AB3C8..0x000AB425` и `0x5FF05` RVA `0x000AB98C..0x000ABA09`
//! материализуют online-count и online-player-ID queries. Общий owner до
//! switch сначала читает request/player ID. Первая ветвь затем читает script
//! ID, берёт 32-битное число `m_lOnlinePlayer` и строит
//! `0x7FC01 + request_id + online_count + script_id`; вторая читает bounded
//! имя и script ID и строит `0x7FC05 + request_id + found_id + script_id`.
//! Оба ответа уходят в исходный socket. Порядок чтения, signed wire-биты и
//! `SendToSocket`
//! подтверждены машинным кодом `Nworldserver.exe`; добавленные Linux-веткой
//! clamp и error-log отсутствуют в EXE и не перенесены.
//! Также материализованы transport-only ветви `0x5FF02/03/07/08/09/0A/0D/0E/0F`
//! и `0x5FF10/11/13/14/15/16`: они создают либо переписывают точные response
//! opcodes, сохраняют исходный payload, где это делал EXE, и используют ровно
//! исходные `SendToSocket`, `SendToMapID` либо `SendAll`.
//!
//! Rust `VecDeque::len` шире старого 32-битного `_Mysize`; значение вне
//! legacy-range безопасно блокируется typed-исходом, а не молча обрезается.
//! Остальные GM opcodes остаются `UNKNOWN` (исследовательский декомпилят хранится локально) ниже. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, исходный owner
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\gmmessage.cpp`.

use std::ffi::CString;

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::worldserver::game::CGame;

const ONLINE_PLAYER_COUNT_REQUEST: i32 = 0x0005_FF01;
const ONLINE_PLAYER_COUNT_RESPONSE: i32 = 0x0007_FC01;
const ONLINE_PLAYER_ID_REQUEST: i32 = 0x0005_FF05;
const ONLINE_PLAYER_ID_RESPONSE: i32 = 0x0007_FC05;

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGmTransportOutcome {
    PlayerNameBroadcast {
        request_type: i32,
        response_type: i32,
        request_id: i32,
        player_name: Vec<u8>,
        online_player_id: u32,
        text: Vec<u8>,
        source_map_id: i32,
        value: i32,
        numeric_payload_complete: [bool; 2],
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    FreshMapRoute {
        request_type: i32,
        response_type: i32,
        request_id: i32,
        first_value: i32,
        target_map_id: i32,
        second_value: i32,
        payload_complete: [bool; 4],
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    Broadcast {
        request_type: i32,
        response_type: i32,
        reused_request: bool,
        request_id_field: Option<i32>,
        appended_map_id: Option<i32>,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    PlayerRoute {
        request_type: i32,
        response_type: i32,
        player_id: i32,
        player_id_complete: bool,
        game_server_id: i32,
        wire: Option<Vec<u8>>,
        delivery: Option<Result<i32, SendMessageError>>,
    },
    DirectMapRoute {
        request_type: i32,
        response_type: i32,
        request_id: i32,
        discarded_value: i32,
        target_map_id: i32,
        payload_complete: [bool; 3],
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    NamedPlayerRoute {
        request_type: i32,
        request_id: i32,
        request_id_complete: bool,
        player_name: Vec<u8>,
        online_player_id: u32,
        target_game_server_id: i32,
        disposition: WorldGmNamedPlayerRouteDisposition,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGmNamedPlayerRouteDisposition {
    TargetRouted {
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    RequesterFallback {
        response_type: i32,
        requester_game_server_id: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    RequesterUnroutable,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGmOnlinePlayerCountOutcome {
    CountOutsideLegacyRange {
        request_id: i32,
        script_id: i32,
        payload_complete: [bool; 2],
        count: usize,
    },
    Responded {
        request_id: i32,
        script_id: i32,
        payload_complete: [bool; 2],
        online_count: i32,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldGmMessageOutcome {
    OnlinePlayerCount(WorldGmOnlinePlayerCountOutcome),
    OnlinePlayerId {
        request_id: i32,
        player_name: Vec<u8>,
        script_id: i32,
        numeric_payload_complete: [bool; 2],
        online_player_id: u32,
        response_type: i32,
        wire: Vec<u8>,
        delivery: Result<i32, SendMessageError>,
    },
    Transport(WorldGmTransportOutcome),
}

pub(crate) enum WorldGmMessageDispatch {
    Handled(WorldGmMessageOutcome),
    Pending(CMessage),
}

/// Исполняет достигнутые query-ветви частичного GM-owner-а.
pub(crate) fn on_gm_message(game: &CGame, mut message: CMessage) -> WorldGmMessageDispatch {
    let decoded_request_id = message.base_mut().get_long();
    let request_id = decoded_request_id.unwrap_or(0);
    match message.message_type() {
        ONLINE_PLAYER_COUNT_REQUEST => {
            let decoded_script_id = message.base_mut().get_long();
            let script_id = decoded_script_id.unwrap_or(0);
            let payload_complete = [decoded_request_id.is_some(), decoded_script_id.is_some()];
            let count = game.online_player_count();
            let Ok(online_count_bits) = u32::try_from(count) else {
                return WorldGmMessageDispatch::Handled(
                    WorldGmMessageOutcome::OnlinePlayerCount(
                        WorldGmOnlinePlayerCountOutcome::CountOutsideLegacyRange {
                            request_id,
                            script_id,
                            payload_complete,
                            count,
                        },
                    ),
                );
            };
            let online_count = online_count_bits as i32;

            let mut response = CMessage::new(ONLINE_PLAYER_COUNT_RESPONSE);
            response.base_mut().add_long(request_id);
            response.base_mut().add_long(online_count);
            response.base_mut().add_long(script_id);
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_to_socket(
                game.current_game_server_sender().as_ref(),
                message.socket_id(),
            );
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::OnlinePlayerCount(
                WorldGmOnlinePlayerCountOutcome::Responded {
                    request_id,
                    script_id,
                    payload_complete,
                    online_count,
                    response_type: ONLINE_PLAYER_COUNT_RESPONSE,
                    wire,
                    delivery,
                },
            ))
        }
        ONLINE_PLAYER_ID_REQUEST => {
            let player_name = message
                .base_mut()
                .get_str_bytes(0x100)
                .expect("literal 0x100 исключает zero-capacity GetStr");
            let decoded_script_id = message.base_mut().get_long();
            let script_id = decoded_script_id.unwrap_or(0);
            let online_player_id = game.online_player_id_by_name(&player_name);

            let mut response = CMessage::new(ONLINE_PLAYER_ID_RESPONSE);
            response.base_mut().add_long(request_id);
            response.base_mut().add_ulong(online_player_id);
            response.base_mut().add_long(script_id);
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_to_socket(
                game.current_game_server_sender().as_ref(),
                message.socket_id(),
            );
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::OnlinePlayerId {
                request_id,
                player_name,
                script_id,
                numeric_payload_complete: [
                    decoded_request_id.is_some(),
                    decoded_script_id.is_some(),
                ],
                online_player_id,
                response_type: ONLINE_PLAYER_ID_RESPONSE,
                wire,
                delivery,
            })
        }
        0x0005_FF02 => {
            let player_name = message
                .base_mut()
                .get_str_bytes(0x100)
                .expect("literal 0x100 исключает zero-capacity GetStr");
            let text = message
                .base_mut()
                .get_str_bytes(0x100)
                .expect("literal 0x100 исключает zero-capacity GetStr");
            let decoded_value = message.base_mut().get_long();
            let value = decoded_value.unwrap_or(0);
            let online_player_id = game.online_player_id_by_name(&player_name);
            let source_map_id = message.map_id();

            let mut response = CMessage::new(0x0007_FC03);
            response.base_mut().add_long(request_id);
            response.base_mut().add_ulong(online_player_id);
            add_c_string(&mut response, &text);
            response.base_mut().add_long(source_map_id);
            response.base_mut().add_long(value);
            let wire = response.as_wire_bytes().to_vec();
            let delivery = response.send_all(game.current_game_server_sender().as_ref());
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::Transport(
                WorldGmTransportOutcome::PlayerNameBroadcast {
                    request_type: 0x0005_FF02,
                    response_type: 0x0007_FC03,
                    request_id,
                    player_name,
                    online_player_id,
                    text,
                    source_map_id,
                    value,
                    numeric_payload_complete: [
                        decoded_request_id.is_some(),
                        decoded_value.is_some(),
                    ],
                    wire,
                    delivery,
                },
            ))
        }
        0x0005_FF03 => {
            let decoded_first = message.base_mut().get_long();
            let decoded_target = message.base_mut().get_long();
            let decoded_second = message.base_mut().get_long();
            let first_value = decoded_first.unwrap_or(0);
            let target_map_id = decoded_target.unwrap_or(0);
            let second_value = decoded_second.unwrap_or(0);
            let mut response = CMessage::new(0x0007_FC02);
            response.base_mut().add_long(request_id);
            response.base_mut().add_long(first_value);
            response.base_mut().add_long(second_value);
            let wire = response.as_wire_bytes().to_vec();
            let delivery = game.send_msg_to_game_server(target_map_id, &response);
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::Transport(
                WorldGmTransportOutcome::FreshMapRoute {
                    request_type: 0x0005_FF03,
                    response_type: 0x0007_FC02,
                    request_id,
                    first_value,
                    target_map_id,
                    second_value,
                    payload_complete: [
                        decoded_request_id.is_some(),
                        decoded_first.is_some(),
                        decoded_target.is_some(),
                        decoded_second.is_some(),
                    ],
                    wire,
                    delivery,
                },
            ))
        }
        0x0005_FF07 => {
            let player_name = message
                .base_mut()
                .get_str_bytes(0x100)
                .expect("literal 0x100 исключает zero-capacity GetStr");
            let online_player_id = game.online_player_id_by_name(&player_name);
            let target_game_server_id =
                game.game_server_number_by_player_id(online_player_id as i32);
            let disposition = if target_game_server_id != 0 {
                message.set_message_type(0x0007_FC06);
                let wire = message.as_wire_bytes().to_vec();
                let delivery = message.send_to_map_id(
                    game.current_game_server_sender().as_ref(),
                    target_game_server_id,
                );
                WorldGmNamedPlayerRouteDisposition::TargetRouted {
                    response_type: 0x0007_FC06,
                    wire,
                    delivery,
                }
            } else {
                let requester_game_server_id = game.game_server_number_by_player_id(request_id);
                if requester_game_server_id == 0 {
                    WorldGmNamedPlayerRouteDisposition::RequesterUnroutable
                } else {
                    let mut response = CMessage::new(0x0007_FC08);
                    response.base_mut().add_long(request_id);
                    response.base_mut().add_char(0);
                    add_c_string(&mut response, &player_name);
                    let wire = response.as_wire_bytes().to_vec();
                    let delivery = response.send_to_map_id(
                        game.current_game_server_sender().as_ref(),
                        requester_game_server_id,
                    );
                    WorldGmNamedPlayerRouteDisposition::RequesterFallback {
                        response_type: 0x0007_FC08,
                        requester_game_server_id,
                        wire,
                        delivery,
                    }
                }
            };
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::Transport(
                WorldGmTransportOutcome::NamedPlayerRoute {
                    request_type: 0x0005_FF07,
                    request_id,
                    request_id_complete: decoded_request_id.is_some(),
                    player_name,
                    online_player_id,
                    target_game_server_id,
                    disposition,
                },
            ))
        }
        0x0005_FF08 => {
            let map_id = message.map_id();
            message.set_message_type(0x0007_FC07);
            message.base_mut().add_long(map_id);
            handled_broadcast(
                game,
                message,
                0x0005_FF08,
                0x0007_FC07,
                true,
                None,
                Some(map_id),
            )
        }
        0x0005_FF09 => handled_player_route(
            game,
            message,
            0x0005_FF09,
            0x0007_FC08,
            request_id,
            decoded_request_id.is_some(),
        ),
        0x0005_FF0A => {
            let mut response = CMessage::new(0x0007_FC09);
            response.base_mut().add_long(request_id);
            handled_broadcast(
                game,
                response,
                0x0005_FF0A,
                0x0007_FC09,
                false,
                Some(request_id),
                None,
            )
        }
        0x0005_FF0D => handled_player_route(
            game,
            message,
            0x0005_FF0D,
            0x0007_FC0C,
            request_id,
            decoded_request_id.is_some(),
        ),
        0x0005_FF0E => handled_rewritten_broadcast(game, message, 0x0005_FF0E, 0x0007_FC0D),
        0x0005_FF0F => handled_rewritten_broadcast(game, message, 0x0005_FF0F, 0x0007_FC0E),
        0x0005_FF10 => handled_player_route(
            game,
            message,
            0x0005_FF10,
            0x0007_FC0F,
            request_id,
            decoded_request_id.is_some(),
        ),
        0x0005_FF11 => handled_rewritten_broadcast(game, message, 0x0005_FF11, 0x0007_FC10),
        0x0005_FF13 => handled_broadcast(
            game,
            CMessage::new(0x0007_F803),
            0x0005_FF13,
            0x0007_F803,
            false,
            None,
            None,
        ),
        0x0005_FF14 => {
            let map_id = message.map_id();
            let mut response = CMessage::new(0x0007_FC11);
            response.base_mut().add_long(request_id);
            response.base_mut().add_long(map_id);
            handled_broadcast(
                game,
                response,
                0x0005_FF14,
                0x0007_FC11,
                false,
                Some(request_id),
                Some(map_id),
            )
        }
        0x0005_FF15 => {
            let decoded_discarded = message.base_mut().get_long();
            let decoded_target = message.base_mut().get_long();
            let discarded_value = decoded_discarded.unwrap_or(0);
            let target_map_id = decoded_target.unwrap_or(0);
            message.set_message_type(0x0007_FC12);
            let wire = message.as_wire_bytes().to_vec();
            let delivery = game.send_msg_to_game_server(target_map_id, &message);
            WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::Transport(
                WorldGmTransportOutcome::DirectMapRoute {
                    request_type: 0x0005_FF15,
                    response_type: 0x0007_FC12,
                    request_id,
                    discarded_value,
                    target_map_id,
                    payload_complete: [
                        decoded_request_id.is_some(),
                        decoded_discarded.is_some(),
                        decoded_target.is_some(),
                    ],
                    wire,
                    delivery,
                },
            ))
        }
        0x0005_FF16 => handled_rewritten_broadcast(game, message, 0x0005_FF16, 0x0007_FC13),
        _ => WorldGmMessageDispatch::Pending(message),
    }
}

fn handled_rewritten_broadcast(
    game: &CGame,
    mut message: CMessage,
    request_type: i32,
    response_type: i32,
) -> WorldGmMessageDispatch {
    message.set_message_type(response_type);
    handled_broadcast(
        game,
        message,
        request_type,
        response_type,
        true,
        None,
        None,
    )
}

fn handled_broadcast(
    game: &CGame,
    message: CMessage,
    request_type: i32,
    response_type: i32,
    reused_request: bool,
    request_id_field: Option<i32>,
    appended_map_id: Option<i32>,
) -> WorldGmMessageDispatch {
    let wire = message.as_wire_bytes().to_vec();
    let delivery = message.send_all(game.current_game_server_sender().as_ref());
    WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::Transport(
        WorldGmTransportOutcome::Broadcast {
            request_type,
            response_type,
            reused_request,
            request_id_field,
            appended_map_id,
            wire,
            delivery,
        },
    ))
}

fn handled_player_route(
    game: &CGame,
    mut message: CMessage,
    request_type: i32,
    response_type: i32,
    player_id: i32,
    player_id_complete: bool,
) -> WorldGmMessageDispatch {
    let game_server_id = game.game_server_number_by_player_id(player_id);
    let (wire, delivery) = if game_server_id == 0 {
        (None, None)
    } else {
        message.set_message_type(response_type);
        let wire = message.as_wire_bytes().to_vec();
        let delivery = message.send_to_map_id(
            game.current_game_server_sender().as_ref(),
            game_server_id,
        );
        (Some(wire), Some(delivery))
    };
    WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::Transport(
        WorldGmTransportOutcome::PlayerRoute {
            request_type,
            response_type,
            player_id,
            player_id_complete,
            game_server_id,
            wire,
            delivery,
        },
    ))
}

fn add_c_string(message: &mut CMessage, bytes: &[u8]) {
    let value = CString::new(bytes).expect("bounded GetStr возвращает bytes до первого NUL");
    message.base_mut().add_str(Some(&value));
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\gmmessage.cpp

// ============================================================================
// FUNCTION: OnGMMessage
// STATUS: IMPLEMENTED_PARTIAL
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\gmmessage.cpp:19
// RVA: 0x000AB370
// ADDRESS: 004ab370
// PROTOTYPE: void __cdecl OnGMMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
