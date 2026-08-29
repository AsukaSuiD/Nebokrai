//! Межвладельческая координация `GodBlessState`.
//!
//! Формулы, replacement и lifecycle принадлежат skill/state-owner-ам. Здесь
//! остаются только временное извлечение региона, перерасчёт player owner-а и
//! фактическая around-доставка в подтверждённом порядке end → begin.

use super::*;
use crate::gameserver::appserver::skills::godblessstate::{GodBlessState, send_god_bless_state_visual};

impl CGame {
    pub(crate) fn install_god_bless_state<Runtime: GameMainLoopRuntime>(&mut self, region_id: i32, target: ShapeIdentity, state: GodBlessState, runtime: &mut Runtime) -> bool {
        let changed = match target.object_type {
            PLAYER_TYPE => self.find_player_mut(target.id).and_then(|player| {
                let x = player.shape().get_tile_x().ok()?;
                let y = player.shape().get_tile_y().ok()?;
                let previous = player.replace_god_bless_state(state);
                Some((x, y, previous))
            }),
            MONSTER_TYPE => if let Some(mut owner) = self.take_region_owner(region_id) {
                let result = owner.base_mut().find_monster_by_id_mut(target.id).and_then(|monster| {
                    let x = monster.move_shape().shape().get_tile_x().ok()?;
                    let y = monster.move_shape().shape().get_tile_y().ok()?;
                    let previous = monster.move_shape_mut().replace_god_bless_state(state);
                    Some((x, y, previous))
                });
                self.restore_region_owner(owner);
                result
            } else { None },
            _ => None,
        };
        let Some((x, y, previous)) = changed else { return false };
        if let Some(previous) = previous { send_god_bless_state_visual(self, region_id, target, x, y, previous, false, runtime.now_milliseconds()); }
        send_god_bless_state_visual(self, region_id, target, x, y, state, true, runtime.now_milliseconds());
        if target.object_type == PLAYER_TYPE { let _ = self.update_player_properties(target.id, runtime); }
        true
    }

}
