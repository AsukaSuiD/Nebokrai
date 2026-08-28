//! Borrow-координация периодического состояния `CLeafCutState`.
//!
//! Формула, RNG и DB-кодек принадлежат `leafcutstate.rs`. Здесь состояние
//! временно извлекается из канонического владельца цели, восстанавливается до
//! межвладельческого `OnBeenAttacked` и удаляется только на подтверждённой
//! границе срока жизни или смерти.

use super::*;

impl CGame {
    pub(super) fn update_player_leaf_cut_state<Runtime: GameMainLoopRuntime>(
        &mut self,
        player_id: i32,
        runtime: &mut Runtime,
    ) -> bool {
        let target = self.find_player_mut(player_id).and_then(|player| {
            let identity = player.shape().identity();
            let x = player.shape().get_tile_x().ok()?;
            let y = player.shape().get_tile_y().ok()?;
            let region_id = player.server_region_id()?;
            let dead = player.is_dead();
            let critical_chance = player.combat_properties().cch;
            let state = player.take_leaf_cut_state_for_ai()?;
            Some((state, identity, x, y, region_id, dead, critical_chance))
        });
        let Some((mut state, identity, x, y, region_id, dead, critical_chance)) = target else {
            return false;
        };
        let lifetime_now = runtime.now_milliseconds();
        let frequency_now = runtime.now_milliseconds();
        let critical_rate = self.globe_setup.critical_rate();
        let tick = {
            let mut random = |maximum| game_legacy_random(&mut self.random_state, maximum);
            state.tick(
                lifetime_now,
                frequency_now,
                dead,
                critical_chance,
                critical_rate,
                &mut random,
            )
        };
        match tick {
            LeafCutStateTick::Pending => {
                if let Some(player) = self.find_player_mut(player_id) {
                    player.restore_leaf_cut_state_after_ai(state);
                }
            }
            LeafCutStateTick::Attack(attack) => {
                let master = state.master();
                if let Some(player) = self.find_player_mut(player_id) {
                    player.restore_leaf_cut_state_after_ai(state);
                }
                self.apply_owned_skill_attack_to_player(
                    master,
                    player_id,
                    region_id,
                    attack,
                    runtime,
                );
            }
            LeafCutStateTick::Ended => {
                if let Some(player) = self.find_player_mut(player_id) {
                    player.finish_leaf_cut_state(state);
                }
                send_leaf_cut_state_visual(
                    self,
                    region_id,
                    identity,
                    x,
                    y,
                    state,
                    false,
                    lifetime_now,
                );
                let _ = self.publish_player_states(player_id);
            }
        }
        true
    }

    pub(super) fn update_monster_leaf_cut_state<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        monster_id: i32,
        runtime: &mut Runtime,
    ) -> bool {
        let Some(mut owner) = self.take_region_owner(region_id) else {
            return false;
        };
        let state_and_target = owner
            .base_mut()
            .find_monster_by_id_mut(monster_id)
            .and_then(|monster| {
                let shape = monster.move_shape().shape();
                let identity = shape.identity();
                let x = shape.get_tile_x().ok()?;
                let y = shape.get_tile_y().ok()?;
                let dead = monster.hit_points() == 0;
                let state = monster.move_shape_mut().take_leaf_cut_state_for_ai()?;
                Some((state, identity, x, y, dead))
            });
        self.restore_region_owner(owner);
        let Some((mut state, identity, x, y, dead)) = state_and_target else {
            return false;
        };
        let lifetime_now = runtime.now_milliseconds();
        let frequency_now = runtime.now_milliseconds();
        let critical_rate = self.globe_setup.critical_rate();
        let tick = {
            let mut random = |maximum| game_legacy_random(&mut self.random_state, maximum);
            // В слоте `CMonster::GetCCH` стоит базовая реализация
            // `CMoveShape` по адресу `0x004e69b0`: `XOR AX,AX; RET`.
            state.tick(
                lifetime_now,
                frequency_now,
                dead,
                0,
                critical_rate,
                &mut random,
            )
        };
        match tick {
            LeafCutStateTick::Pending => {
                if let Some(mut owner) = self.take_region_owner(region_id) {
                    if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
                        monster
                            .move_shape_mut()
                            .restore_leaf_cut_state_after_ai(state);
                    }
                    self.restore_region_owner(owner);
                }
            }
            LeafCutStateTick::Attack(attack) => {
                let master = state.master();
                if let Some(mut owner) = self.take_region_owner(region_id) {
                    if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
                        monster
                            .move_shape_mut()
                            .restore_leaf_cut_state_after_ai(state);
                    }
                    self.restore_region_owner(owner);
                }
                self.apply_owned_skill_attack_to_monster(
                    master,
                    monster_id,
                    region_id,
                    attack,
                    runtime,
                );
            }
            LeafCutStateTick::Ended => {
                if let Some(mut owner) = self.take_region_owner(region_id) {
                    if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
                        monster.move_shape_mut().finish_leaf_cut_state(state);
                    }
                    self.restore_region_owner(owner);
                }
                send_leaf_cut_state_visual(
                    self,
                    region_id,
                    identity,
                    x,
                    y,
                    state,
                    false,
                    lifetime_now,
                );
            }
        }
        true
    }
}
