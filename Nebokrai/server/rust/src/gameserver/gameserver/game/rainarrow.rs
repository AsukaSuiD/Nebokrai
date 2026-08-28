//! Межвладельческая координация дождя стрел.
//!
//! Три пути, остановка лучей, временные границы и формула принадлежат
//! владельцам конкретного навыка.
//! Здесь остаются региональная регистрация, разрешение identity в порядке
//! `GetShape` и применение рассчитанного результата к владельцу цели.

use super::*;
use crate::gameserver::appserver::skills::rainarrowphalanx::{CRainArrowPhalanx, RainArrowBeam};

impl CGame {
    pub(crate) fn add_rain_arrow_phalanx<R: GameMainLoopRuntime>(&mut self, region: i32,
        phalanx: CRainArrowPhalanx, x: i32, y: i32, now: u32, runtime: &mut R)
        -> Option<Result<i32, RegionMembershipBlock>> {
        let mut owner = self.take_region_owner(region)?; let result = owner.base_mut().add_rain_arrow_phalanx(
            phalanx, x, y, self.area_width, self.area_height, now, runtime); self.restore_region_owner(owner); Some(result)
    }
    pub(crate) fn send_rain_arrow_phalanx_entry<R: GameMainLoopRuntime>(&mut self, region: i32,
        id: i32, runtime: &mut R) -> Option<()> {
        let phalanx = self.find_region(region)?.base().find_skill_phalanx(id)?;
        let SummonedSkillShape::RainArrow(phalanx) = phalanx else { return None }; let identity = phalanx.shape().identity();
        let x = phalanx.shape().get_tile_x().ok()?; let y = phalanx.shape().get_tile_y().ok()?;
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?; let mut message = CMessage::new(0x000b_f502);
        message.add_long(identity.object_type); message.add_long(identity.id); message.base_mut().add_guid(identity.ex_id);
        message.add_long(i32::try_from(payload.len()).ok()?); message.base_mut().add(&payload); message.base_mut().add_char(0);
        let _ = self.send_shape_position_around(region, x, y, &message); Some(())
    }
    pub(super) fn apply_rain_arrow_cell<R: GameMainLoopRuntime>(&mut self, region_id: i32,
        phalanx_id: i32, beam: RainArrowBeam, x: i32, y: i32, sampled_at_ms: u32, runtime: &mut R) {
        let Some(phalanx) = self.find_region(region_id).and_then(|r| r.base().find_skill_phalanx(phalanx_id)).cloned() else { return };
        let SummonedSkillShape::RainArrow(snapshot) = &phalanx else { return };
        if snapshot.master().master_type == PLAYER_TYPE && self.find_player(snapshot.master().master_id).is_none() { return }
        let Some(region) = self.find_region(region_id).map(ServerRegionOwner::base) else { return }; let mut shapes = Vec::new();
        if region.get_shapes(x, y, self.area_width, self.area_height, self, &mut shapes).is_err() { return }
        let targets: Vec<_> = shapes.into_iter().map(|view| view.identity).filter(|identity| {
            *identity != snapshot.shape().identity() && !(identity.object_type == snapshot.master().master_type && identity.id == snapshot.master().master_id)
                && matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE) && match identity.object_type {
                    PLAYER_TYPE => self.find_player(identity.id).is_some_and(|p| !p.is_dead())
                        && self.player_base_attackable(snapshot.master().master_id, identity.id),
                    MONSTER_TYPE => self.lighting_arrow_monster_attackable(region_id, snapshot.master(), identity.id), _ => false }
        }).collect();
        let attacked = !targets.is_empty(); for target in targets { match target.object_type {
            PLAYER_TYPE => { self.apply_summoned_skill_to_player(&phalanx, target.id, region_id, false, runtime); }
            MONSTER_TYPE => { self.apply_summoned_skill_to_monster(&phalanx, target.id, region_id, sampled_at_ms, runtime); } _ => {} } }
        if attacked && let Some(mut owner) = self.take_region_owner(region_id) { if let Some(SummonedSkillShape::RainArrow(current)) = owner.base_mut().find_skill_phalanx_mut(phalanx_id) { current.stop_beam(beam); } self.restore_region_owner(owner); }
    }
}
