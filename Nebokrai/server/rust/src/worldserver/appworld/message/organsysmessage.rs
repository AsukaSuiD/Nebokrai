//! Статус корпуса: `IMPLEMENTED_PARTIAL` для общего session-result dispatch
//! opcode `0x60120/0x60122/0x60124`; остальной owner — `UNKNOWN` (исследовательский декомпилят хранится локально).
//! Декомпилятор: Ghidra 12.1.2
//! Полный декомпилят хранится локально и не входит в распространяемый код.
//!
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! `OnOrgasysMessage` RVA `0x000A6110`. Exact диапазон
//! `0x004A74C6..0x004A7509` исправляет повреждённый RAW: общий branch читает
//! `GetLONG64`, `GetLong`, именно `GetChar`, затем `GetLong`, то есть
//! `(session ID, cookie.second, result byte, cookie.first)`. `movzx` расширяет
//! result как unsigned byte до 32 бит. Затем вызывается уже восстановленный
//! `CNetSessionManager::OnSyncCallbackResult(session, first, second, &result)`;
//! manager проверяет cookie, синхронно вызывает endpoint и удаляет session.
//!
//! Legacy getters при нехватке возвращают ноль и не двигают cursor; Rust
//! сохраняет это через `unwrap_or(0)`, а не добавляет отсутствовавший общий
//! reject. Дополнительный хвост owner не проверял. Проверки exact payload и
//! GameServer ownership из старого Linux-донора к этой ветке EXE не относятся
//! и здесь не переносятся. `CMessage`/`CBaseMessage` и session manager уже
//! материализованы; функция ниже добавляет только конкретный opcode dispatch.

use crate::nets::networld::message::CMessage;
use crate::public::netsessionmanager::{CNetSessionManager, NetSessionCallbackOutcome};

const SESSION_RESULT_MESSAGE_TYPES: [i32; 3] = [0x60120, 0x60122, 0x60124];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OrganizingSessionResultDispatch {
    NotHandled,
    Delivered {
        message_type: i32,
        session_id: i64,
        cookie_first: i32,
        cookie_second: i32,
        result: i32,
        outcome: NetSessionCallbackOutcome,
    },
}

/// Выполняет общий session-result branch `OnOrgasysMessage`.
pub(crate) fn dispatch_organizing_session_result(
    message: &mut CMessage,
    manager: &CNetSessionManager,
) -> OrganizingSessionResultDispatch {
    let message_type = message.message_type();
    if !SESSION_RESULT_MESSAGE_TYPES.contains(&message_type) {
        return OrganizingSessionResultDispatch::NotHandled;
    }

    let session_id = message.base_mut().get_long64().unwrap_or(0);
    let cookie_second = message.base_mut().get_long().unwrap_or(0);
    let result = message
        .base_mut()
        .get_char()
        .map_or(0, |result| i32::from(result as u8));
    let cookie_first = message.base_mut().get_long().unwrap_or(0);
    let outcome =
        manager.on_sync_callback_result(session_id, cookie_first, cookie_second, &result);
    OrganizingSessionResultDispatch::Delivered {
        message_type,
        session_id,
        cookie_first,
        cookie_second,
        result,
        outcome,
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\organsysmessage.cpp

// ============================================================================
// FUNCTION: OnOrgasysMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\organsysmessage.cpp:30
// RVA: 0x000A6110
// ADDRESS: 004a6110
// PROTOTYPE: void __cdecl OnOrgasysMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004ac1b5
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\organsysmessage.cpp
// RVA: 0x000AC1B5
// ADDRESS: 004ac1b5
// PROTOTYPE: undefined Catch@004ac1b5()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004ac2a9
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\organsysmessage.cpp
// RVA: 0x000AC2A9
// ADDRESS: 004ac2a9
// PROTOTYPE: undefined Catch@004ac2a9()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004ac3c6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\organsysmessage.cpp
// RVA: 0x000AC3C6
// ADDRESS: 004ac3c6
// PROTOTYPE: undefined Catch@004ac3c6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00532c30
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\organsysmessage.cpp
// RVA: 0x00132C30
// ADDRESS: 00532c30
// PROTOTYPE: undefined Unwind@00532c30()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//














// COMPONENT_VARIANT_END: WorldServer
