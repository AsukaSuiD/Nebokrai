//! Координация заимствований для периодического состояния потери крови.
//!
//! Формула и точная последовательность RNG принадлежат `bloodlossstate.rs`.
//! Модуль извлекает каноническое состояние, восстанавливает его до применения
//! атаки и передаёт рассчитанный результат общему межвладельческому пути
//! `OnBeenAttacked`.

use super::*;

impl CGame {
    pub(super) fn update_player_blood_loss_state<Runtime: GameMainLoopRuntime>(
        &mut self,
        player_id: i32,
        runtime: &mut Runtime,
    ) -> bool {
        let Some(mut state) = self
            .find_player_mut(player_id)
            .and_then(CPlayer::take_blood_loss_state_for_ai)
        else {
            return false;
        };
        let target = self.find_player(player_id).and_then(|player| {
            Some((
                player.shape().identity(),
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.server_region_id()?,
                player.is_dead(),
                player.combat_properties().cch,
            ))
        });
        let Some((identity, x, y, region_id, dead, critical_chance)) = target else {
            return false;
        };
        let lifetime_now_ms = runtime.now_milliseconds();
        let frequency_now_ms = runtime.now_milliseconds();
        let critical_rate = self.globe_setup.critical_rate();
        let tick = {
            let mut random = |maximum| game_legacy_random(&mut self.random_state, maximum);
            // В слоте `CMonster::GetCCH` стоит базовая реализация
            // `CMoveShape` по адресу `0x004e69b0`: `XOR AX,AX; RET`.
            state.tick(
                lifetime_now_ms,
                frequency_now_ms,
                dead,
                critical_chance,
                critical_rate,
                &mut random,
            )
        };
        match tick {
            BloodLossStateTick::Pending => {
                if let Some(player) = self.find_player_mut(player_id) {
                    let _ = player.replace_blood_loss_state(state);
                }
            }
            BloodLossStateTick::Attack(attack) => {
                let master = state.master();
                if let Some(player) = self.find_player_mut(player_id) {
                    let _ = player.replace_blood_loss_state(state);
                }
                self.apply_periodic_state_attack_to_player(
                    master,
                    player_id,
                    region_id,
                    attack,
                    runtime,
                );
            }
            BloodLossStateTick::Ended => {
                if let Some(player) = self.find_player_mut(player_id) {
                    player.finish_periodic_attack_state(state.skill_id());
                }
                send_blood_loss_state_visual(
                    self,
                    region_id,
                    identity,
                    x,
                    y,
                    state,
                    false,
                    lifetime_now_ms,
                );
                let _ = self.publish_player_states(player_id);
            }
        }
        true
    }

    pub(super) fn update_monster_blood_loss_state<Runtime: GameMainLoopRuntime>(
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
                let state = monster.move_shape_mut().take_blood_loss_state_for_ai()?;
                let shape = monster.move_shape().shape();
                Some((
                    state,
                    shape.identity(),
                    shape.get_tile_x().ok()?,
                    shape.get_tile_y().ok()?,
                    monster.hit_points() == 0,
                ))
            });
        self.restore_region_owner(owner);
        let Some((mut state, identity, x, y, dead)) = state_and_target else {
            return false;
        };
        let lifetime_now_ms = runtime.now_milliseconds();
        let frequency_now_ms = runtime.now_milliseconds();
        let critical_rate = self.globe_setup.critical_rate();
        let tick = {
            let mut random = |maximum| game_legacy_random(&mut self.random_state, maximum);
            state.tick(
                lifetime_now_ms,
                frequency_now_ms,
                dead,
                0,
                critical_rate,
                &mut random,
            )
        };
        match tick {
            BloodLossStateTick::Pending => {
                if let Some(mut owner) = self.take_region_owner(region_id) {
                    if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
                        let _ = monster.move_shape_mut().replace_blood_loss_state(state);
                    }
                    self.restore_region_owner(owner);
                }
            }
            BloodLossStateTick::Attack(attack) => {
                let master = state.master();
                if let Some(mut owner) = self.take_region_owner(region_id) {
                    if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
                        let _ = monster.move_shape_mut().replace_blood_loss_state(state);
                    }
                    self.restore_region_owner(owner);
                }
                self.apply_periodic_state_attack_to_monster(
                    master,
                    monster_id,
                    region_id,
                    attack,
                    runtime,
                );
            }
            BloodLossStateTick::Ended => {
                if let Some(mut owner) = self.take_region_owner(region_id) {
                    if let Some(monster) = owner.base_mut().find_monster_by_id_mut(monster_id) {
                        monster
                            .move_shape_mut()
                            .finish_periodic_attack_state(state.skill_id());
                    }
                    self.restore_region_owner(owner);
                }
                send_blood_loss_state_visual(
                    self,
                    region_id,
                    identity,
                    x,
                    y,
                    state,
                    false,
                    lifetime_now_ms,
                );
            }
        }
        true
    }
}
