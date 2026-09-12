//! Межвладельческая координация сферы хаоса.
//!
//! Cast, путь, часы, маска и формула принадлежат `chaossphere.rs` и
//! `chaosspherephalanx.rs`. Здесь остаются регистрация в регионе,
//! пространственный `ForceMove` и применение атак к независимым владельцам.

use super::*;
use crate::gameserver::appserver::skills::chaosspherephalanx::CChaosSpherePhalanx;

impl CGame {
    pub(crate) fn add_chaos_sphere_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx: CChaosSpherePhalanx,
        tile_x: i32,
        tile_y: i32,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_chaos_sphere_phalanx(
            phalanx, tile_x, tile_y, self.area_width, self.area_height,
            started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        Some(result)
    }

    pub(crate) fn send_chaos_sphere_phalanx_entry<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx_id: i32,
        runtime: &mut Runtime,
    ) -> Option<()> {
        let phalanx = self.find_region(region_id)?.base().find_skill_phalanx(phalanx_id)?;
        let SummonedSkillShape::ChaosSphere(phalanx) = phalanx else { return None };
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


}
