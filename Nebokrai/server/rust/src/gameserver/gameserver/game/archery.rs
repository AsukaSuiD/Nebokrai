//! Региональный жизненный цикл базового снаряда Archery.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/archery.cpp
//! и archeryphalanx.cpp. Summon задаёт клетку до свежего региона источника,
//! затем вызывает Add без отдельной сериализации или BF502.
//! Часы срока жизни и задержки читаются отдельно. Цель разрешается через
//! GetObject фактического региона, а форма остаётся опубликованной во время
//! контакта. Отметка удаления следует за Attack, даже при NULL регионе/цели;
//! собственный End Archery не отправляет немедленный пакет выхода.

use super::*;
use crate::gameserver::appserver::skills::archeryphalanx::{
    CArcheryPhalanx, apply_archery_attack,
};

impl CGame {
    pub(crate) fn spawn_archery_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, source: (i32, ShapeIdentity), mut phalanx: CArcheryPhalanx,
        x: i32, y: i32, started_at_ms: u32, runtime: &mut Runtime,
    ) -> Option<()> {
        phalanx.shape_mut().set_pos_xy_base(x as f32 + 0.5, y as f32 + 0.5);
        let source = resolve_state_move_shape(self, source.0, source.1)?;
        if !source.shape().is_assigned_to_server_region() { return None; }
        let region = source.shape().get_region_id();
        let mut owner = self.take_region_owner(region)?;
        let _ = owner.base_mut().add_archery_phalanx(
            phalanx, self.area_width, self.area_height, started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        Some(())
    }

    pub(super) fn archery_phalanx(&self, region: i32, id: i32) -> Option<&CArcheryPhalanx> {
        let SummonedSkillShape::Archery(phalanx) =
            self.find_region(region)?.base().find_skill_phalanx(id)?
        else { return None; };
        Some(phalanx)
    }

    pub(super) fn end_archery_phalanx(&mut self, region: i32, id: i32) {
        if let Some(SummonedSkillShape::Archery(phalanx)) = self.find_region_mut(region)
            .and_then(|owner| owner.base_mut().find_skill_phalanx_mut(id))
        { phalanx.end(); }
    }

    pub(super) fn run_archery_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, runtime: &mut Runtime,
    ) -> bool {
        let lifetime_now = runtime.now_milliseconds();
        let Some(phalanx) = self.archery_phalanx(holder_region, id) else { return false; };
        if phalanx.expired_at(lifetime_now) {
            self.end_archery_phalanx(holder_region, id);
            return true;
        }
        let attack_now = runtime.now_milliseconds();
        let Some(phalanx) = self.archery_phalanx(holder_region, id) else { return false; };
        if !phalanx.attack_due_at(attack_now) { return true; }
        if phalanx.shape().is_assigned_to_server_region() {
            let region = phalanx.shape().get_region_id();
            let target = phalanx.target();
            if self.find_region(region).is_some_and(|owner| owner.base().has_registered_shape(target))
                && resolve_state_move_shape(self, region, target).is_some()
            {
                let snapshot = phalanx.attack_snapshot();
                apply_archery_attack(self, snapshot, (region, target), runtime);
            }
        }
        self.end_archery_phalanx(holder_region, id);
        true
    }
}
