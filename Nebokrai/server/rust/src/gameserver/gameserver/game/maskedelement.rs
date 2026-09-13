//! Живой обход неподвижных элементальных областей.
//! Источник: gameserver.exe/GameServer.pdb, firewallphalanx.cpp и
//! yinyangphalanx{,2}.cpp, AI/Attack; firewall.cpp, Summon.
//! Координаты начала фиксируются перед X→Y; маска читается перед каждой
//! клеткой. Дедупликация предшествует FindPlayer и допуску. Неигровой Master
//! пропускает допуск, а отсутствие Player при Calculate не отменяет receipt.
//! Стена также запоминает отклонённую допуском цель, инь-ян — только попытку
//! Attack. Область остаётся в арене во время callbacks. Add не отменяет входной
//! пакет при отказе регистрации. Только стена после Add обходит соседние area
//! и заменяет прежние стены со свежим уровнем каста и исходной точкой Summon.

use super::*;
use crate::gameserver::appserver::skills::elementphalanxattack::apply_element_phalanx_attack;
use crate::gameserver::appserver::skills::maskedelementphalanx::MaskedElementPhalanx;
use crate::gameserver::appserver::skills::firewall::FIRE_WALL_SKILL_ID;
use crate::gameserver::appserver::states::skill::RegisteredSkill;

impl CGame {
    pub(crate) fn add_masked_element_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region: i32, phalanx: MaskedElementPhalanx,
        started_at_ms: u32, runtime: &mut Runtime,
        replacement: Option<(RegisteredSkill, (i32, i32))>,
    ) -> Option<()> {
        let mut owner = self.take_region_owner(region)?;
        let result = owner.base_mut().add_masked_element_phalanx(
            phalanx, self.area_width, self.area_height, started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        let shape = match &result {
            Ok(id) => self.masked_element_phalanx(region, *id)?.shape().clone(),
            Err((_, phalanx)) => phalanx.shape().clone(),
        };
        if let Some((instance, point)) = replacement {
            self.replace_fire_wall_neighbors(region, &shape, instance, point);
        }
        let phalanx = match &result {
            Ok(id) => self.masked_element_phalanx(region, *id)?,
            Err((_, phalanx)) => phalanx,
        };
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let shape = phalanx.shape().clone();
        self.publish_summoned_shape_entry(&shape, &payload)
    }

    fn masked_element_phalanx(&self, region: i32, id: i32) -> Option<&MaskedElementPhalanx> {
        let SummonedSkillShape::MaskedElement(phalanx) =
            self.find_region(region)?.base().find_skill_phalanx(id)?
        else { return None; };
        Some(phalanx)
    }

    fn masked_element_phalanx_mut(&mut self, region: i32, id: i32) -> Option<&mut MaskedElementPhalanx> {
        let SummonedSkillShape::MaskedElement(phalanx) =
            self.find_region_mut(region)?.base_mut().find_skill_phalanx_mut(id)?
        else { return None; };
        Some(phalanx)
    }

    fn replace_fire_wall_neighbors(
        &mut self, region: i32, shape: &CShape, instance: RegisteredSkill, point: (i32, i32),
    ) {
        let Some(area) = shape.area_index() else { return; };
        let Some(owner) = self.find_region(region) else { return; };
        let neighbors = owner.base().summon_shapes_around_area(
            area, &RegionShapeResolver { game: self, owner },
        );
        for neighbor in neighbors {
            if neighbor.identity == shape.identity() { continue; }
            if !self.masked_element_phalanx(region, neighbor.identity.id)
                .is_some_and(|wall| wall.skill_id() == FIRE_WALL_SKILL_ID)
            { continue; }
            let Some(level) = self.registered_skill(instance).map(|skill| i32::from(skill.level())) else { return; };
            if let Some(wall) = self.masked_element_phalanx_mut(region, neighbor.identity.id) {
                wall.replace_affect_region(level, point.0, point.1);
            }
        }
    }

    pub(super) fn run_masked_element_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, runtime: &mut Runtime,
    ) -> bool {
        let now = runtime.now_milliseconds();
        let Some(phalanx) = self.masked_element_phalanx(holder_region, id) else { return false; };
        let periodic = phalanx.is_periodic();
        if periodic {
            if phalanx.expired_at(now) {
                self.end_summoned_shape(holder_region, id);
                return true;
            }
            let now = runtime.now_milliseconds();
            let Some(phalanx) = self.masked_element_phalanx(holder_region, id) else { return false; };
            if !phalanx.attack_due_at(now) { return true; }
            let now = runtime.now_milliseconds();
            if let Some(phalanx) = self.masked_element_phalanx_mut(holder_region, id) { phalanx.mark_attack_at(now); }
        } else if !phalanx.expired_at(now) { return true; }
        let Some(phalanx) = self.masked_element_phalanx(holder_region, id) else { return false; };
        let region = phalanx.shape().is_assigned_to_server_region()
            .then(|| phalanx.shape().get_region_id());
        if let Some(region) = region.filter(|region| self.find_region(*region).is_some()) {
            self.apply_masked_element_area(holder_region, id, region, runtime);
        }
        if !periodic { self.end_summoned_shape(holder_region, id); }
        true
    }

    fn apply_masked_element_area<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, region: i32, runtime: &mut Runtime,
    ) {
        let Some(phalanx) = self.masked_element_phalanx(holder_region, id) else { return; };
        let (origin_x, origin_y) = phalanx.origin();
        let (length, height) = phalanx.dimensions();
        let mut attacked = Vec::new();
        for x in 0..length {
            for y in 0..height {
                let Some(phalanx) = self.masked_element_phalanx(holder_region, id) else { return; };
                if !phalanx.cell_active(x, y) { continue; }
                let Some(owner) = self.find_region(region) else { return; };
                let mut targets = Vec::new();
                let _ = owner.base().get_shapes(
                    origin_x.wrapping_add(x), origin_y.wrapping_add(y),
                    self.area_width, self.area_height,
                    &RegionShapeResolver { game: self, owner }, &mut targets,
                );
                for target in targets.into_iter().map(|view| view.identity) {
                    let Some(phalanx) = self.masked_element_phalanx(holder_region, id) else { return; };
                    if target == phalanx.shape().identity() { continue; }
                    let Some(shape) = resolve_state_move_shape(self, region, target) else { continue; };
                    let target_region = shape.shape().get_region_id();
                    let master = phalanx.master();
                    if target.object_type == master.master_type && target.id == master.master_id { continue; }
                    if attacked.contains(&target) { continue; }
                    let remember_rejected = phalanx.is_periodic();
                    let allowed = master.master_type != PLAYER_TYPE || self.find_player(master.master_id)
                        .map(|player| (player.shape().get_region_id(), player.shape().identity()))
                        .is_some_and(|source| self.live_skill_target_attackable_between(source, (target_region, target)));
                    if !allowed {
                        if remember_rejected { attacked.push(target); }
                        continue;
                    }
                    let Some(snapshot) = self.masked_element_phalanx(holder_region, id)
                        .map(MaskedElementPhalanx::attack_snapshot)
                    else { return; };
                    apply_element_phalanx_attack(self, snapshot, (target_region, target), false, runtime);
                    attacked.push(target);
                }
            }
        }
    }
}
