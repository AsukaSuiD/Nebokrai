//! WorldServer dispatcher-owner `OnOtherMessage`.
//!
//! Весь dispatcher RVA `0x000AC680` остаётся `UNKNOWN` (исследовательский декомпилят хранится локально), кроме локального
//! honor-reset `0x5FD0C` со статусом `IMPLEMENTED`. Ветка читает один Windows
//! `long`, получает текущий `CGame` и вызывает `ResetHonorElimilateInfo`.
//! Недостаточный payload сохраняет старое поведение numeric getter-а: значение
//! становится нулём без сдвига cursor; отчёт отдельно фиксирует неполноту.
//!
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`;
//! исходный owner
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\othermessage.cpp:43`.
//! Linux C++ подтверждает практическую границу dispatcher-а, но добавленную там
//! проверку synthetic owner-а Rust не переносит: exact EXE её не выполняет.

use crate::nets::networld::message::CMessage;
use crate::worldserver::worldserver::game::CGame;

const HONOR_ELIMINATE_RESET: i32 = 0x0005_FD0C;

/// Наблюдаемый итог одной уже восстановленной ветки `OnOtherMessage`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WorldHonorEliminateReset {
    pub(crate) rank_mask: u32,
    pub(crate) payload_complete: bool,
    pub(crate) legacy_result: bool,
}

/// Узкая диспетчеризация уже выбранного other-owner-а.
pub(crate) enum WorldOtherMessageDispatch {
    Handled(WorldHonorEliminateReset),
    Pending(CMessage),
}

/// Исполняет только доказанный локальный сброс `0x5FD0C`.
pub(crate) fn on_other_message(
    game: &mut CGame,
    mut message: CMessage,
) -> WorldOtherMessageDispatch {
    if message.message_type() != HONOR_ELIMINATE_RESET {
        return WorldOtherMessageDispatch::Pending(message);
    }

    let decoded = message.base_mut().get_long();
    let rank_mask = decoded.unwrap_or(0) as u32;
    WorldOtherMessageDispatch::Handled(WorldHonorEliminateReset {
        rank_mask,
        payload_complete: decoded.is_some(),
        legacy_result: game.reset_honor_eliminate_info(rank_mask),
    })
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\othermessage.cpp

// ============================================================================
// FUNCTION: OnOtherMessage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\message\othermessage.cpp:43
// RVA: 0x000AC680
// ADDRESS: 004ac680
// PROTOTYPE: void __cdecl OnOtherMessage(CMessage * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
