//! Достигнутый ИИ слабого существа `CPuninessCreature`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный владелец `appserver/ai/puninesscreature.cpp` подтверждают поиск
//! ближайшего живого игрока или питомца в девяти соседних областях. При равной
//! `RealDistance` побеждает более поздняя запись. Внутри дальности охраны
//! существо делает один шаг от цели без RNG; только за её пределами применяется
//! проверка дальности преследования и возможная потеря цели.
//!
//! `OnSchedule` вызывает `Tracing` только при существующей цели и пустых
//! основных очередях `CBaseAI`; PDB-владелец и таблица виртуальных методов
//! подтверждают эту проверку. Начальная последовательность
//! `ChangeSkill → Move/Stand → SearchEnemy` проходит через тот же FIFO.
//! CGame вызывает этот OnSchedule (0x0060F4B0) до background/passive,
//! направляя существующую цель в Tracing вместо общего Begin атаки.
//! После фаз отдельный вход без цели выполняет OnIdle. Проверка принадлежности
//! исключает приручённого монстра: его текущий владелец CPet, даже при setup AI7.


use super::baseai::one_step_move_delay_ms;
use super::monsterai::queue_monster_idle;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity, ShapeView};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::public::tools::get_line_direction;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct PuninessTarget {
    identity: ShapeIdentity,
    distance: i32,
}

fn consider_target(
    selected: Option<PuninessTarget>,
    candidate: PuninessTarget,
    guard_range: i32,
) -> Option<PuninessTarget> {
    if candidate.distance > guard_range {
        return selected;
    }
    match selected {
        Some(current) if current.distance < candidate.distance => Some(current),
        _ => Some(candidate),
    }
}

fn live_target_view(
    game: &CGame,
    region: &CServerRegion,
    identity: ShapeIdentity,
) -> Option<ShapeView> {
    match identity.object_type {
        PLAYER_TYPE => game.find_player(identity.id).and_then(|player| {
            (player.server_region_id() == Some(region.id) && !player.is_dead())
                .then(|| player.shape_view())
                .flatten()
        }),
        MONSTER_TYPE => region.find_monster_by_id(identity.id).and_then(|monster| {
            if CMoveShape::is_died(monster.hit_points()) {
                return None;
            }
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?;
            monster.shape_view(property)
        }),
        _ => None,
    }
}

/// Исполняет достигнутую вертикаль `OnSearchEnemy → Tracing`: первый такт
/// назначает ближайшую цель в порядке игроков и питомцев, последующие такты
/// отходят от неё по одной клетке внутри дальности охраны и сбрасывают цель за
/// дальностью преследования.
pub(crate) fn execute_owned_puniness_creature<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    runtime: &mut Runtime,
) -> bool {
    if region.find_monster_by_id(monster_id).is_some_and(CMonster::is_tamed) {
        return false;
    }
    let Some((property, source, source_view, stop_frame, target, schedule_idle)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            let stop_frame = monster.stop_frame(&property);
            Some((
                property.clone(),
                monster.move_shape().shape().clone(),
                monster.shape_view(&property)?,
                stop_frame,
                monster.ai_target(),
                monster.primary_ai_queues_idle(),
            ))
        })
    else {
        return false;
    };
    if property.ai != 7 {
        return false;
    }
    if let Some(target) = target {
        if !schedule_idle {
            return true;
        }
        let Some(target_view) = live_target_view(game, region, target) else {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.lose_ai_target_and_search(runtime.now_milliseconds());
            }
            return true;
        };
        let distance = source_view.real_distance(Some(target_view));
        if distance > property.guard_range as i32 {
            if distance > property.chase_range as i32
                && let Some(monster) = region.find_monster_by_id_mut(monster_id)
            {
                monster.lose_ai_target_and_search(runtime.now_milliseconds());
            }
            return true;
        }
        let direction = get_line_direction(
            target_view.tile_x,
            target_view.tile_y,
            source_view.tile_x,
            source_view.tile_y,
        );
        let origin = ShapeAreaCoordinates {
            x: source_view.tile_x,
            y: source_view.tile_y,
        };
        if let Ok(destination) = CShape::get_direction_position(direction, origin) {
            let figure = CMonster::figure(&property);
            if game.move_owned_monster_step(
                region,
                monster_id,
                destination.x,
                destination.y,
                figure,
            ) {
                if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                    monster.begin_active_ai_move(
                        one_step_move_delay_ms(direction, source.get_speed(), stop_frame),
                        runtime.now_milliseconds(),
                    );
                }
            }
        }
        return true;
    }

    if !schedule_idle {
        return true;
    }
    queue_monster_idle(game, region, monster_id, &property, runtime)
}

/// Выполняет отдельный `OnSearchEnemy` AI7 без движения и без запуска навыка.
pub(crate) fn search_puniness_enemy(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
) -> bool {
    let Some((property, source_view, area_index)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            let source_view = monster.shape_view(&property)?;
            let area_index = monster.move_shape().shape().area_index()?;
            Some((property, source_view, area_index))
        })
    else {
        return false;
    };
    if property.ai != 7 {
        return false;
    }
    let mut selected = None;
    for player_id in region.player_ids_around_area(area_index) {
        let identity = ShapeIdentity {
            object_type: PLAYER_TYPE,
            id: player_id,
            ex_id: crate::public::guid::CGuid::GUID_INVALID,
        };
        let Some(candidate) = live_target_view(game, region, identity) else {
            continue;
        };
        selected = consider_target(
            selected,
            PuninessTarget {
                identity,
                distance: source_view.real_distance(Some(candidate)),
            },
            property.guard_range as i32,
        );
    }
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some(pet) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !CMoveShape::is_died(pet.hit_points()))
        else {
            continue;
        };
        let Some(property_key) = pet.base_property_key() else {
            continue;
        };
        let Some(pet_property) = game.find_monster_property_by_origin_name(property_key) else {
            continue;
        };
        let Some(candidate) = pet.shape_view(pet_property) else {
            continue;
        };
        selected = consider_target(
            selected,
            PuninessTarget {
                identity: candidate.identity,
                distance: source_view.real_distance(Some(candidate)),
            },
            property.guard_range as i32,
        );
    }
    if let Some(selected) = selected
        && let Some(monster) = region.find_monster_by_id_mut(monster_id)
    {
        monster.set_ai_target(selected.identity);
    }
    true
}
