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
//!
//! Rust `VecDeque::len` шире старого 32-битного `_Mysize`; значение вне
//! legacy-range безопасно блокируется typed-исходом, а не молча обрезается.
//! Остальные GM opcodes остаются `UNKNOWN` (исследовательский декомпилят хранится локально) ниже. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, исходный owner
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\gmmessage.cpp`.

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::worldserver::game::CGame;

const ONLINE_PLAYER_COUNT_REQUEST: i32 = 0x0005_FF01;
const ONLINE_PLAYER_COUNT_RESPONSE: i32 = 0x0007_FC01;
const ONLINE_PLAYER_ID_REQUEST: i32 = 0x0005_FF05;
const ONLINE_PLAYER_ID_RESPONSE: i32 = 0x0007_FC05;

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
        _ => WorldGmMessageDispatch::Pending(message),
    }
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
