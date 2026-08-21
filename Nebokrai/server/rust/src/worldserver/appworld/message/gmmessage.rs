//! WorldServer dispatcher-owner `OnGMMessage`.
//!
//! Статус `IMPLEMENTED_PARTIAL`: exact ветвь `0x5FF01` RVA
//! `0x000AB3C8..0x000AB425` материализует запрос online-count. Общий owner до
//! switch сначала читает request/player ID; ветвь затем читает script ID,
//! берёт 32-битное число `m_lOnlinePlayer`, строит
//! `0x7FC01 + request_id + online_count + script_id` и отправляет ответ в
//! исходный socket. Порядок чтения, signed wire-биты и `SendToSocket`
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
}

pub(crate) enum WorldGmMessageDispatch {
    Handled(WorldGmMessageOutcome),
    Pending(CMessage),
}

/// Исполняет достигнутую exact ветвь `0x5FF01` частичного GM-owner-а.
pub(crate) fn on_gm_message(game: &CGame, mut message: CMessage) -> WorldGmMessageDispatch {
    let decoded_request_id = message.base_mut().get_long();
    let request_id = decoded_request_id.unwrap_or(0);
    if message.message_type() != ONLINE_PLAYER_COUNT_REQUEST {
        return WorldGmMessageDispatch::Pending(message);
    }

    let decoded_script_id = message.base_mut().get_long();
    let script_id = decoded_script_id.unwrap_or(0);
    let payload_complete = [decoded_request_id.is_some(), decoded_script_id.is_some()];
    let count = game.online_player_count();
    let Ok(online_count_bits) = u32::try_from(count) else {
        return WorldGmMessageDispatch::Handled(WorldGmMessageOutcome::OnlinePlayerCount(
            WorldGmOnlinePlayerCountOutcome::CountOutsideLegacyRange {
                request_id,
                script_id,
                payload_complete,
                count,
            },
        ));
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
