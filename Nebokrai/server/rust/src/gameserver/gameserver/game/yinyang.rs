//! Межвладельческая координация области инь-ян.
//!
//! Применение навыка, маска, замещение, срок жизни и формула принадлежат `yinyang.rs` и
//! `yinyangphalanx.rs`. Здесь остаются регистрация в регионе и фактическое
//! применение атаки к независимым владельцам игроков и монстров.

use super::*;
use crate::gameserver::appserver::skills::yinyangphalanx::CYinYangPhalanx;

impl CGame {
    pub(crate) fn add_yin_yang_phalanx<Runtime: GameMainLoopRuntime>(&mut self, region_id: i32, phalanx: CYinYangPhalanx, tile_x: i32, tile_y: i32, started_at_ms: u32, runtime: &mut Runtime) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_yin_yang_phalanx(phalanx, tile_x, tile_y, self.area_width, self.area_height, started_at_ms, runtime);
        self.restore_region_owner(owner);
        Some(result)
    }

    pub(crate) fn send_yin_yang_phalanx_entry(&mut self, region_id: i32, phalanx_id: i32) -> Option<()> {
        let phalanx = self.find_region(region_id)?.base().find_skill_phalanx(phalanx_id)?;
        let SummonedSkillShape::YinYang(phalanx) = phalanx else { return None };
        let identity = phalanx.shape().identity();
        let tile_x = phalanx.shape().get_tile_x().ok()?;
        let tile_y = phalanx.shape().get_tile_y().ok()?;
        let payload = phalanx.encode_client_snapshot()?;
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
