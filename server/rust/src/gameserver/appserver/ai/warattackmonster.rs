//! Поиск цели атакующего монстра войны стран `WarAttackMonster`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! подтверждает поиск ближайшего живого противника по лагерю `CountryWarSys`.
//! Реальные типы ИИ `17/18` сохраняют порядок игроков перед питомцами и замену
//! предыдущей цели при равной `RealDistance`; номер типа задаёт собственный
//! лагерь как `ai - 17`.

use crate::gameserver::appserver::ai::gladiator::{
    GladiatorTarget, consider_gladiator_target,
};
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::gameserver::game::CGame;

/// Отбрасывает собственный лагерь и сохраняет общий выбор ближайшей цели.
pub(crate) fn consider_country_war_target(
    selected: Option<GladiatorTarget>,
    candidate: GladiatorTarget,
    guard_range: i32,
    own_camp: i32,
    candidate_camp: i32,
) -> Option<GladiatorTarget> {
    if candidate_camp == own_camp {
        selected
    } else {
        consider_gladiator_target(selected, candidate, guard_range)
    }
}

/// Выполняет достигнутый поиск AI17/AI18, сохраняя отдельные проходы игроков
/// и питомцев и разрешая их лагерь через канонический `CountryWarSys`.
pub(crate) fn select_country_war_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    ai_type: u32,
    guard_range: i32,
) -> Option<ShapeIdentity> {
    let own_camp = ai_type as i32 - 17;
    let mut selected = None;
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
        selected = consider_country_war_target(
            selected,
            GladiatorTarget {
                identity: candidate.identity,
                distance: owner.real_distance(Some(candidate)),
            },
            guard_range,
            own_camp,
            game.country_war_sys()
                .get_war_camp(i32::from(player.country())),
        );
    }
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some((candidate, country)) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !CMoveShape::is_died(pet.hit_points()))
            .and_then(|pet| {
                let property =
                    game.find_monster_property_by_origin_name(pet.base_property_key()?)?;
                Some((pet.shape_view(property)?, pet.master_info().master_country_id))
            })
        else {
            continue;
        };
        selected = consider_country_war_target(
            selected,
            GladiatorTarget {
                identity: candidate.identity,
                distance: owner.real_distance(Some(candidate)),
            },
            guard_range,
            own_camp,
            game.country_war_sys().get_war_camp(country),
        );
    }
    selected.map(|selected| selected.identity)
}
