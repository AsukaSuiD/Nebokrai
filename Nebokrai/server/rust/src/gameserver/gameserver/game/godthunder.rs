//! Межвладельческая координация божественного грома.
//!
//! Skill-owner и владелец формы хранят проверки, RNG-окна, формулу и lifecycle.
//! Здесь остаются регистрация в регионе, чтение упорядоченного пространственного
//! индекса, разрешение независимых владельцев и фактическая доставка.

use super::*;
use crate::gameserver::appserver::skills::godthunderphalanx::CGodThunderPhalanx;

impl CGame {
    pub(crate) fn add_god_thunder_phalanx<R: GameMainLoopRuntime>(
        &mut self, region_id: i32, mut phalanx: CGodThunderPhalanx,
        x: i32, y: i32, now_ms: u32, runtime: &mut R,
    ) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut random = |maximum| game_legacy_random(&mut self.random_state, maximum);
        phalanx.initialize(x, y, &mut random);
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_god_thunder_phalanx(
            phalanx, x, y, self.area_width, self.area_height, now_ms, runtime,
        );
        self.restore_region_owner(owner);
        Some(result)
    }

    pub(crate) fn send_god_thunder_phalanx_entry<R: GameMainLoopRuntime>(
        &mut self, region_id: i32, id: i32, runtime: &mut R,
    ) -> Option<()> {
        let phalanx = self.find_region(region_id)?.base().find_skill_phalanx(id)?;
        let SummonedSkillShape::GodThunder(phalanx) = phalanx else { return None };
        let identity = phalanx.shape().identity();
        let x = phalanx.shape().get_tile_x().ok()?;
        let y = phalanx.shape().get_tile_y().ok()?;
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let mut message = CMessage::new(0x000b_f502);
        message.add_long(identity.object_type); message.add_long(identity.id);
        message.base_mut().add_guid(identity.ex_id);
        message.add_long(i32::try_from(payload.len()).ok()?);
        message.base_mut().add(&payload); message.base_mut().add_char(0);
        let _ = self.send_shape_position_around(region_id, x, y, &message);
        Some(())
    }

    pub(super) fn god_thunder_targets(
        &self, region_id: i32, phalanx: &CGodThunderPhalanx,
    ) -> Vec<ShapeIdentity> {
        let Some(region) = self.find_region(region_id).map(ServerRegionOwner::base) else { return Vec::new() };
        let mut targets = Vec::new();
        for (x, y) in phalanx.attack_cells() {
            let mut shapes = Vec::new();
            if region.get_shapes(x, y, self.area_width, self.area_height, self, &mut shapes).is_err() { continue; }
            for shape in shapes {
                let identity = shape.identity;
                if identity != phalanx.shape().identity()
                    && !(identity.object_type == phalanx.master().master_type && identity.id == phalanx.master().master_id)
                    && matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
                    && self.owned_player_skill_target_attackable(phalanx.master(), identity, region_id)
                {
                    targets.push(identity);
                }
            }
        }
        targets
    }

    pub(super) fn calculate_god_thunder_attack(
        &mut self, phalanx: &CGodThunderPhalanx, target_level: u8,
    ) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
        let player = self.find_player(phalanx.master().master_id)?;
        let combat = player.combat_properties();
        let occupation = player.occupation();
        let level = player.level();
        let weapon = player.equipment().get_goods(2).map_or(0, |goods| {
            goods.addon_property_value(self.goods_factory(), GAP_WEAPON_DAMAGE_LEVEL, 1)
        });
        let (divisor, minimum) = self.globe_setup.weapon_damage_factors();
        let delta = weapon.wrapping_sub(i32::from(target_level)).max(0);
        let factor = if divisor == 0.0 { 1.0 } else { (delta as f32 / divisor).min(1.0).max(minimum) };
        let critical_rate = self.globe_setup.critical_rate();
        let mut random = |maximum| game_legacy_random(&mut self.random_state, maximum);
        Some(phalanx.calculate_attack(combat, occupation, level, factor, critical_rate, &mut random))
    }
}
