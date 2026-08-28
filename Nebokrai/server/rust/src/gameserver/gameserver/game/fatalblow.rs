//! Межвладельческая координация смертельного удара боевого духа.
//!
//! Формула, стадии и двоичное представление принадлежат `fatalblow.rs` и
//! `fatalblowphalanx.rs`. Здесь остаются только временное извлечение региона,
//! проверка общей `IsAttackAble`-семантики, регистрация снаряда и фактическая
//! круговая доставка, которым нужен доступ к нескольким владельцам `CGame`.

use super::*;

impl CGame {
    pub(crate) fn add_fatal_blow_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx: CFatalBlowPhalanx,
        tile_x: i32,
        tile_y: i32,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_fatal_blow_phalanx(
            phalanx,
            tile_x,
            tile_y,
            self.area_width,
            self.area_height,
            started_at_ms,
            runtime,
        );
        self.restore_region_owner(owner);
        Some(result)
    }

    pub(crate) fn send_fatal_blow_phalanx_entry<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx_id: i32,
        runtime: &mut Runtime,
    ) -> Option<()> {
        let phalanx = self
            .find_region(region_id)?
            .base()
            .find_skill_phalanx(phalanx_id)?;
        let SummonedSkillShape::FatalBlow(phalanx) = phalanx else {
            return None;
        };
        let identity = phalanx.shape().identity();
        let tile_x = phalanx.shape().get_tile_x().ok()?;
        let tile_y = phalanx.shape().get_tile_y().ok()?;
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let payload_size = i32::try_from(payload.len()).ok()?;
        let mut message = CMessage::new(0x000b_f502);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.base_mut().add_guid(identity.ex_id);
        message.add_long(payload_size);
        message.base_mut().add(&payload);
        message.base_mut().add_char(0);
        let _ = self.send_shape_position_around(region_id, tile_x, tile_y, &message);
        Some(())
    }

    pub(super) fn calculate_fatal_blow_attack(
        &mut self,
        phalanx: &CFatalBlowPhalanx,
    ) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
        let master = phalanx.master();
        if master.master_type != PLAYER_TYPE || master.master_id == 0 {
            return None;
        }
        let player = self.find_player(master.master_id)?;
        let battle_fairy_attack = player.war_soul_attack(&self.goods_factory)?;
        let battle_fairy_attack = (f64::from(battle_fairy_attack) * 0.0001)
            .round_ties_even() as i32;
        let combat = player.combat_properties();
        let occupation = player.occupation();
        let attacker_level = player.level();
        let mut random_below = |maximum| game_legacy_random(&mut self.random_state, maximum);
        let attack = phalanx.calculate_attack(battle_fairy_attack, &mut random_below);
        Some((attack, combat, occupation, attacker_level))
    }

    pub(super) fn fatal_blow_attack_ready(
        &self,
        phalanx: &CFatalBlowPhalanx,
        target: ShapeIdentity,
        region_id: i32,
    ) -> Option<bool> {
        let master = phalanx.master();
        if self.find_player(master.master_id).is_none() {
            return Some(false);
        }
        match target.object_type {
            PLAYER_TYPE => {
                let target = self.find_player(target.id)?;
                Some(
                    !target.is_dead()
                        && target.server_region_id() == Some(region_id)
                        && self.player_base_attackable(master.master_id, target.player_id()),
                )
            }
            MONSTER_TYPE => {
                let monster = self
                    .find_region(region_id)?
                    .base()
                    .find_monster_by_id(target.id)?;
                let property = monster
                    .base_property_key()
                    .and_then(|key| self.find_monster_property_by_origin_name(key));
                let Some(property) = property else {
                    return Some(false);
                };
                if monster.hit_points() == 0
                    || monster.move_shape().is_god()
                    || !self.guard_monster_attackable(master.master_id, region_id, property)
                {
                    return Some(false);
                }
                if !monster.is_tamed() && !monster.is_carriage(property) {
                    return Some(true);
                }
                let owner = monster.master_info();
                if owner.master_type != PLAYER_TYPE || owner.master_id == 0 {
                    return Some(true);
                }
                Some(if owner.master_id == master.master_id {
                    master.permitted_to_kill_criminal != 0
                } else {
                    self.player_base_attackable(master.master_id, owner.master_id)
                })
            }
            _ => None,
        }
    }

    pub(super) fn finish_fatal_blow_phalanx(&mut self, region_id: i32, phalanx_id: i32) {
        if let Some(mut owner) = self.take_region_owner(region_id) {
            owner.base_mut().finish_fatal_blow_phalanx(phalanx_id);
            self.restore_region_owner(owner);
        }
    }
}
