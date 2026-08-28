//! Достигнутая часть окружного охранника с мечом (AI15).
//!
//! Точная пара gameserver.exe + GameServer.pdb и исходный владелец
//! appserver/ai/vilcouguardwithsword.cpp подтверждают фильтрацию страны,
//! отдельный упорядоченный поиск игроков и питомцев и необычное предпочтение
//! целей не ближе минимальной дистанции навыка. Поиск повозок сохранён как RAW:
//! его реальный вызов из достигнутого OnSearch пока не подтверждён.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\vilcouguardwithsword.cpp

// ============================================================================
// FUNCTION: CVilCouGuardWithSword::SearchEnemyCarriage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\vilcouguardwithsword.cpp:203
// RVA: 0x0020D6C0
// ADDRESS: 0060d6c0
// PROTOTYPE: CMoveShape * __thiscall SearchEnemyCarriage(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer

use crate::gameserver::appserver::shape::{ShapeAreaCoordinates, ShapeIdentity, ShapeView};
use crate::gameserver::appserver::skills::baseattack::real_distance;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryGuardState {
    station: Option<ShapeAreaCoordinates>,
}
impl CountryGuardState {
    pub(crate) fn record_station(&mut self, owner: ShapeView) {
        self.station.get_or_insert(ShapeAreaCoordinates {
            x: owner.tile_x,
            y: owner.tile_y,
        });
    }

    pub(crate) fn left_chase_range(&self, owner: ShapeView, chase_range: i32) -> bool {
        self.station.is_some_and(|station| {
            real_distance(owner.tile_x, owner.tile_y, station.x, station.y) > chase_range
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryGuardTarget {
    pub(crate) identity: ShapeIdentity,
    pub(crate) distance: i32,
}

/// Сохраняет исходное правило выбора внутри отдельного списка целей: сначала
/// ищется ближайняя цель не ближе минимальной дистанции навыка, а если все
/// цели ближе неё, остаётся последняя запись списка.
pub(crate) fn consider_country_guard_target(
    selected: Option<CountryGuardTarget>,
    candidate: CountryGuardTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<CountryGuardTarget> {
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

pub(crate) fn consider_village_country_guard_player(
    selected: Option<CountryGuardTarget>,
    candidate: CountryGuardTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
    region_country: u8,
    player_country: u8,
) -> Option<CountryGuardTarget> {
    if region_country != 0 && player_country == region_country {
        return selected;
    }
    consider_country_guard_target(
        selected,
        candidate,
        guard_range,
        minimum_skill_distance,
    )
}

pub(crate) fn consider_village_country_guard_pet(
    selected: Option<CountryGuardTarget>,
    candidate: CountryGuardTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
    region_country: u8,
    live_master_country: Option<u8>,
) -> Option<CountryGuardTarget> {
    if region_country != 0 && live_master_country == Some(region_country) {
        return selected;
    }
    consider_country_guard_target(
        selected,
        candidate,
        guard_range,
        minimum_skill_distance,
    )
}

/// Сводит отдельные результаты поиска игроков и питомцев. Первый список
/// побеждает при равной дистанции.
pub(crate) fn select_country_guard_target(
    player: Option<CountryGuardTarget>,
    pet: Option<CountryGuardTarget>,
) -> Option<CountryGuardTarget> {
    match (player, pet) {
        (Some(player), Some(pet)) if pet.distance < player.distance => Some(pet),
        (Some(player), _) => Some(player),
        (None, pet) => pet,
    }
}
