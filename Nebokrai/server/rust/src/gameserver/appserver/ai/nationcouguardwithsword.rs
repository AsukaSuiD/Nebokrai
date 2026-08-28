//! Достигнутая часть национального окружного охранника с мечом (AI19).
//!
//! Точная пара gameserver.exe + GameServer.pdb и исходный владелец
//! appserver/ai/nationcouguardwithsword.cpp подтверждают отдельный поиск
//! игрока и питомца, выбор цели с учётом минимальной дистанции текущего навыка
//! и преимущество игрока при равном итоговом расстоянии. Проверка сохранённой
//! точки охраны при уже выбранной цели остаётся RAW: владелец этой координаты
//! в достигнутом жизненном цикле Rust пока не подтверждён.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\nationcouguardwithsword.cpp

// ============================================================================
// FUNCTION: CNationCouGuardWithSword::CNationCouGuardWithSword
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\nationcouguardwithsword.cpp:8
// RVA: 0x0020BF60
// ADDRESS: 0060bf60
// PROTOTYPE: undefined __thiscall CNationCouGuardWithSword(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNationCouGuardWithSword::OnSearchEnemy
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: поиск игроков и питомцев подключён к реальному AI19 runtime;
// проверка сохранённой точки охраны при существующей цели остаётся ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\nationcouguardwithsword.cpp:17
// RVA: 0x0020BF80
// ADDRESS: 0060bf80
// PROTOTYPE: int __thiscall OnSearchEnemy(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::shape::ShapeIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct NationCountryGuardTarget {
    pub(crate) identity: ShapeIdentity,
    pub(crate) distance: i32,
}

fn consider_target(
    selected: Option<NationCountryGuardTarget>,
    candidate: NationCountryGuardTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<NationCountryGuardTarget> {
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

pub(crate) fn consider_nation_country_guard_player(
    selected: Option<NationCountryGuardTarget>,
    candidate: NationCountryGuardTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
    region_country: u8,
    guard_country: u32,
    player_country: u8,
    player_is_badman: bool,
) -> Option<NationCountryGuardTarget> {
    if (region_country != 0 && player_country == region_country)
        || (!player_is_badman && u32::from(player_country) == guard_country)
    {
        return selected;
    }
    consider_target(
        selected,
        candidate,
        guard_range,
        minimum_skill_distance,
    )
}

pub(crate) fn consider_nation_country_guard_pet(
    selected: Option<NationCountryGuardTarget>,
    candidate: NationCountryGuardTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
    region_country: u8,
    live_master_country: Option<u8>,
) -> Option<NationCountryGuardTarget> {
    if region_country != 0 && live_master_country == Some(region_country) {
        return selected;
    }
    consider_target(
        selected,
        candidate,
        guard_range,
        minimum_skill_distance,
    )
}

/// Сводит результаты двух исходных поисков. Игрок побеждает питомца при
/// равной дистанции, независимо от правила минимальной дистанции внутри списка.
pub(crate) fn select_nation_country_guard_target(
    player: Option<NationCountryGuardTarget>,
    pet: Option<NationCountryGuardTarget>,
) -> Option<NationCountryGuardTarget> {
    match (player, pet) {
        (Some(player), Some(pet)) if pet.distance < player.distance => Some(pet),
        (Some(player), _) => Some(player),
        (None, pet) => pet,
    }
}
