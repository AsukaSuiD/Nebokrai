//! WorldServer dispatcher-owner `OnPlayerMessage`.
//!
//! Статус корпуса: `IMPLEMENTED`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, функция RVA
//! `0x000AD580`, исходный owner
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\playermessage.cpp:17`.
//! Exact `0x004AD580..0x004AD5E5` подтверждает четыре in-place relay branch:
//! `0x5FC01 -> 0x7FA08`, `0x5FC02 -> 0x7FA09`, `0x5FC03 -> 0x7FA0A`,
//! `0x5FC04 -> 0x7FA0B`. Только первая до broadcast дописывает signed
//! `m_lMapID`; остальные сохраняют payload byte-for-byte. Все четыре вызывают
//! общий `CMessage::SendAll`, не делают `Update`, не читают payload и не
//! проверяют socket/map ownership или хвост. Эти дополнительные проверки и
//! `SendAllCurrentMaps` из старого Linux-донора в EXE отсутствуют и не
//! перенесены. Неизвестный opcode остаётся owned pending без mutation/send.
//!
//! Декомпилятор: Ghidra 12.1.2. Сырой C++ ниже сохранён как локальная
//! документация, а не как Rust-реализация.

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::worldserver::worldserver::game::CGame;

const SET_SKILL_LEVEL: i32 = 0x0005_FC01;
const DELETE_SKILL: i32 = 0x0005_FC02;
const ADD_SKILL: i32 = 0x0005_FC03;
const USE_SKILL: i32 = 0x0005_FC04;

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct WorldPlayerMessageOutcome {
    pub(crate) request_type: i32,
    pub(crate) response_type: i32,
    pub(crate) appended_map_id: Option<i32>,
    pub(crate) wire: Vec<u8>,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

pub(crate) enum WorldPlayerMessageDispatch {
    Handled(WorldPlayerMessageOutcome),
    Pending(CMessage),
}

/// Исполняет весь exact `OnPlayerMessage` owner.
pub(crate) fn on_player_message(
    game: &CGame,
    mut message: CMessage,
) -> WorldPlayerMessageDispatch {
    let request_type = message.message_type();
    let (response_type, append_map_id) = match request_type {
        SET_SKILL_LEVEL => (0x0007_FA08, true),
        DELETE_SKILL => (0x0007_FA09, false),
        ADD_SKILL => (0x0007_FA0A, false),
        USE_SKILL => (0x0007_FA0B, false),
        _ => return WorldPlayerMessageDispatch::Pending(message),
    };

    message.set_message_type(response_type);
    let appended_map_id = append_map_id.then(|| {
        let map_id = message.map_id();
        message.base_mut().add_long(map_id);
        map_id
    });
    let wire = message.as_wire_bytes().to_vec();
    let delivery = message.send_all(game.current_game_server_sender().as_ref());
    WorldPlayerMessageDispatch::Handled(WorldPlayerMessageOutcome {
        request_type,
        response_type,
        appended_map_id,
        wire,
        delivery,
    })
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\playermessage.cpp

// ============================================================================
// FUNCTION: OnPlayerMessage
// STATUS: IMPLEMENTED_OWNER
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\playermessage.cpp:17
// RVA: 0x000AD580
// ADDRESS: 004ad580
// PROTOTYPE: void __cdecl OnPlayerMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004ad742
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\playermessage.cpp
// RVA: 0x000AD742
// ADDRESS: 004ad742
// PROTOTYPE: undefined Catch@004ad742()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004ad843
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\playermessage.cpp
// RVA: 0x000AD843
// ADDRESS: 004ad843
// PROTOTYPE: undefined Catch@004ad843()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004adaa5
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\playermessage.cpp
// RVA: 0x000ADAA5
// ADDRESS: 004adaa5
// PROTOTYPE: undefined Catch@004adaa5()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004adb56
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\playermessage.cpp
// RVA: 0x000ADB56
// ADDRESS: 004adb56
// PROTOTYPE: undefined Catch@004adb56()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Unwind@00532d70
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\playermessage.cpp
// RVA: 0x00132D70
// ADDRESS: 00532d70
// PROTOTYPE: undefined Unwind@00532d70()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00532d90
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\playermessage.cpp
// RVA: 0x00132D90
// ADDRESS: 00532d90
// PROTOTYPE: undefined Unwind@00532d90()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




































































// COMPONENT_VARIANT_END: WorldServer
