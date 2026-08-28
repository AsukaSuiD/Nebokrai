//! Достигнутая часть ИИ неподвижного лучника `CFixedPositionArcher`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный владелец `appserver/ai/fixedpositionarcher.cpp` подтверждают
//! порядок девяти соседних областей, игроков перед питомцами, дальность охраны
//! и необычное предпочтение цели вне минимальной дистанции текущего навыка.
//! Реальный путь `monsterbaseattack` выполняет выбор навыка и поиск цели, а
//! достигнутый владелец навыка не использует произвольный порядок хранилища
//! сущностей.
//!
//! Полные очереди `OnChangeSkill`, `OnFighting` и `OnIdle` ниже остаются RAW;
//! достигнутые части их выбора и сна выполняются общим циклом монстра.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\fixedpositionarcher.cpp

// FUNCTION: CFixedPositionArcher::OnChangeSkill
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: общий цикл монстра назначает текущий навык до поиска цели;
// точные события `CHANGE_SKILL/STAND` остаются ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\fixedpositionarcher.cpp:26
// RVA: 0x0020F9F0
// ADDRESS: 0060f9f0
// PROTOTYPE: int __thiscall OnChangeSkill(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFixedPositionArcher::OnFighting
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\fixedpositionarcher.cpp:184
// RVA: 0x0020FA70
// ADDRESS: 0060fa70
// PROTOTYPE: int __thiscall OnFighting(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFixedPositionArcher::OnIdle
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `execute_owned_monster_base_attack` сохраняет проверку игроков
// в девяти соседних областях и спящий переход; точная очередь событий остаётся ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\fixedpositionarcher.cpp:116
// RVA: 0x0020FAA0
// ADDRESS: 0060faa0
// PROTOTYPE: void __thiscall OnIdle(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::shape::ShapeIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FixedArcherTarget {
    pub(crate) identity: ShapeIdentity,
    pub(crate) distance: i32,
}

/// Повторяет необычное правило `OnSearchEnemy`: выбирается ближайшая цель не
/// ближе минимальной дистанции навыка; если уже выбранная цель слишком близка,
/// следующая допустимая по дальности охраны запись заменяет её даже при большей
/// дистанции. Поэтому порядок игроков, затем питомцев и порядок внутри индексов
/// являются частью результата.
pub(crate) fn consider_fixed_archer_target(
    selected: Option<FixedArcherTarget>,
    candidate: FixedArcherTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<FixedArcherTarget> {
    if candidate.distance > guard_range {
        return selected;
    }
    let Some(current) = selected else {
        return Some(candidate);
    };
    if current.distance <= candidate.distance {
        if current.distance < minimum_skill_distance {
            Some(candidate)
        } else {
            Some(current)
        }
    } else if candidate.distance < minimum_skill_distance {
        Some(current)
    } else {
        Some(candidate)
    }
}
