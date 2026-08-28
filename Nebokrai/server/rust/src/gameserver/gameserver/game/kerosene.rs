//! Borrow-координация периодического `KeroseneState`.
//!
//! Формула, строгие таймеры и DB-кодек принадлежат `kerosenestate.rs`.
//! Здесь `CGame` только извлекает конкретного player/monster owner-а,
//! восстанавливает состояние перед атакой и выполняет фактическую доставку.

use super::*;
use crate::gameserver::appserver::skills::kerosenestate::{send_kerosene_state_visual, KeroseneStateTick};

impl CGame {
    pub(super) fn update_player_kerosene_state<Runtime: GameMainLoopRuntime>(&mut self, player_id: i32, runtime: &mut Runtime) -> bool {
        let target = self.find_player_mut(player_id).and_then(|player| { let identity = player.shape().identity(); let x = player.shape().get_tile_x().ok()?; let y = player.shape().get_tile_y().ok()?; let region = player.server_region_id()?; let dead = player.is_dead(); let state = player.take_kerosene_state_for_ai()?; Some((state, identity, x, y, region, dead)) });
        let Some((mut state, identity, x, y, region_id, dead)) = target else { return false }; let lifetime_now = runtime.now_milliseconds();
        let tick = if state.ended(lifetime_now, dead) { KeroseneStateTick::Ended } else { state.tick(runtime.now_milliseconds()) };
        match tick {
            KeroseneStateTick::Pending => if let Some(player) = self.find_player_mut(player_id) { player.restore_kerosene_state_after_ai(state); },
            KeroseneStateTick::Attack(attack) => { let master = state.master(); if let Some(player) = self.find_player_mut(player_id) { player.restore_kerosene_state_after_ai(state); } self.apply_owned_skill_attack_to_player(master, player_id, region_id, attack, runtime); }
            KeroseneStateTick::Ended => { if let Some(player) = self.find_player_mut(player_id) { player.finish_kerosene_state(state); } send_kerosene_state_visual(self, region_id, identity, x, y, state, false, lifetime_now); let _ = self.publish_player_states(player_id); }
        } true
    }

    pub(super) fn update_monster_kerosene_state<Runtime: GameMainLoopRuntime>(&mut self, region_id: i32, monster_id: i32, runtime: &mut Runtime) -> bool {
        let Some(mut owner) = self.take_region_owner(region_id) else { return false }; let target = owner.base_mut().find_monster_by_id_mut(monster_id).and_then(|monster| { let identity = monster.move_shape().shape().identity(); let x = monster.move_shape().shape().get_tile_x().ok()?; let y = monster.move_shape().shape().get_tile_y().ok()?; let dead = monster.hit_points() == 0; let state = monster.move_shape_mut().take_kerosene_state_for_ai()?; Some((state, identity, x, y, dead)) }); self.restore_region_owner(owner);
        let Some((mut state, identity, x, y, dead)) = target else { return false }; let lifetime_now = runtime.now_milliseconds(); let tick = if state.ended(lifetime_now, dead) { KeroseneStateTick::Ended } else { state.tick(runtime.now_milliseconds()) };
        match tick {
            KeroseneStateTick::Pending => { if let Some(mut owner) = self.take_region_owner(region_id) { if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) { monster.move_shape_mut().restore_kerosene_state_after_ai(state); } self.restore_region_owner(owner); } },
            KeroseneStateTick::Attack(attack) => { let master = state.master(); if let Some(mut owner) = self.take_region_owner(region_id) { if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) { monster.move_shape_mut().restore_kerosene_state_after_ai(state); } self.restore_region_owner(owner); } self.apply_owned_skill_attack_to_monster(master, monster_id, region_id, attack, runtime); },
            KeroseneStateTick::Ended => { if let Some(mut owner) = self.take_region_owner(region_id) { if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) { monster.move_shape_mut().finish_kerosene_state(state); } self.restore_region_owner(owner); } send_kerosene_state_visual(self, region_id, identity, x, y, state, false, lifetime_now); }
        } true
    }
}
