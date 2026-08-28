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

    pub(super) fn finish_player_god_bless<Runtime: GameMainLoopRuntime>(&mut self, player_id: i32, now_ms: u32, runtime: &mut Runtime) -> bool {
        let ended = self.find_player_mut(player_id).and_then(|player| {
            let region = player.server_region_id()?;
            let x = player.shape().get_tile_x().ok()?;
            let y = player.shape().get_tile_y().ok()?;
            let state = player.take_expired_god_bless_state(now_ms)?;
            Some((region, x, y, state))
        });
        let Some((region, x, y, state)) = ended else { return false };
        send_god_bless_state_visual(self, region, ShapeIdentity { object_type: PLAYER_TYPE, id: player_id, ex_id: CGuid::GUID_INVALID }, x, y, state, false, now_ms);
        let _ = self.update_player_properties(player_id, runtime);
        true
    }

    pub(super) fn finish_monster_god_bless(&mut self, region_id: i32, monster_id: i32, now_ms: u32) -> bool {
        let ended = if let Some(mut owner) = self.take_region_owner(region_id) {
            let result = owner.base_mut().find_monster_by_id_mut(monster_id).and_then(|monster| {
                let x = monster.move_shape().shape().get_tile_x().ok()?;
                let y = monster.move_shape().shape().get_tile_y().ok()?;
                let state = monster.move_shape_mut().take_expired_god_bless_state(now_ms)?;
                Some((x, y, state))
            });
            self.restore_region_owner(owner);
            result
        } else { None };
        let Some((x, y, state)) = ended else { return false };
        send_god_bless_state_visual(self, region_id, ShapeIdentity { object_type: MONSTER_TYPE, id: monster_id, ex_id: CGuid::GUID_INVALID }, x, y, state, false, now_ms);
        true
    }
}
