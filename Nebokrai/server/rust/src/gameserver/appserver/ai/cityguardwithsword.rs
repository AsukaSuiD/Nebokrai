//! Достигнутая часть городского охранника с мечом (AI10).
//!
//! Точная пара gameserver.exe + GameServer.pdb и исходный владелец
//! appserver/ai/cityguardwithsword.cpp подтверждают точку поста, фильтры игроков
//! и питомцев по `faction_id`/`union_id`, правило минимальной дистанции и особую
//! ветвь `Tracing`: шаг назад, `ForceMove` около далёкой цели и сброс за
//! `chase_range`. Городской поиск также включает вражеские повозки `603`,
//! исключая повозки членов faction/union владельца города. AI10 и производные
//! окружные AI15/AI19 наследуют
//! `OnMoving` с отдельным `ASA_SEARCH_ENEMY` и эту ветвь преследования, меняя
//! только selector. `OnIdle` один раз фиксирует пост и продолжает через общий
//! idle FIFO. `OnLoseTarget` возвращает владельца к посту, выбирая случайную
//! соседнюю клетку для заблокированной точки, и лишь затем ставит поиск.
//! Сохранённое RAW-тело поиска повозок остаётся локальным доказательством
//! порядка и фильтров достигнутого selector-а.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\cityguardwithsword.cpp

// ============================================================================
// FUNCTION: CCityGuardWithSword::SearchEnemyGuildCarriage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\cityguardwithsword.cpp:218
// RVA: 0x0020E780
// ADDRESS: 0060e780
// PROTOTYPE: CMoveShape * __thiscall SearchEnemyGuildCarriage(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer

use super::guardtarget::{
    GuardDistanceTarget, consider_guard_distance_target, select_guard_target_groups,
};
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity, ShapeView};
use crate::gameserver::appserver::skills::baseattack::real_distance;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::public::guid::CGuid;
use crate::public::tools::get_line_direction;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

fn belongs_to_city_owner(
    faction_id: i32,
    union_id: i32,
    owner_faction_id: i32,
    owner_union_id: i32,
) -> bool {
    (faction_id != 0 && faction_id == owner_faction_id)
        || (union_id != 0 && union_id == owner_union_id)
}

