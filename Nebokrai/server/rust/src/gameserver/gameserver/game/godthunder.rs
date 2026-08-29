//! Межвладельческая координация божественного грома.
//!
//! Skill-owner и владелец формы хранят проверки, RNG-окна, формулу, выбор целей
//! и lifecycle. Здесь остаются регистрация в регионе, разрешение независимых
//! владельцев и фактическая доставка.

use super::*;
use crate::gameserver::appserver::skills::godthunderphalanx::CGodThunderPhalanx;
use crate::gameserver::appserver::skills::godthunderphalanx2::CGodThunderPhalanx2;

impl CGame {
    pub(crate) fn add_god_thunder_phalanx<R: GameMainLoopRuntime>(
        &mut self, region_id: i32, phalanx: CGodThunderPhalanx,
        x: i32, y: i32, now_ms: u32, runtime: &mut R,
    ) -> Option<Result<i32, RegionMembershipBlock>> {
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

    pub(crate) fn add_god_thunder_2_phalanx<R: GameMainLoopRuntime>(
        &mut self, region_id: i32, phalanx: CGodThunderPhalanx2,
        x: i32, y: i32, now_ms: u32, runtime: &mut R,
    ) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_god_thunder_2_phalanx(phalanx, x, y, self.area_width, self.area_height, now_ms, runtime);
        self.restore_region_owner(owner); Some(result)
    }
    pub(crate) fn send_god_thunder_2_phalanx_entry<R: GameMainLoopRuntime>(&mut self, region_id: i32, id: i32, runtime: &mut R) -> Option<()> {
        let phalanx = self.find_region(region_id)?.base().find_skill_phalanx(id)?;
        let SummonedSkillShape::GodThunder2(phalanx) = phalanx else { return None };
        let identity = phalanx.shape().identity(); let x = phalanx.shape().get_tile_x().ok()?; let y = phalanx.shape().get_tile_y().ok()?;
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let mut message = CMessage::new(0x000b_f502); message.add_long(identity.object_type); message.add_long(identity.id);
        message.base_mut().add_guid(identity.ex_id); message.add_long(i32::try_from(payload.len()).ok()?);
        message.base_mut().add(&payload); message.base_mut().add_char(0);
        let _ = self.send_shape_position_around(region_id, x, y, &message); Some(())
    }
}
