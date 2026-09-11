//! Межвладельческая координация `RoarState`.
//!
//! Формулы и жизненный цикл принадлежат `roarstate.rs`. Здесь остаются только
//! временное извлечение владельцев региона и игрока, атомарная замена,
//! перерасчёт свойств цели и рассылка вокруг в порядке `End → Begin`.
//! CRoar::AI: успешный Begin 0x0054B4EA → push_back 0x0054B500 →
//! virtual UpdateProperty 0x0054B50A, отдельно для каждой достигнутой цели.

use super::*;
use crate::gameserver::appserver::skills::roarstate::{
    RoarState, send_roar_state_visual,
};

impl CGame {
    pub(crate) fn install_roar_state<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        state: RoarState,
        runtime: &mut Runtime,
    ) -> bool {
        let changed = match target.object_type {
            PLAYER_TYPE => self.find_player_mut(target.id).and_then(|player| {
                let x = player.shape().get_tile_x().ok()?;
                let y = player.shape().get_tile_y().ok()?;
                Some((x, y, player.replace_roar_state(state)))
            }),
            MONSTER_TYPE => if let Some(mut owner) = self.take_region_owner(region_id) {
                let changed = owner.base_mut().find_monster_by_id_mut(target.id).and_then(|monster| {
                    let x = monster.move_shape().shape().get_tile_x().ok()?;
                    let y = monster.move_shape().shape().get_tile_y().ok()?;
                    Some((x, y, monster.move_shape_mut().replace_roar_state(state)))
                });
                self.restore_region_owner(owner);
                changed
            } else { None },
            _ => None,
        };
        let Some((x, y, previous)) = changed else { return false };
        if let Some(previous) = previous {
            send_roar_state_visual(
                self,
                region_id,
                target,
                x,
                y,
                previous,
                false,
                || runtime.now_milliseconds(),
            );
        }
        send_roar_state_visual(
            self,
            region_id,
            target,
            x,
            y,
            state,
            true,
            || runtime.now_milliseconds(),
        );
        let _ = self.update_move_shape_properties(region_id, target);
        true
    }

}