/// Выбирает цель AI10/AI11 отдельными исходными проходами игроков, питомцев и
/// повозок. Совпадение ненулевой фракции либо союза с владельцем города
/// исключает живого игрока и принадлежащих ему существ; owner без разрешимого
/// локального игрока остаётся допустимой целью.
pub(crate) fn select_city_guard_enemy(
    game: &CGame,
    region: &CServerRegion,
    owner: ShapeView,
    area_index: usize,
    guard_range: i32,
    minimum_skill_distance: i32,
) -> Option<GuardDistanceTarget> {
    let owner_faction_id = region.owned_city_faction();
    let owner_union_id = region.owned_city_union();
    let mut selected_player = None;
    for player_id in region.player_ids_around_area(area_index) {
        let Some(player) = game.find_player(player_id) else {
            continue;
        };
        if player.server_region_id() != Some(region.id)
            || player.is_dead()
            || belongs_to_city_owner(
                player.faction_id(),
                player.union_id(),
                owner_faction_id,
                owner_union_id,
            )
        {
            continue;
        }
        let Some(candidate) = player.shape_view() else {
            continue;
        };
        selected_player = consider_guard_distance_target(
            selected_player,
            GuardDistanceTarget {
                identity: candidate.identity,
                distance: real_distance(
                    owner.tile_x,
                    owner.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
            },
            guard_range,
            minimum_skill_distance,
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
        let protected_by_owner = (master.master_type == PLAYER_TYPE)
            .then(|| game.find_player(master.master_id))
            .flatten()
            .is_some_and(|player| {
                belongs_to_city_owner(
                    player.faction_id(),
                    player.union_id(),
                    owner_faction_id,
                    owner_union_id,
                )
            });
        if protected_by_owner {
            continue;
        }
        selected_pet = consider_guard_distance_target(
            selected_pet,
            GuardDistanceTarget {
                identity: candidate.identity,
                distance: real_distance(
                    owner.tile_x,
                    owner.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
            },
            guard_range,
            minimum_skill_distance,
        );
    }
    let mut selected_carriage = None;
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
        let protected_by_owner = (master.master_type == PLAYER_TYPE)
            .then(|| game.find_player(master.master_id))
            .flatten()
            .is_some_and(|player| {
                belongs_to_city_owner(
                    player.faction_id(),
                    player.union_id(),
                    owner_faction_id,
                    owner_union_id,
                )
            });
        if protected_by_owner {
            continue;
        }
        selected_carriage = consider_guard_distance_target(
            selected_carriage,
            GuardDistanceTarget {
                identity: candidate.identity,
                distance: real_distance(
                    owner.tile_x,
                    owner.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
            },
            guard_range,
            minimum_skill_distance,
        );
    }
    select_guard_target_groups(
        select_guard_target_groups(selected_player, selected_pet),
        selected_carriage,
    )
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CitySwordTraceOutcome {
    Ready,
    Handled,
}

/// Выполняет унаследованный `OnLoseTarget` AI10/AI15/AI19 перед тем, как
/// окружающий `OnSchedule` поставит `SearchEnemy`: цель очищается, владелец
/// возвращается к сохранённому посту, а заблокированная клетка заменяется одним
/// `GetRandomPosInRange` на квадрате 3×3.
pub(crate) fn lose_guard_sword_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    runtime: &mut Runtime,
) {
    let station = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        monster.clear_ai_target();
        monster.guard_station_ai_mut()?.station()
    });
    if let Some(station) = station {
        let (destination_x, destination_y) = if region
            .region
            .get_block(station.x, station.y)
            .unwrap_or(2)
            != 0
        {
            region
                .region
                .get_random_pos_in_range(
                    station.x.wrapping_sub(1),
                    station.y.wrapping_sub(1),
                    3,
                    3,
                    runtime,
                )
                .map(|position| (position.x, position.y))
                .unwrap_or((station.x, station.y))
        } else {
            (station.x, station.y)
        };
        let _ = game.force_move_owned_shape(
            region,
            ShapeIdentity {
                object_type: MONSTER_TYPE,
                id: monster_id,
                ex_id: CGuid::GUID_INVALID,
            },
            destination_x,
            destination_y,
            0,
        );
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.begin_active_ai_search_enemy(runtime.now_milliseconds());
    }
}

/// Выполняет общую ветвь `Tracing` AI10 и производных AI15/AI19. Слишком
/// близкая цель вызывает один шаг назад, далёкая в пределах преследования —
/// исходный `ForceMove` в случайную клетку вокруг неё, а выход за
/// `chase_range` сбрасывает цель.
#[allow(clippy::too_many_arguments, reason = "граница сохраняет отдельные пределы навыка и преследования")]
pub(crate) fn trace_city_sword_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    owner: ShapeView,
    target_x: i32,
    target_y: i32,
    minimum_distance: i32,
    maximum_distance: i32,
    chase_range: i32,
    runtime: &mut Runtime,
) -> CitySwordTraceOutcome {
    let distance = real_distance(owner.tile_x, owner.tile_y, target_x, target_y);
    if minimum_distance <= distance && distance <= maximum_distance {
        return CitySwordTraceOutcome::Ready;
    }
    if distance > chase_range {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return CitySwordTraceOutcome::Handled;
    }
    if distance > maximum_distance {
        if let Ok(destination) = region.region.get_random_pos_in_range(
            target_x.wrapping_sub(1),
            target_y.wrapping_sub(1),
            3,
            3,
            runtime,
        ) {
            let _ = game.force_move_owned_shape(
                region,
                ShapeIdentity {
                    object_type: MONSTER_TYPE,
                    id: monster_id,
                    ex_id: CGuid::GUID_INVALID,
                },
                destination.x,
                destination.y,
                0,
            );
        }
        return CitySwordTraceOutcome::Handled;
    }

    let direction = get_line_direction(target_x, target_y, owner.tile_x, owner.tile_y);
    let origin = ShapeAreaCoordinates {
        x: owner.tile_x,
        y: owner.tile_y,
    };
    if let Ok(destination) = CShape::get_direction_position(direction, origin) {
        let figure = region.find_monster_by_id(monster_id).and_then(|monster| {
            let property =
                game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            Some(crate::gameserver::appserver::monster::CMonster::figure(property))
        });
        if let Some(figure) = figure {
            let _ = game.move_owned_monster_step(
                region,
                monster_id,
                destination.x,
                destination.y,
                figure,
            );
        }
    }
    CitySwordTraceOutcome::Handled
}
