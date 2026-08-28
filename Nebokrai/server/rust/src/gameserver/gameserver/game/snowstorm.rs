//! Межвладельческая координация снежной бури.
//!
//! Проверки, RNG, формула, окна и снимок принадлежат `snowstorm.rs` и
//! `snowstormphalanx.rs`. Здесь остаются регистрация в регионе, разрешение
//! упорядоченных целей и фактическая доставка между независимыми владельцами.

use super::*;

impl CGame {
    pub(crate) fn add_snow_storm_phalanx<Runtime: GameMainLoopRuntime>(&mut self, region_id: i32, mut phalanx: CSnowStormPhalanx, tile_x: i32, tile_y: i32, started_at_ms: u32, runtime: &mut Runtime) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut random_below = |maximum| game_legacy_random(&mut self.random_state, maximum);
        if !phalanx.initialize(tile_x, tile_y, &mut random_below) { return None; }
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_snow_storm_phalanx(phalanx, tile_x, tile_y, self.area_width, self.area_height, started_at_ms, runtime);
        self.restore_region_owner(owner);
        Some(result)
    }

    pub(crate) fn send_snow_storm_phalanx_entry<Runtime: GameMainLoopRuntime>(&mut self, region_id: i32, phalanx_id: i32, runtime: &mut Runtime) -> Option<()> {
        let phalanx = self.find_region(region_id)?.base().find_skill_phalanx(phalanx_id)?;
        let SummonedSkillShape::SnowStorm(phalanx) = phalanx else { return None; };
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

    pub(super) fn snow_storm_targets(&self, region_id: i32, phalanx: &CSnowStormPhalanx) -> Vec<ShapeIdentity> {
        let Some(region) = self.find_region(region_id).map(ServerRegionOwner::base) else { return Vec::new(); };
        let mut targets = Vec::new();
        for (x, y) in phalanx.current_cells() {
            let mut shapes = Vec::new();
            if region.get_shapes(x, y, self.area_width, self.area_height, self, &mut shapes).is_err() { continue; }
            for shape in shapes {
                if shape.identity == phalanx.shape().identity()
                    || (shape.identity.object_type == phalanx.master().master_type && shape.identity.id == phalanx.master().master_id)
                    || !matches!(shape.identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
                { continue; }
                targets.push(shape.identity);
            }
        }
        targets
    }

    pub(super) fn calculate_snow_storm_attack(&mut self, phalanx: &CSnowStormPhalanx) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
        let master = phalanx.master();
        let player = self.find_player(master.master_id)?;
        let combat = player.combat_properties();
        let occupation = player.occupation();
        let level = player.level();
        let mut random_below = |maximum| game_legacy_random(&mut self.random_state, maximum);
        Some(phalanx.calculate_attack(combat, occupation, level, &mut random_below))
    }
}
