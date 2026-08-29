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

use super::monsterai::one_step_move_delay_ms;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity, ShapeView};
use crate::gameserver::appserver::skills::baseattack::real_distance;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::setup::monsterlist::MonsterProperties;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StupidArcherSearch {
    NoTarget,
    Target(ShapeIdentity),
    Handled,
}

/// Выполняет достигнутый поиск AI6 и его особый отход от слишком близкой
/// цели. `Handled` означает, что владелец выдерживает задержку уже сделанного
/// шага либо завершил текущую попытку отхода.
#[allow(clippy::too_many_arguments, reason = "граница сохраняет отдельные параметры владельца и навыка")]
pub(crate) fn search_stupid_archer_enemy<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    owner: ShapeView,
    area_index: usize,
    property: &MonsterProperties,
    minimum_skill_distance: i32,
    speed: f32,
    runtime: &mut Runtime,
) -> StupidArcherSearch {
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
        selected = consider_stupid_archer_target(
            selected,
            StupidArcherTarget {
                identity: candidate.identity,
                distance: real_distance(
                    owner.tile_x,
                    owner.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
            },
            property.guard_range as i32,
        );
    }
    for pet_id in region.pet_ids_around_area(area_index) {
        let Some(candidate) = region
            .find_monster_by_id(pet_id)
            .filter(|pet| pet.is_tamed() && !CMoveShape::is_died(pet.hit_points()))
            .and_then(|pet| {
                let property =
                    game.find_monster_property_by_origin_name(pet.base_property_key()?)?;
                pet.shape_view(property)
            })
        else {
            continue;
        };
        selected = consider_stupid_archer_target(
            selected,
            StupidArcherTarget {
                identity: candidate.identity,
                distance: real_distance(
                    owner.tile_x,
                    owner.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                ),
            },
            property.guard_range as i32,
        );
    }
    let Some(selected) = selected else {
        return StupidArcherSearch::NoTarget;
    };
    if selected.distance >= minimum_skill_distance {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.set_ai_target(selected.identity);
        }
        return StupidArcherSearch::Target(selected.identity);
    }

    let direction = game.skill_random_below(8);
    let origin = ShapeAreaCoordinates {
        x: owner.tile_x,
        y: owner.tile_y,
    };
    if let Ok(destination) = CShape::get_direction_position(direction, origin) {
        let moved = game.move_owned_monster_step(
            region,
            monster_id,
            destination.x,
            destination.y,
            CMonster::figure(property),
        );
        if moved
            && let Some(monster) = region.find_monster_by_id_mut(monster_id)
        {
            monster.begin_active_ai_move(
                one_step_move_delay_ms(direction, speed, property.stop_frame),
                runtime.now_milliseconds(),
            );
        }
    }
    StupidArcherSearch::Handled
}
