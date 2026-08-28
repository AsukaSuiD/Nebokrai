//! Межвладельческое применение состояний и отбрасывания семейства рывков.
//!
//! Выбор цели, длительность и конечная клетка принадлежат владельцу навыка.
//! `CGame` временно извлекает регион только для атомарной работы канонического
//! состояния игрока либо монстра, пространственного индекса, сети и ожидания ИИ.

use super::*;
use crate::gameserver::appserver::skills::rushstate::{
    RushState, replace_monster_rush_state, replace_player_rush_state,
};
use crate::gameserver::appserver::skills::rushstate2::{
    Rush2State, replace_monster_rush_2_state, replace_player_rush_2_state,
};

impl CGame {
    pub(crate) fn apply_owned_skill_contact<Runtime: GameMainLoopRuntime>(
        &mut self,
        master: crate::gameserver::appserver::masterinfo::MasterInfo,
        target: ShapeIdentity,
        region_id: i32,
        mut attack: AttackInformation,
        runtime: &mut Runtime,
    ) {
        if target.object_type == PLAYER_TYPE {
            let Some((attacker_properties, attacker_occupation, target_properties, target_mana, target_war_soul_mana)) =
                self.find_player(master.master_id).and_then(|attacker| {
                    let target = self.find_player(target.id)?;
                    Some((
                        attacker.combat_properties(),
                        attacker.occupation(),
                        target.combat_properties(),
                        target.mana(),
                        target.war_soul_mana(&self.goods_factory),
                    ))
                })
            else { return };
            let _ = self.player_on_first_attack_at_victim(
                master.master_id, target.id, Some(region_id), runtime,
            );
            let pillar_damage_factor = self.find_player(target.id)
                .and_then(CPlayer::pillar_state).map(|state| state.damage_factor());
            let mut defense_shields = self.find_player_mut(target.id)
                .map(CPlayer::take_defense_shields).unwrap_or_default();
            let mut random = |maximum| game_legacy_random(&mut self.random_state, maximum);
            defend_player_base_attack(
                &mut attack,
                attacker_properties,
                attacker_occupation,
                target_properties,
                target_mana,
                target_war_soul_mana,
                &self.globe_setup,
                &mut random,
                &mut defense_shields,
                pillar_damage_factor,
            );
            if let Some(target) = self.find_player_mut(target.id) {
                target.restore_defense_shields(defense_shields);
            }
            if attack.full_miss != 0 {
                let mut missed = CMessage::new(0x000b_f612);
                missed.add_byte(attack.full_miss);
                missed.add_long(PLAYER_TYPE);
                missed.add_long(target.id);
                let _ = self.send_player_shape_around(target.id, None, &missed);
            } else {
                self.damage_player_armor(target.id, runtime);
            }
            return;
        }
        if target.object_type != MONSTER_TYPE { return }
        let Some((property, target_properties, target_master, tamed, carriage, x, y)) = self
            .find_region(region_id)
            .and_then(|owner| {
                let monster = owner.base().find_monster_by_id(target.id)?;
                let property = self.find_monster_property_by_origin_name(monster.base_property_key()?)?.clone();
                let shape = monster.move_shape().shape();
                Some((
                    property.clone(),
                    monster.combat_properties(&property),
                    monster.master_info(),
                    monster.is_tamed(),
                    monster.is_carriage(&property),
                    shape.get_tile_x().ok()?,
                    shape.get_tile_y().ok()?,
                ))
            })
        else { return };
        if (tamed || carriage) && target_master.master_type == PLAYER_TYPE && target_master.master_id != 0 {
            let _ = self.player_on_first_attack_at_position(
                master.master_id,
                target_master.master_id,
                Some(region_id),
                (x, y),
                runtime,
            );
        }
        let Some((attacker_properties, attacker_occupation, attacker_level)) = self
            .find_player(master.master_id)
            .map(|attacker| (attacker.combat_properties(), attacker.occupation(), attacker.level()))
        else { return };
        self.apply_guard_monster_first_attack(
            master.master_id, region_id, &property, runtime.now_milliseconds(),
        );
        let mut random = |maximum| game_legacy_random(&mut self.random_state, maximum);
        defend_monster_base_attack(
            &mut attack,
            attacker_properties,
            attacker_occupation,
            attacker_level,
            target_properties,
            &self.globe_setup,
            &mut random,
        );
        if attack.full_miss != 0 {
            let mut missed = CMessage::new(0x000b_f612);
            missed.add_byte(attack.full_miss);
            missed.add_long(MONSTER_TYPE);
            missed.add_long(target.id);
            if let Some(owner) = self.find_region(region_id)
                && let Some(monster) = owner.base().find_monster_by_id(target.id)
            {
                let _ = self.send_game_shape_around(
                    owner.base(), monster.move_shape().shape(), None, &missed,
                );
            }
        }
    }

    #[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца состояния и пространственный эффект")]
    pub(crate) fn apply_rush_control<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        state: RushState,
        destination_x: i32,
        destination_y: i32,
        duration_ms: u32,
        now_ms: u32,
        runtime: &mut Runtime,
    ) -> bool {
        if target.object_type == PLAYER_TYPE {
            if !replace_player_rush_state(self, target.id, state, now_ms) {
                return false;
            }
            let Some(mut owner) = self.take_region_owner(region_id) else { return false };
            let moved = self.force_move_owned_shape(
                owner.base_mut(), target, destination_x, destination_y, duration_ms, runtime,
            ).is_some();
            self.restore_region_owner(owner);
            return moved;
        }
        if target.object_type != MONSTER_TYPE { return false }
        let Some(mut owner) = self.take_region_owner(region_id) else { return false };
        let installed = replace_monster_rush_state(
            self, owner.base_mut(), target.id, state, now_ms,
        );
        let moved = installed && self.force_move_owned_shape(
            owner.base_mut(), target, destination_x, destination_y, duration_ms, runtime,
        ).is_some();
        self.restore_region_owner(owner);
        moved
    }

    #[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца состояния и пространственный эффект")]
    pub(crate) fn apply_rush_2_control<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        state: Rush2State,
        destination_x: i32,
        destination_y: i32,
        duration_ms: u32,
        now_ms: u32,
        runtime: &mut Runtime,
    ) -> bool {
        if target.object_type == PLAYER_TYPE {
            if !replace_player_rush_2_state(self, target.id, state, now_ms) { return false }
            let Some(mut owner) = self.take_region_owner(region_id) else { return false };
            let _ = self.force_move_owned_shape(
                owner.base_mut(), target, destination_x, destination_y, duration_ms, runtime,
            );
            self.restore_region_owner(owner);
            return true;
        }
        if target.object_type != MONSTER_TYPE { return false }
        let Some(mut owner) = self.take_region_owner(region_id) else { return false };
        let installed = replace_monster_rush_2_state(self, owner.base_mut(), target.id, state, now_ms);
        if installed {
            let _ = self.force_move_owned_shape(
                owner.base_mut(), target, destination_x, destination_y, duration_ms, runtime,
            );
        }
        self.restore_region_owner(owner);
        installed
    }
}
