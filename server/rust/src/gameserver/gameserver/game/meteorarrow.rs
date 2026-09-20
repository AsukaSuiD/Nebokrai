//! Региональное исполнение метеорных стрел.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/meteorarrow.cpp,
//! meteorarrowphalanx.cpp и совместимый CFallingStarPhalanx.
//! Форма остаётся в региональной арене во время попаданий. После проверки
//! частоты отдельные часы записывают last-attack до проверки числа стрел;
//! индекс увеличивается после всех callbacks, даже при отсутствующем регионе.
//! Add не подавляет сериализацию и отправку при отказе регистрации.

use super::*;
use crate::gameserver::appserver::skills::meteorarrowphalanx::{
    CMeteorArrowPhalanx, apply_meteor_arrow_attack,
};

impl CGame {
    pub(crate) fn spawn_meteor_arrow_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region_id: i32, phalanx: CMeteorArrowPhalanx,
        started_at_ms: u32, runtime: &mut Runtime,
    ) -> Option<()> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_meteor_arrow_phalanx(
            phalanx, self.area_width, self.area_height, started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        let failed;
        let phalanx = match result {
            Ok(id) => self.meteor_arrow_phalanx(region_id, id)?,
            Err((_, phalanx)) => { failed = phalanx; &failed }
        };
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let shape = phalanx.shape().clone();
        self.publish_summoned_shape_entry(&shape, &payload)
    }

    fn meteor_arrow_phalanx(&self, region: i32, id: i32) -> Option<&CMeteorArrowPhalanx> {
        let SummonedSkillShape::MeteorArrow(phalanx) =
            self.find_region(region)?.base().find_skill_phalanx(id)?
        else { return None; };
        Some(phalanx)
    }

    fn meteor_arrow_phalanx_mut(&mut self, region: i32, id: i32) -> Option<&mut CMeteorArrowPhalanx> {
        let SummonedSkillShape::MeteorArrow(phalanx) =
            self.find_region_mut(region)?.base_mut().find_skill_phalanx_mut(id)?
        else { return None; };
        Some(phalanx)
    }

    pub(super) fn run_meteor_arrow_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region: i32, id: i32, runtime: &mut Runtime,
    ) -> bool {
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.meteor_arrow_phalanx(region, id) else { return false; };
        if phalanx.expired_at(now) {
            self.end_summoned_shape(region, id);
            return true;
        }
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.meteor_arrow_phalanx(region, id) else { return false; };
        if !phalanx.attack_due_at(now) { return true; }
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.meteor_arrow_phalanx_mut(region, id) else { return false; };
        phalanx.mark_attack_at(now);
        if phalanx.attack_cells_finished() {
            self.end_summoned_shape(region, id);
            return true;
        }
        if let Some((x, y)) = phalanx.current_cell() {
            self.apply_meteor_arrow_cell(region, id, x, y, runtime);
        }
        if let Some(phalanx) = self.meteor_arrow_phalanx_mut(region, id) {
            phalanx.advance_attack_cell();
        }
        true
    }

    fn apply_meteor_arrow_cell<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, x: i32, y: i32, runtime: &mut Runtime,
    ) {
        let Some(phalanx) = self.meteor_arrow_phalanx(holder_region, id) else { return; };
        if !phalanx.shape().is_assigned_to_server_region() { return; }
        let region = phalanx.shape().get_region_id();
        let Some(owner) = self.find_region(region) else { return; };
        let mut shapes = Vec::new();
        let _ = owner.base().get_shapes(
            x, y, self.area_width, self.area_height,
            &RegionShapeResolver { game: self, owner }, &mut shapes,
        );
        for target in shapes.into_iter().map(|view| view.identity) {
            let Some(phalanx) = self.meteor_arrow_phalanx(holder_region, id) else { return; };
            if !self.summoned_skill_scan_target_allowed(region, phalanx.master(), target) { continue; }
            let snapshot = phalanx.attack_snapshot();
            apply_meteor_arrow_attack(self, snapshot, region, target, runtime);
        }
    }
}
