//! Региональный жизненный цикл прицельных снарядов Archery, BaseMagic и FireBolt.
//! Источник: gameserver.exe/GameServer.pdb, одноимённые skills и phalanx.
//! Summon задаёт клетку до свежего региона источника, затем вызывает Add
//! без отдельной сериализации или BF502. Часы срока жизни и задержки
//! читаются отдельно; цель разрешается через GetObject фактического региона.
//! Форма остаётся опубликованной во время контакта. Общий тихий End следует
//! за Attack, даже при NULL регионе/цели, и не отправляет немедленный BF504.
//! Формулы и снимки атаки остаются у соответствующих владельцев.

use super::*;
use crate::gameserver::appserver::skills::archeryphalanx::apply_archery_attack;
use crate::gameserver::appserver::skills::baseprojectilephalanx::BaseProjectileFlight;

fn flight(phalanx: &SummonedSkillShape) -> Option<&BaseProjectileFlight> {
    match phalanx {
        SummonedSkillShape::Archery(phalanx) => Some(phalanx.flight()),
        SummonedSkillShape::BaseMagic(phalanx) => Some(phalanx.flight()),
        SummonedSkillShape::FireBolt(phalanx) => Some(phalanx.flight()),
        _ => None,
    }
}

fn flight_mut(phalanx: &mut SummonedSkillShape) -> Option<&mut BaseProjectileFlight> {
    match phalanx {
        SummonedSkillShape::Archery(phalanx) => Some(phalanx.flight_mut()),
        SummonedSkillShape::BaseMagic(phalanx) => Some(phalanx.flight_mut()),
        SummonedSkillShape::FireBolt(phalanx) => Some(phalanx.flight_mut()),
        _ => None,
    }
}

impl CGame {
    pub(crate) fn spawn_base_projectile<Runtime: GameMainLoopRuntime>(
        &mut self, source: (i32, ShapeIdentity), mut phalanx: SummonedSkillShape,
        x: i32, y: i32, started_at_ms: u32, runtime: &mut Runtime,
    ) -> Option<()> {
        flight(&phalanx)?;
        phalanx.shape_mut().set_pos_xy_base(x as f32 + 0.5, y as f32 + 0.5);
        let source = resolve_state_move_shape(self, source.0, source.1)?;
        if !source.shape().is_assigned_to_server_region() { return None; }
        let region = source.shape().get_region_id();
        let mut owner = self.take_region_owner(region)?;
        let _ = owner.base_mut().add_base_projectile(
            phalanx, self.area_width, self.area_height, started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        Some(())
    }

    pub(super) fn base_projectile_flight(&self, region: i32, id: i32) -> Option<&BaseProjectileFlight> {
        flight(self.find_region(region)?.base().find_skill_phalanx(id)?)
    }

    pub(super) fn end_base_projectile(&mut self, region: i32, id: i32) {
        if let Some(flight) = self.find_region_mut(region)
            .and_then(|owner| owner.base_mut().find_skill_phalanx_mut(id))
            .and_then(flight_mut)
        { flight.end(); }
    }

    pub(super) fn run_base_projectile<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, runtime: &mut Runtime,
    ) -> bool {
        let lifetime_now = runtime.now_milliseconds();
        let Some(flight) = self.base_projectile_flight(holder_region, id) else { return false; };
        if flight.expired_at(lifetime_now) {
            self.end_base_projectile(holder_region, id);
            return true;
        }
        let attack_now = runtime.now_milliseconds();
        let Some(flight) = self.base_projectile_flight(holder_region, id) else { return false; };
        if !flight.attack_due_at(attack_now) { return true; }
        if flight.shape().is_assigned_to_server_region() {
            let region = flight.shape().get_region_id();
            let target = flight.target();
            if self.find_region(region).is_some_and(|owner| owner.base().has_registered_shape(target))
                && resolve_state_move_shape(self, region, target).is_some()
                && let Some(phalanx) = self.find_region(holder_region)
                    .and_then(|owner| owner.base().find_skill_phalanx(id))
            {
                match phalanx {
                    SummonedSkillShape::Archery(phalanx) => {
                        let snapshot = phalanx.attack_snapshot();
                        apply_archery_attack(self, snapshot, (region, target), runtime);
                    }
                    SummonedSkillShape::BaseMagic(phalanx) => {
                        let snapshot = phalanx.attack_snapshot();
                        snapshot.apply(self, (region, target), false, runtime);
                    }
                    SummonedSkillShape::FireBolt(phalanx) => {
                        let snapshot = phalanx.attack_snapshot();
                        snapshot.apply(self, (region, target), false, runtime);
                    }
                    _ => {}
                }
            }
        }
        self.end_base_projectile(holder_region, id);
        true
    }
}
