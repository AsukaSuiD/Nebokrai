//! Живой региональный обход двух областей божественного грома.
//! Источник: gameserver.exe/GameServer.pdb, godthunderphalanx{,2}.cpp.
//! Три чтения часов разделяют срок, готовность окна и запись last-attack.
//! Номер окна меняется после всех callbacks, включая отсутствие региона.
//! GodThunder2 сначала обходит упорядоченный индекс боевых духов, затем
//! заново читает клетку для обычных фигур. Дедупликации нет; непользовательский
//! master пропускает только обычную проверку допуска, но не Attack/IsDied.

use super::*;
use crate::gameserver::appserver::skills::elementphalanxattack::apply_element_phalanx_attack;
use crate::gameserver::appserver::skills::godthunderphalanx::CGodThunderPhalanx;

impl CGame {
    pub(crate) fn add_god_thunder_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region: i32, phalanx: CGodThunderPhalanx,
        started_at_ms: u32, runtime: &mut Runtime,
    ) -> Option<()> {
        let mut owner = self.take_region_owner(region)?;
        let result = owner.base_mut().add_god_thunder_phalanx(
            phalanx, self.area_width, self.area_height, started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        let failed;
        let phalanx = match result {
            Ok(id) => self.god_thunder_phalanx(region, id)?,
            Err((_, phalanx)) => { failed = phalanx; &failed }
        };
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let shape = phalanx.shape().clone();
        self.publish_summoned_shape_entry(&shape, &payload)
    }

    fn god_thunder_phalanx(&self, region: i32, id: i32) -> Option<&CGodThunderPhalanx> {
        let SummonedSkillShape::GodThunder(phalanx) =
            self.find_region(region)?.base().find_skill_phalanx(id)?
        else { return None; };
        Some(phalanx)
    }

    fn god_thunder_phalanx_mut(&mut self, region: i32, id: i32) -> Option<&mut CGodThunderPhalanx> {
        let SummonedSkillShape::GodThunder(phalanx) =
            self.find_region_mut(region)?.base_mut().find_skill_phalanx_mut(id)?
        else { return None; };
        Some(phalanx)
    }

    pub(super) fn run_god_thunder_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, runtime: &mut Runtime,
    ) -> bool {
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.god_thunder_phalanx(holder_region, id) else { return false; };
        if phalanx.expired_at(now) {
            self.end_summoned_shape(holder_region, id);
            return true;
        }
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.god_thunder_phalanx(holder_region, id) else { return false; };
        if !phalanx.attack_due_at(now) { return true; }
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.god_thunder_phalanx_mut(holder_region, id) else { return false; };
        phalanx.mark_attack_at(now);
        let region = phalanx.shape().is_assigned_to_server_region()
            .then(|| phalanx.shape().get_region_id());
        if let Some(region) = region.filter(|region| self.find_region(*region).is_some()) {
            self.apply_god_thunder_window(holder_region, id, region, runtime);
        }
        if let Some(phalanx) = self.god_thunder_phalanx_mut(holder_region, id) {
            phalanx.advance_attack_window();
        }
        true
    }

    fn apply_god_thunder_window<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, region: i32, runtime: &mut Runtime,
    ) {
        let mut index = 0;
        loop {
            let Some(phalanx) = self.god_thunder_phalanx(holder_region, id) else { return; };
            if index >= phalanx.scope_area() { break; }
            let Some((x, y)) = phalanx.current_cell(index) else { break; };
            if (x, y) == (0, 0) { break; }
            if phalanx.has_war_soul_pass() {
                self.apply_god_thunder_war_souls(holder_region, id, region, x, y, runtime);
            }
            let Some((x, y)) = self.god_thunder_phalanx(holder_region, id)
                .and_then(|phalanx| phalanx.current_cell(index))
            else { break; };
            let Some(owner) = self.find_region(region) else { break; };
            let mut targets = Vec::new();
            let _ = owner.base().get_shapes(
                x, y, self.area_width, self.area_height,
                &RegionShapeResolver { game: self, owner }, &mut targets,
            );
            for target in targets.into_iter().map(|view| view.identity) {
                let Some(phalanx) = self.god_thunder_phalanx(holder_region, id) else { return; };
                if target == phalanx.shape().identity() { continue; }
                let Some(shape) = resolve_state_move_shape(self, region, target) else { continue; };
                let target = (shape.shape().get_region_id(), target);
                let master = phalanx.master();
                if target.1.object_type == master.master_type && target.1.id == master.master_id {
                    continue;
                }
                if master.master_type == PLAYER_TYPE {
                    let Some(source) = self.find_player(master.master_id)
                        .map(|player| (player.shape().get_region_id(), player.shape().identity()))
                    else { continue; };
                    if !self.live_skill_target_attackable_between(source, target) { continue; }
                }
                let Some(snapshot) = self.god_thunder_phalanx(holder_region, id)
                    .map(CGodThunderPhalanx::attack_snapshot)
                else { return; };
                apply_element_phalanx_attack(self, snapshot, target, false, runtime);
            }
            index += 1;
        }
    }

    #[allow(clippy::too_many_arguments, reason = "живая форма и текущая клетка её обхода")]
    fn apply_god_thunder_war_souls<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, region: i32,
        x: i32, y: i32, runtime: &mut Runtime,
    ) {
        let players: Vec<_> = self.find_region(region)
            .map(|owner| owner.base().war_souls_at(x, y).keys().copied().collect())
            .unwrap_or_default();
        for target_id in players {
            let Some(phalanx) = self.god_thunder_phalanx(holder_region, id) else { return; };
            let target = self.find_player(target_id as i32);
            let source = self.find_player(phalanx.master().master_id);
            let (Some(target), Some(source)) = (target, source) else { continue; };
            if target_id as i32 == phalanx.master().master_id { continue; }
            if target.shape().get_action() == 6 || target.is_dead() { continue; }
            let target = (target.shape().get_region_id(), target.shape().identity());
            let source = (source.shape().get_region_id(), source.shape().identity());
            if !self.live_skill_target_attackable_between(source, target) { continue; }
            let Some(snapshot) = self.god_thunder_phalanx(holder_region, id)
                .map(CGodThunderPhalanx::attack_snapshot)
            else { return; };
            apply_element_phalanx_attack(self, snapshot, target, true, runtime);
        }
    }
}
