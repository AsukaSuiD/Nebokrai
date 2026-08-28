//! Межвладельческая координация отложенного грома боевого духа.
//!
//! Проверки, формула и жизненный цикл принадлежат `thunder2.rs` и
//! `thunder2phalanx.rs`. Здесь остаются регистрация в регионе, разрешение
//! цели через упорядоченного владельца пространства, применение к независимому
//! владельцу и фактическая круговая доставка.

use super::*;

impl CGame {
    pub(crate) fn add_leiming2_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx: CLeimingPhalanx2,
        tile_x: i32,
        tile_y: i32,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_leiming2_phalanx(
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

    pub(crate) fn send_leiming2_phalanx_entry<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx_id: i32,
        runtime: &mut Runtime,
    ) -> Option<()> {
        let phalanx = self.find_region(region_id)?.base().find_skill_phalanx(phalanx_id)?;
        let SummonedSkillShape::Leiming2(phalanx) = phalanx else {
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

    pub(super) fn leiming2_targets(
        &self,
        region_id: i32,
        phalanx: &CLeimingPhalanx2,
    ) -> Vec<ShapeIdentity> {
        let Some(region) = self.find_region(region_id).map(ServerRegionOwner::base) else {
            return Vec::new();
        };
        let (Ok(tile_x), Ok(tile_y)) = (
            phalanx.shape().get_tile_x(),
            phalanx.shape().get_tile_y(),
        ) else {
            return Vec::new();
        };
        let mut shapes = Vec::new();
        if region
            .get_shapes(
                tile_x,
                tile_y,
                self.area_width,
                self.area_height,
                self,
                &mut shapes,
            )
            .is_err()
        {
            return Vec::new();
        }
        let mut targets = Vec::new();
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
        targets
    }

    pub(super) fn calculate_leiming2_attack(
        &mut self,
        phalanx: &CLeimingPhalanx2,
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
        let target_damage_factor = self
            .skill_base_properties(LEIMING2_SKILL_ID, phalanx.skill_level())?
            .query_property(LEIMING2_TARGET_DAMAGE_FACTOR_PROPERTY);
        let mut random_below = |maximum| game_legacy_random(&mut self.random_state, maximum);
        Some(phalanx.calculate_attack(
            sprite,
            combat,
            occupation,
            attacker_level,
            target_damage_factor,
            &mut random_below,
        ))
    }
}
