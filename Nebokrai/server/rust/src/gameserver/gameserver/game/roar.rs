//! Межвладельческая координация `RoarState`.
//!
//! Формулы и жизненный цикл принадлежат `roarstate.rs`. Здесь остаются только
//! временное извлечение владельцев региона и игрока, атомарная замена,
//! перерасчёт свойств игрока и рассылка вокруг в порядке `End → Begin`.

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
        let now_ms = runtime.now_milliseconds();
        if let Some(previous) = previous {
            send_roar_state_visual(self, region_id, target, x, y, previous, false, now_ms);
        }
        send_roar_state_visual(self, region_id, target, x, y, state, true, now_ms);
        if target.object_type == PLAYER_TYPE {
            let _ = self.update_player_properties(target.id, runtime);
        }
        true
    }

    pub(super) fn finish_player_roar<Runtime: GameMainLoopRuntime>(
        &mut self,
        player_id: i32,
        now_ms: u32,
        runtime: &mut Runtime,
    ) -> bool {
        let ended = self.find_player_mut(player_id).and_then(|player| {
            let region_id = player.server_region_id()?;
            let x = player.shape().get_tile_x().ok()?;
            let y = player.shape().get_tile_y().ok()?;
            let identity = player.shape().identity();
            let state = player.take_expired_roar_state(now_ms)?;
            Some((region_id, x, y, identity, state))
        });
        let Some((region_id, x, y, identity, state)) = ended else { return false };
        send_roar_state_visual(self, region_id, identity, x, y, state, false, now_ms);
        let _ = self.update_player_properties(player_id, runtime);
        true
    }

    pub(super) fn finish_monster_roar(
        &mut self,
        region_id: i32,
        monster_id: i32,
        now_ms: u32,
    ) -> bool {
        let ended = if let Some(mut owner) = self.take_region_owner(region_id) {
            let ended = owner.base_mut().find_monster_by_id_mut(monster_id).and_then(|monster| {
                let x = monster.move_shape().shape().get_tile_x().ok()?;
                let y = monster.move_shape().shape().get_tile_y().ok()?;
                let identity = monster.move_shape().shape().identity();
                let state = monster.move_shape_mut().take_expired_roar_state(now_ms)?;
                Some((x, y, identity, state))
            });
            self.restore_region_owner(owner);
            ended
        } else { None };
        let Some((x, y, identity, state)) = ended else { return false };
        send_roar_state_visual(self, region_id, identity, x, y, state, false, now_ms);
        true
    }
}
