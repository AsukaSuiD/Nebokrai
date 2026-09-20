//! Живой региональный цикл сферы хаоса.
//! Источник: gameserver.exe/GameServer.pdb, chaosspherephalanx.cpp.
//! ForceMove выполняется до записи первых часов движения. Центр поражения
//! следует пути независимо от видимой позиции. Каждый scan фиксирует начало
//! квадрата, обходит X→Y и доставляет WarSoul до body той же клетки.
//! Body-dedup пополняется после Attack, включая его ранний IsDied; отказ
//! допуска цель не запоминает. Add не отменяет encoder при ошибке регистрации.

use super::*;
use crate::gameserver::appserver::skills::chaosspherephalanx::CChaosSpherePhalanx;
use crate::gameserver::appserver::skills::elementphalanxattack::{
    apply_element_phalanx_attack, apply_element_phalanx_war_soul,
};

impl CGame {
    pub(crate) fn add_chaos_sphere_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region: i32, phalanx: CChaosSpherePhalanx,
        started_at_ms: u32, runtime: &mut Runtime,
    ) -> Option<()> {
        let mut owner = self.take_region_owner(region)?;
        let result = owner.base_mut().add_chaos_sphere_phalanx(
            phalanx, self.area_width, self.area_height, started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        let failed;
        let phalanx = match result {
            Ok(id) => self.chaos_sphere_phalanx(region, id)?,
            Err((_, phalanx)) => { failed = phalanx; &failed }
        };
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let shape = phalanx.shape().clone();
        self.publish_summoned_shape_entry(&shape, &payload)
    }

    fn chaos_sphere_phalanx(&self, region: i32, id: i32) -> Option<&CChaosSpherePhalanx> {
        let SummonedSkillShape::ChaosSphere(phalanx) =
            self.find_region(region)?.base().find_skill_phalanx(id)?
        else { return None; };
        Some(phalanx)
    }

    fn chaos_sphere_phalanx_mut(&mut self, region: i32, id: i32) -> Option<&mut CChaosSpherePhalanx> {
        let SummonedSkillShape::ChaosSphere(phalanx) =
            self.find_region_mut(region)?.base_mut().find_skill_phalanx_mut(id)?
        else { return None; };
        Some(phalanx)
    }

    pub(super) fn run_chaos_sphere_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, runtime: &mut Runtime,
    ) -> bool {
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.chaos_sphere_phalanx(holder_region, id) else { return false; };
        if phalanx.expired_at(now) {
            self.end_summoned_shape(holder_region, id);
            return true;
        }
        if !phalanx.has_path() { return true; }
        if let Some((x, y, duration)) = phalanx.initial_force_move() {
            self.force_move_summoned_shape(holder_region, id, x, y, duration);
            let now = runtime.now_milliseconds();
            let Some(phalanx) = self.chaos_sphere_phalanx_mut(holder_region, id) else { return false; };
            phalanx.mark_force_moved_at(now);
        }
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.chaos_sphere_phalanx(holder_region, id) else { return false; };
        if phalanx.movement_due_at(now) {
            let now = runtime.now_milliseconds();
            let Some(phalanx) = self.chaos_sphere_phalanx_mut(holder_region, id) else { return false; };
            phalanx.advance_at(now);
        }
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.chaos_sphere_phalanx(holder_region, id) else { return false; };
        if !phalanx.attack_due_at(now) { return true; }
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.chaos_sphere_phalanx_mut(holder_region, id) else { return false; };
        phalanx.mark_attack_at(now);
        let region = phalanx.shape().is_assigned_to_server_region()
            .then(|| phalanx.shape().get_region_id());
        if let Some(region) = region.filter(|region| self.find_region(*region).is_some()) {
            self.apply_chaos_sphere_area(holder_region, id, region, runtime);
        }
        true
    }

    fn apply_chaos_sphere_area<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, region: i32, runtime: &mut Runtime,
    ) {
        let Some((origin_x, origin_y)) = self.chaos_sphere_phalanx(holder_region, id)
            .and_then(CChaosSpherePhalanx::attack_origin)
        else { return; };
        let mut attacked = Vec::new();
        for dx in 0..3 {
            for dy in 0..3 {
                let x = origin_x.wrapping_add(dx);
                let y = origin_y.wrapping_add(dy);
                let players: Vec<_> = self.find_region(region)
                    .map(|owner| owner.base().war_souls_at(x, y).keys().copied().collect())
                    .unwrap_or_default();
                for target_id in players {
                    let Some(snapshot) = self.chaos_sphere_phalanx(holder_region, id)
                        .map(CChaosSpherePhalanx::attack_snapshot)
                    else { return; };
                    apply_element_phalanx_war_soul(self, snapshot, target_id as i32, runtime);
                }
                let Some(owner) = self.find_region(region) else { return; };
                let mut targets = Vec::new();
                let _ = owner.base().get_shapes(
                    x, y, self.area_width, self.area_height,
                    &RegionShapeResolver { game: self, owner }, &mut targets,
                );
                for target in targets.into_iter().map(|view| view.identity) {
                    let Some(phalanx) = self.chaos_sphere_phalanx(holder_region, id) else { return; };
                    if target == phalanx.shape().identity() { continue; }
                    let Some(shape) = resolve_state_move_shape(self, region, target) else { continue; };
                    let target_region = shape.shape().get_region_id();
                    let master = phalanx.master();
                    if target.object_type == master.master_type && target.id == master.master_id { continue; }
                    if attacked.contains(&target) { continue; }
                    if master.master_type == PLAYER_TYPE {
                        let Some(source) = self.find_player(master.master_id)
                            .map(|player| (player.shape().get_region_id(), player.shape().identity()))
                        else { continue; };
                        if !self.live_skill_target_attackable_between(source, (target_region, target)) {
                            continue;
                        }
                    }
                    let Some(snapshot) = self.chaos_sphere_phalanx(holder_region, id)
                        .map(CChaosSpherePhalanx::attack_snapshot)
                    else { return; };
                    apply_element_phalanx_attack(self, snapshot, (target_region, target), false, runtime);
                    attacked.push(target);
                }
            }
        }
    }
}
