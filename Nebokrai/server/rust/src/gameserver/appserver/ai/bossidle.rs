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
use crate::gameserver::appserver::skills::baseattack::time_reached;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::setup::monsterlist::MonsterProperties;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BossIdleProgress {
    Waiting,
    SearchEnemy,
}

/// `CBossBlue::OnSchedule` и `CBossFiend::OnSchedule` переходят от
/// `Tracing/CheckCast` прямо к `ASA_ATTACK` и не имеют дополнительной проверки
/// `CMonster::GetAttackSpeed`, присутствующей в обычном `CMonsterAI`.
/// Задержка повторного применения самого навыка остаётся отдельной проверкой.
pub(crate) const fn schedule_attack_interval(ai_type: u32, ordinary_interval_ms: u32) -> u32 {
    if matches!(ai_type, 0x67 | 0x68) {
        0
    } else {
        ordinary_interval_ms
    }
}

/// Проводит последовательный `MOVE/STAND → SEARCH_ENEMY`, не материализуя
/// параллельную очередь событий. `trace_move_delay` хранит ровно одно уже
/// начатое действие и не допускает повторного RNG до его завершения.
pub(crate) fn advance_boss_idle<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    property: &MonsterProperties,
    runtime: &mut Runtime,
) -> BossIdleProgress {
    let Some((delay, origin, speed)) = region.find_monster_by_id(monster_id).and_then(|monster| {
        let shape = monster.move_shape().shape();
        Some((
            monster.trace_move_delay(),
            ShapeAreaCoordinates {
                x: shape.get_tile_x().ok()?,
                y: shape.get_tile_y().ok()?,
            },
            shape.get_speed(),
        ))
    }) else {
        return BossIdleProgress::Waiting;
    };
    if let Some(delay) = delay {
        let now_ms = runtime.now_milliseconds();
        if !time_reached(now_ms, delay.started_at_ms, delay.delay_ms) {
            return BossIdleProgress::Waiting;
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_trace_move_delay();
        }
        return BossIdleProgress::SearchEnemy;
    }

    let move_roll = game.skill_random_below(10_000);
    let delay_ms = if (move_roll as u32) < property.move_timer {
        let direction = game.skill_random_below(8);
        let Ok(destination) = CShape::get_direction_position(direction, origin) else {
            return BossIdleProgress::SearchEnemy;
        };
        if !game.move_owned_monster_step(
            region,
            monster_id,
            destination.x,
            destination.y,
            CMonster::figure(property),
        ) {
            return BossIdleProgress::SearchEnemy;
        }
        one_step_move_delay_ms(direction, speed, property.stop_frame)
    } else {
        property.stop_frame
    };
    if delay_ms == 0 {
        return BossIdleProgress::SearchEnemy;
    }
    let now_ms = runtime.now_milliseconds();
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.begin_trace_move_delay(now_ms, delay_ms);
    }
    BossIdleProgress::Waiting
}
