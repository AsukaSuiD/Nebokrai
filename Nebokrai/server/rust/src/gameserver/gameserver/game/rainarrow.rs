//! Региональное исполнение дождя стрел.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/rainarrow.cpp,
//! rainarrowphalanx.cpp и совместимый ThunderBlowPhalanx::Begin.
//! Форма остаётся в арене во время контактов. Каждый луч получает собственный
//! снимок GetShapes и останавливается после всех допущенных целей клетки.
//! End по дальности не прерывает AI; только истечение срока даёт ранний выход.
//! SetTileXY и завершение прежнего ThunderBlow предшествуют Add; его отказ
//! не отменяет сериализацию и попытку публикации новой формы.

use super::*;
use crate::gameserver::appserver::skills::rainarrowphalanx::{
    CRainArrowPhalanx, RainArrowBeam, apply_rain_arrow_attack,
};

impl CGame {
    pub(crate) fn spawn_rain_arrow_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region: i32, mut phalanx: CRainArrowPhalanx,
        x: i32, y: i32, started_at_ms: u32, runtime: &mut Runtime,
    ) -> Option<()> {
        self.find_region(region)?;
        phalanx.shape_mut().set_pos_xy_base(x as f32 + 0.5, y as f32 + 0.5);
        self.end_overlapping_thunder_blow(region, x, y);
        let mut owner = self.take_region_owner(region)?;
        let result = owner.base_mut().add_rain_arrow_phalanx(
            phalanx, self.area_width, self.area_height, started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        let failed;
        let phalanx = match result {
            Ok(id) => self.rain_arrow_phalanx(region, id)?,
            Err((_, phalanx)) => { failed = phalanx; &failed }
        };
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let shape = phalanx.shape().clone();
        self.publish_summoned_shape_entry(&shape, &payload)
    }

    fn rain_arrow_phalanx(&self, region: i32, id: i32) -> Option<&CRainArrowPhalanx> {
        let SummonedSkillShape::RainArrow(phalanx) =
            self.find_region(region)?.base().find_skill_phalanx(id)?
        else { return None; };
        Some(phalanx)
    }

    fn rain_arrow_phalanx_mut(&mut self, region: i32, id: i32) -> Option<&mut CRainArrowPhalanx> {
        let SummonedSkillShape::RainArrow(phalanx) =
            self.find_region_mut(region)?.base_mut().find_skill_phalanx_mut(id)?
        else { return None; };
        Some(phalanx)
    }

    pub(super) fn run_rain_arrow_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region: i32, id: i32, runtime: &mut Runtime,
    ) -> bool {
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.rain_arrow_phalanx(region, id) else { return false; };
        if phalanx.expired_at(now) {
            self.end_summoned_shape(region, id);
            return true;
        }
        if phalanx.maximum_distance_exceeded() {
            self.end_summoned_shape(region, id);
        }
        let Some(phalanx) = self.rain_arrow_phalanx(region, id) else { return false; };
        let step = phalanx.current_step();
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.rain_arrow_phalanx(region, id) else { return false; };
        if !phalanx.attack_due_at(step, now) { return true; }
        for beam in [RainArrowBeam::Center, RainArrowBeam::Right, RainArrowBeam::Left] {
            let Some(phalanx) = self.rain_arrow_phalanx(region, id) else { return false; };
            let beam_step = if beam == RainArrowBeam::Center { step } else { phalanx.current_step() };
            if let Some((x, y)) = phalanx.beam_cell(beam, beam_step)
                && self.apply_rain_arrow_cell(region, id, x, y, runtime)
                && let Some(phalanx) = self.rain_arrow_phalanx_mut(region, id)
            {
                phalanx.stop_beam(beam);
            }
        }
        if let Some(phalanx) = self.rain_arrow_phalanx_mut(region, id) {
            phalanx.advance_step();
        }
        true
    }

    fn apply_rain_arrow_cell<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, x: i32, y: i32, runtime: &mut Runtime,
    ) -> bool {
        let Some(phalanx) = self.rain_arrow_phalanx(holder_region, id) else { return false; };
        if !phalanx.shape().is_assigned_to_server_region() { return false; }
        let region = phalanx.shape().get_region_id();
        let Some(owner) = self.find_region(region) else { return false; };
        let mut shapes = Vec::new();
        let _ = owner.base().get_shapes(
            x, y, self.area_width, self.area_height,
            &RegionShapeResolver { game: self, owner }, &mut shapes,
        );
        let mut attacked = false;
        for target in shapes.into_iter().map(|view| view.identity) {
            let Some(phalanx) = self.rain_arrow_phalanx(holder_region, id) else { return attacked; };
            if !self.summoned_skill_scan_target_allowed(region, phalanx.master(), target) { continue; }
            let snapshot = phalanx.attack_snapshot();
            apply_rain_arrow_attack(self, snapshot, region, target, runtime);
            // Native Attack(target) возвращает void: его ранний IsDied тоже
            // считается контактом клетки и после обхода останавливает луч.
            attacked = true;
        }
        attacked
    }
}
