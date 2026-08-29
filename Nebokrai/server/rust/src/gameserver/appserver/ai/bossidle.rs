//! Общий достигнутый префикс `OnIdle` синего босса и демона-босса.
//!
//! Оба исходных owner-а после выбора текущего навыка выполняют один
//! `random(10000)`: при успехе — один `random(8)` и соседний шаг, иначе —
//! ожидание `stop_frame`. Поиск противника начинается только после этой
//! последовательной задержки. Игровые формулы и выбор навыка остаются у
//! конкретных владельцев боссов, а `CGame` координирует только пространственный шаг.

use super::monsterai::one_step_move_delay_ms;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::setup::monsterlist::MonsterProperties;

/// Ставит подтверждённую последовательность `MOVE/STAND → SEARCH_ENEMY` в
/// общую FIFO-очередь `CBaseAI`. Поэтому задержка уже совершённого шага не
/// допускает повторного RNG до отдельного такта поиска противника.
pub(crate) fn queue_boss_idle<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    runtime: &mut Runtime,
) -> bool {
    let Some((origin, speed)) = region.find_monster_by_id(monster_id).and_then(|monster| {
        let shape = monster.move_shape().shape();
        Some((
            ShapeAreaCoordinates {
                x: shape.get_tile_x().ok()?,
                y: shape.get_tile_y().ok()?,
            },
            shape.get_speed(),
        ))
    }) else {
        return false;
    };
    let move_roll = game.skill_random_below(10_000);
    let movement_delay = if (move_roll as u32) < property.move_timer {
        let direction = game.skill_random_below(8);
        let Ok(destination) = CShape::get_direction_position(direction, origin) else {
            return queue_search(region, monster_id, runtime.now_milliseconds());
        };
        if !game.move_owned_monster_step(
            region,
            monster_id,
            destination.x,
            destination.y,
            CMonster::figure(property),
        ) {
            return queue_search(region, monster_id, runtime.now_milliseconds());
        }
        Some(one_step_move_delay_ms(direction, speed, property.stop_frame))
    } else {
        None
    };
    let now_ms = runtime.now_milliseconds();
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        if let Some(delay_ms) = movement_delay {
            monster.begin_active_ai_move(delay_ms, now_ms);
        } else {
            monster.begin_active_ai_stand(property.stop_frame, now_ms);
        }
        monster.begin_active_ai_search_enemy(now_ms);
    }
    true
}

fn queue_search(
    region: &mut CServerRegion,
    monster_id: i32,
    now_ms: u32,
) -> bool {
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.begin_active_ai_search_enemy(now_ms);
    }
    true
}
