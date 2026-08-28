//! Достигнутая часть ИИ простого лучника `CStupidArcher`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный владелец `appserver/ai/stupidarcher.cpp` подтверждают ближайшую
//! живую цель по `RealDistance`, замену при равной дистанции и порядок игроков
//! перед питомцами. Если цель ближе минимальной дистанции навыка, владелец
//! делает ровно один `random(8)`, отходит на соседнюю клетку и выдерживает
//! исходную задержку `CBaseAI::MoveTo`; иначе цель передаётся существующему
//! навыку.
//!
//! `OnFighting` ниже остаётся RAW: точный момент проверки завершения навыка и
//! постановки `ASA_SEARCH_ENEMY` ещё не отделён от общего жизненного цикла
//! навыка.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\stupidarcher.cpp

// FUNCTION: CStupidArcher::OnFighting
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\stupidarcher.cpp:40
// RVA: 0x0020F6F0
// ADDRESS: 0060f6f0
// PROTOTYPE: int __thiscall OnFighting(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::shape::ShapeIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StupidArcherTarget {
    pub(crate) identity: ShapeIdentity,
    pub(crate) distance: i32,
}

/// Выбирает ближайшую живую цель внутри дальности охраны; равная дистанция
/// заменяет предыдущую запись, поэтому более поздний питомец может вытеснить
/// игрока. Случайный отход при слишком близкой цели выполняет вызывающий владелец.
pub(crate) fn consider_stupid_archer_target(
    selected: Option<StupidArcherTarget>,
    candidate: StupidArcherTarget,
    guard_range: i32,
) -> Option<StupidArcherTarget> {
    if candidate.distance > guard_range {
        return selected;
    }
    match selected {
        Some(current) if current.distance < candidate.distance => Some(current),
        _ => Some(candidate),
    }
}

/// Длительность одного шага совпадает с общим `CBaseAI::MoveTo`: диагональ
/// длиннее осевого шага, затем добавляется исходное время остановочного кадра.
pub(crate) fn stupid_archer_retreat_delay_ms(
    direction: i32,
    speed: f32,
    stop_frame: u32,
) -> u32 {
    let distance_units = if direction % 2 == 0 {
        1_000_000.0
    } else {
        1_414_000.0
    };
    if speed > 0.0 {
        (distance_units * 0.68 / speed + stop_frame as f32)
            .round()
            .max(0.0) as u32
    } else {
        0
    }
}
