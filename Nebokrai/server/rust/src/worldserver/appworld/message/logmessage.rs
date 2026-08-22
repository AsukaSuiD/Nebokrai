//! WorldServer dispatcher-owner `OnLogMessage`.
//!
//! Корпус остаётся `UNKNOWN` (исследовательский декомпилят хранится локально), кроме restore-role leaf `0x4FB03` со
//! статусом `IMPLEMENTED`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`; исходный owner
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\logmessage.cpp:30`.
//! Exact `0x004B1692..0x004B171F` читает account через `GetStr(..., 0x14)`,
//! затем signed player ID, удаляет первое совпадение из live deletion-list,
//! добавляет уникальный ID в хвост restore-list и посылает в текущий
//! LoginServer `0x1FF04 + char(0x15) + player_id + account\0` без priority.
//! Добавленные Linux-донором peer/ownership/DB-preflight gates в EXE
//! отсутствуют и не перенесены. `VecDeque` заменяет только старые list-nodes,
//! а существующая client FIFO — WinSock transport без изменения wire/order.
//!
//! Декомпилятор: Ghidra 12.1.2. Сырой C++ ниже сохранён как локальная
//! документация, а не как Rust-реализация.

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::worldserver::game::CGame;

const RESTORE_ROLE_REQUEST: i32 = 0x0004_FB03;
const RESTORE_ROLE_RESPONSE: i32 = 0x0001_FF04;
const RESTORE_ROLE_STATUS: i8 = 0x15;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldRestoreRoleOutcome {
    pub(crate) account: Vec<u8>,
    pub(crate) player_id: u32,
    pub(crate) player_id_complete: bool,
    pub(crate) response_type: i32,
    pub(crate) status: i8,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum WorldLogMessageOutcome {
    RestoreRole(WorldRestoreRoleOutcome),
}

pub(crate) enum WorldLogMessageDispatch {
    Handled(WorldLogMessageOutcome),
    Pending(CMessage),
}

pub(crate) fn on_log_message(
    game: &mut CGame,
    mut message: CMessage,
) -> WorldLogMessageDispatch {
    if message.message_type() != RESTORE_ROLE_REQUEST {
        return WorldLogMessageDispatch::Pending(message);
    }

    let account = message
        .base_mut()
        .get_str_bytes(0x14)
        .expect("literal 0x14 исключает zero-capacity GetStr");
    let decoded_player_id = message.base_mut().get_long();
    let player_id = decoded_player_id.unwrap_or(0) as u32;

    game.delete_deletion_player(player_id);
    game.append_restore_player(player_id);

    let mut response = CMessage::new(RESTORE_ROLE_RESPONSE);
    response.base_mut().add_char(RESTORE_ROLE_STATUS);
    response.base_mut().add_ulong(player_id);
    response.base_mut().add(&account);
    response.base_mut().add_char(0);
    let wire = response.as_wire_bytes().to_vec();
    let delivery = response.send(
        game.current_login_client().map(|client| client.send_queue()),
        false,
    );
    WorldLogMessageDispatch::Handled(WorldLogMessageOutcome::RestoreRole(
        WorldRestoreRoleOutcome {
            account,
            player_id,
            player_id_complete: decoded_player_id.is_some(),
            response_type: RESTORE_ROLE_RESPONSE,
            status: RESTORE_ROLE_STATUS,
            wire,
            delivery,
        },
    ))
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\logmessage.cpp

// ============================================================================
// FUNCTION: OnLogMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\logmessage.cpp:30
// RVA: 0x000B0D10
// ADDRESS: 004b0d10
// PROTOTYPE: void __cdecl OnLogMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: WorldServer
