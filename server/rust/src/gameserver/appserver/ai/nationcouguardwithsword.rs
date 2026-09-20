//! Национальный окружной охранник с мечом (AI16).
//!
//! Источник: точная пара gameserver.exe + GameServer.pdb, исходный владелец
//! appserver/ai/nationcouguardwithsword.cpp. Общий владелец окружной охраны
//! сохраняет точку поста, дальность преследования и правило минимальной
//! дистанции навыка; этот наследник добавляет фильтр страны монстра и статус
//! преступника. Поиск игроков выполняется раньше поиска питомцев, а игрок
//! побеждает при равном итоговом расстоянии. Унаследованные `OnIdle` и
//! `OnSearchEnemy` используют общий FIFO мечника и этот национальный selector;
//! повозки проходят унаследованный country-фильтр окружной охраны.

use super::guardtarget::{GuardDistanceTarget, select_guard_target_groups};
use super::vilcouguardwithsword::{
    consider_country_guard_target, consider_village_country_guard_pet,
    select_country_guard_target, select_village_country_guard_carriage,
};
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeView;
use crate::gameserver::gameserver::game::CGame;

const PLAYER_TYPE: i32 = 400;

pub(crate) fn consider_nation_country_guard_player(
    selected: Option<GuardDistanceTarget>,
    candidate: GuardDistanceTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
    region_country: u8,
    guard_country: u32,
    player_country: u8,
    player_is_badman: bool,
) -> Option<GuardDistanceTarget> {
    if (region_country != 0 && player_country == region_country)
        || (!player_is_badman && u32::from(player_country) == guard_country)
    {
        return selected;
    }
    consider_country_guard_target(
        selected,
        candidate,
        guard_range,
        minimum_skill_distance,
    )
}

/// Выполняет достигнутый поиск AI16: отдельный проход игроков применяет
/// национальный фильтр, а питомцы сохраняют правило живого владельца общего
/// окружного охранника. Игрок выигрывает при равной итоговой дистанции.
pub(crate) fn select_nation_country_guard_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
    minimum_skill_distance: i32,
    guard_country: u32,
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
        selected_player = consider_nation_country_guard_player(
            selected_player,
            GuardDistanceTarget {
                identity: candidate.identity,
                distance: owner.real_distance(Some(candidate)),
            },
            guard_range,
            minimum_skill_distance,
            region.country,
            guard_country,
            player.country(),
            player.is_badman(game.globe_setup().pk_count_per_kill()),
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
