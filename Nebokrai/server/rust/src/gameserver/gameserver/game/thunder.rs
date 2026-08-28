//! Межвладельческая координация грома боевого духа.
//!
//! Формула, маска и жизненный цикл принадлежат `thunder.rs` и
//! `thunderphalanx.rs`. Здесь остаются регистрация в регионе, разрешение
//! упорядоченных целей через владельца пространства, применение атак к
//! независимым владельцам и фактическая круговая доставка.

use super::*;

impl CGame {
    pub(crate) fn add_thunder_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        mut phalanx: CThunderPhalanx,
        tile_x: i32,
        tile_y: i32,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut random_below = |maximum| game_legacy_random(&mut self.random_state, maximum);
        phalanx.initialize(tile_x, tile_y, &mut random_below);
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_thunder_phalanx(
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

    pub(crate) fn send_thunder_phalanx_entry<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx_id: i32,
        runtime: &mut Runtime,
    ) -> Option<()> {
        let phalanx = self.find_region(region_id)?.base().find_skill_phalanx(phalanx_id)?;
        let SummonedSkillShape::Thunder(phalanx) = phalanx else {
            return None;
        };
        let identity = phalanx.shape().identity();
        let tile_x = phalanx.shape().get_tile_x().ok()?;
        let tile_y = phalanx.shape().get_tile_y().ok()?;
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let mut message = CMessage::new(0x000b_f502);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.base_mut().add_guid(identity.ex_id);
        message.add_long(i32::try_from(payload.len()).ok()?);
        message.base_mut().add(&payload);
        message.base_mut().add_char(0);
        let _ = self.send_shape_position_around(region_id, tile_x, tile_y, &message);
        Some(())
    }

    pub(super) fn thunder_targets(
        &self,
        region_id: i32,
        phalanx: &CThunderPhalanx,
    ) -> Vec<ShapeIdentity> {
        let Some(region) = self.find_region(region_id).map(ServerRegionOwner::base) else {
            return Vec::new();
        };
        let (Ok(center_x), Ok(center_y)) = (
            phalanx.shape().get_tile_x(),
            phalanx.shape().get_tile_y(),
        ) else {
            return Vec::new();
        };
        let mut targets = Vec::new();
        for (offset_x, offset_y) in phalanx.scope_cells() {
            let mut shapes = Vec::new();
            if region
                .get_shapes(
                    center_x.wrapping_add(offset_x),
                    center_y.wrapping_add(offset_y),
                    self.area_width,
                    self.area_height,
                    self,
                    &mut shapes,
                )
                .is_err()
            {
                continue;
            }
            for shape in shapes {
                if shape.identity == phalanx.shape().identity()
                    || (shape.identity.object_type == phalanx.master().master_type
                        && shape.identity.id == phalanx.master().master_id)
                    || !matches!(shape.identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
                    || targets.contains(&shape.identity)
                {
                    continue;
                }
                targets.push(shape.identity);
            }
        }
        targets
    }

    pub(super) fn calculate_thunder_attack(
        &mut self,
        phalanx: &CThunderPhalanx,
        target_level: u8,
    ) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
        let master = phalanx.master();
        if master.master_type != PLAYER_TYPE || master.master_id == 0 {
            return None;
        }
        let player = self.find_player(master.master_id)?;
        let sprite = player
            .war_soul_goods(&self.goods_factory)?
            .addon_property_value(&self.goods_factory, GAP_BF_SPRITE, 1);
        let combat = player.combat_properties();
        let occupation = player.occupation();
        let attacker_level = player.level();
        let weapon_level = player.equipment().get_goods(2).map_or(0, |goods| {
            goods.addon_property_value(&self.goods_factory, GAP_WEAPON_DAMAGE_LEVEL, 1)
        });
        let (weapon_divisor, weapon_minimum) = self.globe_setup.weapon_damage_factors();
        let level_delta = weapon_level.wrapping_sub(i32::from(target_level)).max(0);
        let mut weapon_damage_factor = if weapon_divisor == 0.0 {
            1.0
        } else {
            level_delta as f32 / weapon_divisor
        };
        weapon_damage_factor = weapon_damage_factor.min(1.0).max(weapon_minimum);
        let target_damage_factor = self
            .skill_base_properties(THUNDER_SKILL_ID, phalanx.skill_level())?
            .query_property(THUNDER_TARGET_DAMAGE_FACTOR_PROPERTY);
        let mut random_below = |maximum| game_legacy_random(&mut self.random_state, maximum);
        Some(phalanx.calculate_attack(
            sprite,
            combat,
            occupation,
            attacker_level,
            target_damage_factor,
            weapon_damage_factor,
            &mut random_below,
        ))
    }
}
