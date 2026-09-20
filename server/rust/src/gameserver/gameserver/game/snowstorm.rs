//! Региональная цепочка снежной бури.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/snowstormphalanx.cpp.
//! Область остаётся в живой арене во время попаданий. Региональный источник
//! фиксируется перед клетками, а клетка и текущий счётчик читаются каждый раз.
//! Last-attack записывается до обхода, attack-count — после него, включая
//! отсутствие региона. Повторные цели сохраняются. Add не отменяет encoder.

use super::*;
use crate::gameserver::appserver::skills::snowstormphalanx::{
    CSnowStormPhalanx, SNOW_STORM_SCOPE_AREA, apply_snow_storm_attack,
};
use crate::gameserver::appserver::states::state::resolve_region_move_shape;

impl CGame {
    pub(crate) fn add_snow_storm_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region: i32, phalanx: CSnowStormPhalanx,
        started_at_ms: u32, runtime: &mut Runtime,
    ) -> Option<()> {
        let mut owner = self.take_region_owner(region)?;
        let result = owner.base_mut().add_snow_storm_phalanx(
            phalanx, self.area_width, self.area_height, started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        let failed;
        let phalanx = match result {
            Ok(id) => self.snow_storm_phalanx(region, id)?,
            Err((_, phalanx)) => { failed = phalanx; &failed }
        };
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let shape = phalanx.shape().clone();
        self.publish_summoned_shape_entry(&shape, &payload)
    }

    fn snow_storm_phalanx(&self, region: i32, id: i32) -> Option<&CSnowStormPhalanx> {
        let SummonedSkillShape::SnowStorm(phalanx) =
            self.find_region(region)?.base().find_skill_phalanx(id)?
        else { return None; };
        Some(phalanx)
    }

    fn snow_storm_phalanx_mut(&mut self, region: i32, id: i32) -> Option<&mut CSnowStormPhalanx> {
        let SummonedSkillShape::SnowStorm(phalanx) =
            self.find_region_mut(region)?.base_mut().find_skill_phalanx_mut(id)?
        else { return None; };
        Some(phalanx)
    }

    pub(super) fn run_snow_storm_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, runtime: &mut Runtime,
    ) -> bool {
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.snow_storm_phalanx(holder_region, id) else { return false; };
        if phalanx.expired_at(now) {
            self.end_summoned_shape(holder_region, id);
            return true;
        }
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.snow_storm_phalanx(holder_region, id) else { return false; };
        if !phalanx.attack_due_at(now) { return true; }
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.snow_storm_phalanx_mut(holder_region, id) else { return false; };
        phalanx.mark_attack_at(now);
        let region = phalanx.shape().is_assigned_to_server_region()
            .then(|| phalanx.shape().get_region_id());
        if let Some(region) = region.filter(|region| self.find_region(*region).is_some()) {
            self.apply_snow_storm_window(holder_region, id, region, runtime);
        }
        if let Some(phalanx) = self.snow_storm_phalanx_mut(holder_region, id) {
            phalanx.advance_attack_window();
        }
        true
    }

    fn apply_snow_storm_window<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, region: i32, runtime: &mut Runtime,
    ) {
        let Some(phalanx) = self.snow_storm_phalanx(holder_region, id) else { return; };
        let master = phalanx.master();
        let source = resolve_region_move_shape(self, region, ShapeIdentity {
            object_type: master.master_type, id: master.master_id, ex_id: CGuid::GUID_INVALID,
        }).map(|shape| (shape.shape().get_region_id(), shape.shape().identity()));
        for index in 0..SNOW_STORM_SCOPE_AREA {
            let Some((x, y)) = self.snow_storm_phalanx(holder_region, id)
                .and_then(|phalanx| phalanx.current_cell(index))
            else { break; };
            let Some(owner) = self.find_region(region) else { break; };
            let mut targets = Vec::new();
            let _ = owner.base().get_shapes(
                x, y, self.area_width, self.area_height,
                &RegionShapeResolver { game: self, owner }, &mut targets,
            );
            for target in targets.into_iter().map(|view| view.identity) {
                let Some(phalanx) = self.snow_storm_phalanx(holder_region, id) else { return; };
                if target == phalanx.shape().identity() { continue; }
                let Some(shape) = resolve_state_move_shape(self, region, target) else { continue; };
                let master = phalanx.master();
                if target.object_type == master.master_type && target.id == master.master_id {
                    continue;
                }
                let Some(source) = source else { continue; };
                if !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) { continue; }
                let target_region = shape.shape().get_region_id();
                if !self.live_skill_target_attackable_between(source, (target_region, target)) {
                    continue;
                }
                let Some(snapshot) = self.snow_storm_phalanx(holder_region, id)
                    .map(CSnowStormPhalanx::attack_snapshot)
                else { return; };
                apply_snow_storm_attack(self, snapshot, target_region, target, runtime);
            }
        }
    }
}
