//! Региональное исполнение световой стрелы.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/lightingarrow.cpp,
//! lightingarrowphalanx.cpp и ThunderBlowPhalanx::Begin. Форма остаётся
//! в единственном хранилище во время Attack и callbacks. Следующая клетка,
//! End и флаг ForceMove изменяются после соответствующих вызовов.
//! Перед Add прежние формы 13F из снимка лицевой клетки получают Begin:
//! совпадение их живых координат вызывает полный End с сообщением удаления.
//! Результат Add не отменяет сериализацию и отправку новой формы.

use super::*;
use crate::gameserver::appserver::skills::lightingarrowphalanx::{
    CLightingArrowPhalanx, apply_lighting_arrow_attack, lighting_arrow_targets,
};

impl CGame {
    pub(crate) fn spawn_lighting_arrow_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region_id: i32, mut phalanx: CLightingArrowPhalanx,
        tile_x: i32, tile_y: i32, started_at_ms: u32, runtime: &mut Runtime,
    ) -> Option<()> {
        self.find_region(region_id)?;
        phalanx.shape_mut().set_pos_xy_base(tile_x as f32 + 0.5, tile_y as f32 + 0.5);
        self.end_overlapping_thunder_blow(region_id, tile_x, tile_y);
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_lighting_arrow_phalanx(
            phalanx, self.area_width, self.area_height, started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        let failed;
        let phalanx = match result {
            Ok(id) => self.lighting_arrow_phalanx(region_id, id)?,
            Err((_, phalanx)) => { failed = phalanx; &failed }
        };
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let shape = phalanx.shape().clone();
        self.publish_summoned_shape_entry(&shape, &payload)
    }

    fn lighting_arrow_phalanx(&self, region_id: i32, id: i32) -> Option<&CLightingArrowPhalanx> {
        let SummonedSkillShape::LightingArrow(phalanx) =
            self.find_region(region_id)?.base().find_skill_phalanx(id)?
        else { return None; };
        Some(phalanx)
    }

    fn lighting_arrow_phalanx_mut(&mut self, region_id: i32, id: i32) -> Option<&mut CLightingArrowPhalanx> {
        let SummonedSkillShape::LightingArrow(phalanx) =
            self.find_region_mut(region_id)?.base_mut().find_skill_phalanx_mut(id)?
        else { return None; };
        Some(phalanx)
    }

    pub(super) fn run_lighting_arrow_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region_id: i32, id: i32, runtime: &mut Runtime,
    ) -> bool {
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.lighting_arrow_phalanx(region_id, id) else { return false; };
        if phalanx.expired_at(now) {
            self.end_summoned_shape(region_id, id);
            return true;
        }
        let Some(phalanx) = self.lighting_arrow_phalanx_mut(region_id, id) else { return false; };
        let current = phalanx.prepare_attack_cells();
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.lighting_arrow_phalanx(region_id, id) else { return false; };
        if let Some((x, y)) = phalanx.due_attack_cell(current, now) {
            self.apply_lighting_arrow_cell(region_id, id, x, y, runtime);
            let Some(phalanx) = self.lighting_arrow_phalanx_mut(region_id, id) else { return false; };
            phalanx.advance_attack_cell();
        }
        let Some(phalanx) = self.lighting_arrow_phalanx(region_id, id) else { return false; };
        if phalanx.attack_cells_finished() {
            self.end_summoned_shape(region_id, id);
        } else if let Some((x, y, duration)) = phalanx.force_move_destination() {
            let _ = self.force_move_summoned_shape(region_id, id, x, y, duration);
            if let Some(phalanx) = self.lighting_arrow_phalanx_mut(region_id, id) {
                phalanx.mark_force_moved();
            }
        }
        true
    }

    fn apply_lighting_arrow_cell<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, x: i32, y: i32, runtime: &mut Runtime,
    ) {
        let Some(snapshot) = self.lighting_arrow_phalanx(holder_region, id).cloned() else { return; };
        if !snapshot.shape().is_assigned_to_server_region() { return; }
        let region = snapshot.shape().get_region_id();
        let targets = lighting_arrow_targets(self, region, x, y);
        for target in targets {
            let Some(current) = self.lighting_arrow_phalanx(holder_region, id) else { return; };
            if current.was_attacked(region, target)
                || !self.summoned_skill_scan_target_allowed(region, current.master(), target)
            { continue; }
            // Attack заново проверяет смерть и тот же список после IsAttackAble.
            if self.move_shape_health(region, target).is_none_or(|hp| hp == 0) { continue; }
            let Some(current) = self.lighting_arrow_phalanx_mut(holder_region, id) else { return; };
            if !current.mark_attacked(region, target) { continue; }
            let snapshot = current.clone();
            apply_lighting_arrow_attack(self, &snapshot, region, target, runtime);
        }
    }
}
