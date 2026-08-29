//! Межвладельческая координация божественной кары.
//!
//! Конкретные проверки, часы, формула и wire-снимок принадлежат skill-owner-ам.
//! Здесь остаются регистрация в регионе и фактическое применение атаки к
//! независимым владельцам.
use super::*;
use crate::gameserver::appserver::skills::godpunishmentphalanx::CGodPunishmentPhalanx;
impl CGame {
    pub(crate) fn add_god_punishment_phalanx<R: GameMainLoopRuntime>(&mut self, region: i32, phalanx: CGodPunishmentPhalanx, x: i32, y: i32, now: u32, runtime: &mut R) -> Option<Result<i32, RegionMembershipBlock>> { if self.find_region(region)?.base().skill_cell_block(x, y) == 2 { return None; } let mut owner = self.take_region_owner(region)?; let result = owner.base_mut().add_god_punishment_phalanx(phalanx, x, y, self.area_width, self.area_height, now, runtime); self.restore_region_owner(owner); Some(result) }
    pub(crate) fn send_god_punishment_phalanx_entry<R: GameMainLoopRuntime>(&mut self, region: i32, id: i32, runtime: &mut R) -> Option<()> { let phalanx = self.find_region(region)?.base().find_skill_phalanx(id)?; let SummonedSkillShape::GodPunishment(phalanx) = phalanx else { return None }; let identity = phalanx.shape().identity(); let x = phalanx.shape().get_tile_x().ok()?; let y = phalanx.shape().get_tile_y().ok()?; let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?; let mut message = CMessage::new(0x000b_f502); message.add_long(identity.object_type); message.add_long(identity.id); message.base_mut().add_guid(identity.ex_id); message.add_long(i32::try_from(payload.len()).ok()?); message.base_mut().add(&payload); message.base_mut().add_char(0); let _ = self.send_shape_position_around(region, x, y, &message); Some(()) }
}
