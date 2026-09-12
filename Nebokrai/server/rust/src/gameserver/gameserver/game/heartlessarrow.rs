//! Живое региональное исполнение HeartLessArrowPhalanx2/3.
//! Источник: gameserver.exe/GameServer.pdb, одноимённые владельцы skills.
//! После единственного чтения часов берётся фактический регион формы и
//! снимок её клетки. Допуск и Attack выполняются последовательно при
//! опубликованной форме; каждый допущенный контакт вызывает полный End,
//! даже если Attack вернулся по IsDied. End не завершает обход снимка.

use super::*;
use crate::gameserver::appserver::skills::heartlessarrowphalanx2::{
    CHeartlessArrowPhalanx, apply_heartless_arrow_attack,
};

impl CGame {
    pub(super) fn heartless_arrow_phalanx(&self, region: i32, id: i32) -> Option<&CHeartlessArrowPhalanx> {
        let SummonedSkillShape::HeartlessArrow(phalanx) =
            self.find_region(region)?.base().find_skill_phalanx(id)?
        else { return None; };
        Some(phalanx)
    }

    pub(super) fn run_heartless_arrow_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, runtime: &mut Runtime,
    ) -> bool {
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.heartless_arrow_phalanx(holder_region, id) else { return false; };
        if phalanx.expired_at(now) {
            self.end_summoned_shape(holder_region, id);
            return true;
        }
        if !phalanx.shape().is_assigned_to_server_region() { return true; }
        let region = phalanx.shape().get_region_id();
        let Some(owner) = self.find_region(region) else { return true; };
        let y = phalanx.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = phalanx.shape().get_tile_x().unwrap_or(i32::MIN);
        let mut shapes = Vec::new();
        let _ = owner.base().get_shapes(
            x, y, self.area_width, self.area_height,
            &RegionShapeResolver { game: self, owner }, &mut shapes,
        );
        for target in shapes.into_iter().map(|view| view.identity) {
            let Some(phalanx) = self.heartless_arrow_phalanx(holder_region, id) else { return false; };
            let identity = phalanx.shape().identity();
            if (target.object_type == identity.object_type && target.id == identity.id)
                || !self.summoned_skill_scan_target_allowed(region, phalanx.master(), target)
            { continue; }
            let snapshot = phalanx.attack_snapshot();
            apply_heartless_arrow_attack(self, snapshot, region, target, runtime);
            self.end_summoned_shape(holder_region, id);
        }
        true
    }
}
