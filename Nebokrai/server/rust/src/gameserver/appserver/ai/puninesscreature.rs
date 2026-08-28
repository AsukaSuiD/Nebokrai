//! Достигнутый ИИ слабого существа `CPuninessCreature`.
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb` и
//! исходный владелец `appserver/ai/puninesscreature.cpp` подтверждают поиск
//! ближайшего живого игрока или питомца в девяти соседних областях. При равной
//! `RealDistance` побеждает более поздняя запись. Внутри дальности охраны
//! существо делает один шаг от цели без RNG; только за её пределами применяется
//! проверка дальности преследования и возможная потеря цели.
//!
//! `OnSchedule` ниже остаётся RAW в части точных проверок очереди событий;
//! достигнутый `Tracing` вызывается реальным циклом региона до боевого ИИ.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\puninesscreature.cpp

// FUNCTION: CPuninessCreature::OnSchedule
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: реальный цикл региона вызывает достигнутый `Tracing` только для
// AI `7`; точная проверка внутренних полей очереди остаётся ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\puninesscreature.cpp:135
// RVA: 0x0020F4B0
// ADDRESS: 0060f4b0
// PROTOTYPE: void __thiscall OnSchedule(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

use super::monsterai::one_step_move_delay_ms;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity, ShapeView};
use crate::gameserver::appserver::skills::baseattack::{real_distance, time_reached};
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
    let Some((property, source, source_view, target, move_delay)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            Some((
                property.clone(),
                monster.move_shape().shape().clone(),
                monster.shape_view(&property)?,
                monster.ai_target(),
                monster.trace_move_delay(),
            ))
        })
    else {
        return false;
    };
    if property.ai != 7 {
        return false;
    }
    if let Some(target) = target {
        let Some(target_view) = live_target_view(game, region, target) else {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        };
        let distance = real_distance(
            source_view.tile_x,
            source_view.tile_y,
            target_view.tile_x,
            target_view.tile_y,
        );
        if distance > property.guard_range as i32 {
            if distance > property.chase_range as i32
                && let Some(monster) = region.find_monster_by_id_mut(monster_id)
            {
                monster.clear_ai_target();
            }
            return true;
        }
        if let Some(delay) = move_delay {
            let now_ms = runtime.now_milliseconds();
            if !time_reached(now_ms, delay.started_at_ms, delay.delay_ms) {
                return true;
            }
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_trace_move_delay();
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
                let started_at_ms = runtime.now_milliseconds();
                if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                    monster.begin_trace_move_delay(
                        started_at_ms,
                        one_step_move_delay_ms(direction, source.get_speed(), property.stop_frame),
                    );
                }
            }
        }
        return true;
    }

    let Some(area_index) = source.area_index() else {
        return true;
    };
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
                distance: real_distance(
                    source_view.tile_x,
                    source_view.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
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
                distance: real_distance(
                    source_view.tile_x,
                    source_view.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
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
