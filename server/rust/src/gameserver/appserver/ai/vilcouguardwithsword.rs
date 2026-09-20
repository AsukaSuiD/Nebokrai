//! Достигнутая часть окружного охранника с мечом (AI12).
//!
//! Точная пара gameserver.exe + GameServer.pdb и исходный владелец
//! appserver/ai/vilcouguardwithsword.cpp подтверждают фильтрацию страны,
//! отдельный упорядоченный поиск игроков и питомцев и необычное предпочтение
//! целей не ближе минимальной дистанции навыка. Унаследованные `OnIdle` и
//! `OnSearchEnemy` проходят через общий FIFO городского мечника, но вызывают
//! окружной selector. Третий проход выбирает повозки, исключая живого хозяина
//! страны региона; сохранённое RAW-тело фиксирует его порядок и фильтры.

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

use super::guardtarget::{
    GuardDistanceTarget, consider_guard_distance_target, select_guard_target_groups,
};
use crate::gameserver::appserver::shape::ShapeView;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::gameserver::game::CGame;

const PLAYER_TYPE: i32 = 400;

/// Сохраняет исходное правило выбора внутри отдельного списка целей: сначала
/// ищется ближайняя цель не ближе минимальной дистанции навыка, а если все
/// цели ближе неё, остаётся последняя запись списка.
pub(crate) fn consider_country_guard_target(
    selected: Option<GuardDistanceTarget>,
    candidate: GuardDistanceTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<GuardDistanceTarget> {
    consider_guard_distance_target(selected, candidate, guard_range, minimum_skill_distance)
}

pub(crate) fn consider_village_country_guard_player(
    selected: Option<GuardDistanceTarget>,
    candidate: GuardDistanceTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
    region_country: u8,
    player_country: u8,
) -> Option<GuardDistanceTarget> {
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
    selected: Option<GuardDistanceTarget>,
    candidate: GuardDistanceTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
    region_country: u8,
    live_master_country: Option<u8>,
) -> Option<GuardDistanceTarget> {
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
    player: Option<GuardDistanceTarget>,
    pet: Option<GuardDistanceTarget>,
) -> Option<GuardDistanceTarget> {
    select_guard_target_groups(player, pet)
}

pub(crate) fn select_village_country_guard_carriage(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<GuardDistanceTarget> {
    let mut selected = None;
    for carriage_id in region.carriage_ids_around_area(area_index) {
        let Some((candidate, master)) = region
            .find_monster_by_id(carriage_id)
            .filter(|carriage| !CMoveShape::is_died(carriage.hit_points()))
            .and_then(|carriage| {
                let property =
                    game.find_monster_property_by_origin_name(carriage.base_property_key()?)?;
                if !carriage.is_carriage(property) {
                    return None;
                }
                Some((carriage.shape_view(property)?, carriage.master_info()))
            })
        else {
            continue;
        };
        let live_master_country = (master.master_type == PLAYER_TYPE)
            .then(|| game.find_player(master.master_id).map(|player| player.country()))
            .flatten();
        selected = consider_village_country_guard_pet(
            selected,
            GuardDistanceTarget {
                identity: candidate.identity,
                distance: owner.real_distance(Some(candidate)),
            },
            guard_range,
            minimum_skill_distance,
            region.country,
            live_master_country,
        );
    }
    selected
}

pub(crate) fn select_village_country_guard_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<GuardDistanceTarget> {
    let mut selected_player = None;
    for player_id in region.player_ids_around_area(area_index) {
        let Some(player) = game.find_player(player_id) else {
            continue;
        };
        if player.server_region_id() != Some(region.id) || player.is_dead() {
            continue;
        }
        let Some(candidate) = player.shape_view() else {
            continue;
        };
        selected_player = consider_village_country_guard_player(
            selected_player,
            GuardDistanceTarget {
                identity: candidate.identity,
                distance: owner.real_distance(Some(candidate)),
            },
            guard_range,
            minimum_skill_distance,
            region.country,
            player.country(),
        );
    }
    let mut selected_pet = None;
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some((candidate, master)) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !CMoveShape::is_died(pet.hit_points()))
            .and_then(|pet| {
                let property =
                    game.find_monster_property_by_origin_name(pet.base_property_key()?)?;
                Some((pet.shape_view(property)?, pet.master_info()))
            })
        else {
            continue;
        };
        let live_master_country = (master.master_type == PLAYER_TYPE)
            .then(|| game.find_player(master.master_id).map(|player| player.country()))
            .flatten();
        selected_pet = consider_village_country_guard_pet(
            selected_pet,
            GuardDistanceTarget {
                identity: candidate.identity,
                distance: owner.real_distance(Some(candidate)),
            },
            guard_range,
            minimum_skill_distance,
            region.country,
            live_master_country,
        );
    }
    let selected_carriage = select_village_country_guard_carriage(
        game,
        region,
        owner,
        area_index,
        guard_range,
        minimum_skill_distance,
    );
    select_guard_target_groups(
        select_country_guard_target(selected_player, selected_pet),
        selected_carriage,
    )
}
