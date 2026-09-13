//! Живой обход однократных областей инь-ян.
//! Источник: gameserver.exe/GameServer.pdb, yinyangphalanx{,2}.cpp, AI/Attack.
//! Координаты начала фиксируются перед X→Y; маска читается перед каждой
//! клеткой. Дедупликация предшествует FindPlayer и допуску. Неигровой Master
//! пропускает допуск, а отсутствие Player при Calculate не отменяет receipt.
//! Область остаётся в арене до конца обхода. Add не отменяет входной пакет
//! при отказе регистрации и не выполняет синтетическое замещение соседей.

use super::*;
use crate::gameserver::appserver::skills::elementphalanxattack::apply_element_phalanx_attack;
use crate::gameserver::appserver::skills::yinyangphalanx::CYinYangPhalanx;

impl CGame {
    pub(crate) fn add_yin_yang_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region: i32, phalanx: CYinYangPhalanx,
        started_at_ms: u32, runtime: &mut Runtime,
    ) -> Option<()> {
        let mut owner = self.take_region_owner(region)?;
        let result = owner.base_mut().add_yin_yang_phalanx(
            phalanx, self.area_width, self.area_height, started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        let failed;
        let phalanx = match result {
            Ok(id) => self.yin_yang_phalanx(region, id)?,
            Err((_, phalanx)) => { failed = phalanx; &failed }
        };
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let shape = phalanx.shape().clone();
        self.publish_summoned_shape_entry(&shape, &payload)
    }

    fn yin_yang_phalanx(&self, region: i32, id: i32) -> Option<&CYinYangPhalanx> {
        let SummonedSkillShape::YinYang(phalanx) =
            self.find_region(region)?.base().find_skill_phalanx(id)?
        else { return None; };
        Some(phalanx)
    }

    pub(super) fn run_yin_yang_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, runtime: &mut Runtime,
    ) -> bool {
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.yin_yang_phalanx(holder_region, id) else { return false; };
        if !phalanx.expired_at(now) { return true; }
        let region = phalanx.shape().is_assigned_to_server_region()
            .then(|| phalanx.shape().get_region_id());
        if let Some(region) = region.filter(|region| self.find_region(*region).is_some()) {
            self.apply_yin_yang_area(holder_region, id, region, runtime);
        }
        self.end_summoned_shape(holder_region, id);
        true
    }

    fn apply_yin_yang_area<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, region: i32, runtime: &mut Runtime,
    ) {
        let Some(phalanx) = self.yin_yang_phalanx(holder_region, id) else { return; };
        let (origin_x, origin_y) = phalanx.origin();
        let (length, height) = phalanx.dimensions();
        let mut attacked = Vec::new();
        for x in 0..length {
            for y in 0..height {
                let Some(phalanx) = self.yin_yang_phalanx(holder_region, id) else { return; };
                if !phalanx.cell_active(x, y) { continue; }
                let Some(owner) = self.find_region(region) else { return; };
                let mut targets = Vec::new();
                let _ = owner.base().get_shapes(
                    origin_x.wrapping_add(x), origin_y.wrapping_add(y),
                    self.area_width, self.area_height,
                    &RegionShapeResolver { game: self, owner }, &mut targets,
                );
                for target in targets.into_iter().map(|view| view.identity) {
                    let Some(phalanx) = self.yin_yang_phalanx(holder_region, id) else { return; };
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
                    let Some(snapshot) = self.yin_yang_phalanx(holder_region, id)
                        .map(CYinYangPhalanx::attack_snapshot)
                    else { return; };
                    apply_element_phalanx_attack(self, snapshot, (target_region, target), false, runtime);
                    attacked.push(target);
                }
            }
        }
    }
}
