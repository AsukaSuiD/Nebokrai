//! Межвладельческая координация области метеорных стрел.
//!
//! Execution, выбор клеток, часы и формула принадлежат `meteorarrow.rs` и
//! `meteorarrowphalanx.rs`. Здесь остаются регистрация формы, разрешение
//! identity в порядке регионального `GetShape` и применение готового удара.

use super::*;
use crate::gameserver::appserver::skills::meteorarrowphalanx::CMeteorArrowPhalanx;

impl CGame {
    pub(crate) fn add_meteor_arrow_phalanx<Runtime: GameMainLoopRuntime>(&mut self, region_id: i32,
        phalanx: CMeteorArrowPhalanx, tile_x: i32, tile_y: i32, started_at_ms: u32,
        runtime: &mut Runtime) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_meteor_arrow_phalanx(phalanx, tile_x, tile_y,
            self.area_width, self.area_height, started_at_ms, runtime);
        self.restore_region_owner(owner); Some(result)
    }

    pub(crate) fn send_meteor_arrow_phalanx_entry<Runtime: GameMainLoopRuntime>(&mut self,
        region_id: i32, phalanx_id: i32, runtime: &mut Runtime) -> Option<()> {
        let phalanx = self.find_region(region_id)?.base().find_skill_phalanx(phalanx_id)?;
        let SummonedSkillShape::MeteorArrow(phalanx) = phalanx else { return None };
        let identity = phalanx.shape().identity(); let x = phalanx.shape().get_tile_x().ok()?;
        let y = phalanx.shape().get_tile_y().ok()?; let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let mut message = CMessage::new(0x000b_f502); message.add_long(identity.object_type); message.add_long(identity.id);
        message.base_mut().add_guid(identity.ex_id); message.add_long(i32::try_from(payload.len()).ok()?);
        message.base_mut().add(&payload); message.base_mut().add_char(0);
        let _ = self.send_shape_position_around(region_id, x, y, &message); Some(())
    }

    pub(super) fn apply_meteor_arrow_cell<Runtime: GameMainLoopRuntime>(&mut self, region_id: i32,
        phalanx_id: i32, tile_x: i32, tile_y: i32, _sampled_at_ms: u32, runtime: &mut Runtime) {
        let Some(phalanx) = self.find_region(region_id).and_then(|owner| owner.base().find_skill_phalanx(phalanx_id)).cloned() else { return };
        let SummonedSkillShape::MeteorArrow(snapshot) = &phalanx else { return };
        let Some(region) = self.find_region(region_id).map(ServerRegionOwner::base) else { return };
        let mut shapes = Vec::new();
        if region.get_shapes(tile_x, tile_y, self.area_width, self.area_height, self, &mut shapes).is_err() { return }
        for target in shapes.into_iter().map(|view| view.identity) {
            if target == snapshot.shape().identity()
                || !self.summoned_skill_scan_target_allowed(region_id, snapshot.master(), target)
            { continue }
            self.apply_summoned_skill_to_target(&phalanx, target, region_id, false, runtime);
        }
    }
}
