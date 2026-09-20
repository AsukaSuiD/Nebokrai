//! Живой региональный AI и публикация движущегося FireBall.
//! Источник: gameserver.exe/GameServer.pdb, fireball.cpp и fireballphalanx.cpp.
//! Summon задаёт клетку до свежего региона источника; Add не определяет,
//! будет ли вызван encoder. Неуспешный Add сохраняет форму до этой попытки.
//! AI не извлекает owner и не отмечает его заранее. После области полный
//! End предшествует увеличению позиции, а ForceMove завершает тот же AI.
//! Каждая клетка получает свежий снимок боевых фей, затем фигур. Обычный
//! dedup локален всему проходу области; феи не входят в него и не дают
//! результата, по которому AI завершает форму.

use super::*;
use crate::gameserver::appserver::skills::fireballphalanx::CFireBallPhalanx;

impl CGame {
    pub(crate) fn spawn_fire_ball_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, source: (i32, ShapeIdentity), mut phalanx: CFireBallPhalanx,
        x: i32, y: i32, started_at_ms: u32, runtime: &mut Runtime,
    ) -> Option<()> {
        phalanx.shape_mut().set_pos_xy_base(x as f32 + 0.5, y as f32 + 0.5);
        let source = resolve_state_move_shape(self, source.0, source.1)?;
        if !source.shape().is_assigned_to_server_region() { return None; }
        let region = source.shape().get_region_id();
        let mut owner = self.take_region_owner(region)?;
        let result = owner.base_mut().add_fire_ball_phalanx(
            phalanx, self.area_width, self.area_height, started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        let (shape, payload) = match result {
            Ok(id) => {
                let phalanx = self.fire_ball_phalanx(region, id)?;
                (phalanx.shape().clone(), phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?)
            }
            Err((_, phalanx)) => {
                (phalanx.shape().clone(), phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?)
            }
        };
        self.publish_summoned_shape_entry(&shape, &payload)
    }

    fn fire_ball_phalanx(&self, region: i32, id: i32) -> Option<&CFireBallPhalanx> {
        let SummonedSkillShape::FireBall(phalanx) = self.find_region(region)?.base().find_skill_phalanx(id)?
        else { return None; };
        Some(phalanx)
    }

    fn fire_ball_phalanx_mut(&mut self, region: i32, id: i32) -> Option<&mut CFireBallPhalanx> {
        let SummonedSkillShape::FireBall(phalanx) = self.find_region_mut(region)?.base_mut().find_skill_phalanx_mut(id)?
        else { return None; };
        Some(phalanx)
    }

    fn apply_fire_ball_area<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, center: (i32, i32), runtime: &mut Runtime,
    ) -> bool {
        let Some(phalanx) = self.fire_ball_phalanx(holder_region, id) else { return false; };
        if !phalanx.shape().is_assigned_to_server_region() { return false; }
        let region = phalanx.shape().get_region_id();
        let own = phalanx.shape().identity();
        let attack = phalanx.attack_snapshot();
        let master = attack.master();
        let mut attacked = Vec::new();
        for (x, y) in CFireBallPhalanx::scope_cells(center.0, center.1) {
            let war_souls = self.find_region(region)
                .map(|owner| owner.base().war_souls_at(x, y)).unwrap_or_default();
            for (player_id, _) in war_souls {
                let Some(target) = self.find_player(player_id as i32) else { continue; };
                let Some(source) = self.find_player(master.master_id) else { continue; };
                if target.player_id() == master.master_id || target.shape().get_action() == 6 || target.is_dead() {
                    continue;
                }
                let target = target.shape().identity();
                if self.live_skill_target_attackable(region, source.shape().identity(), target) {
                    attack.apply(self, (region, target), true, runtime);
                }
            }
            let mut shapes = Vec::new();
            if let Some(owner) = self.find_region(region) {
                let _ = owner.base().get_shapes(
                    x, y, self.area_width, self.area_height,
                    &RegionShapeResolver { game: self, owner }, &mut shapes,
                );
            }
            for shape in shapes {
                let identity = shape.identity;
                if (identity.object_type == own.object_type && identity.id == own.id)
                    || resolve_state_move_shape(self, region, identity).is_none()
                    || (identity.object_type == master.master_type && identity.id == master.master_id)
                { continue; }
                let key = (identity.object_type, identity.id);
                if attacked.contains(&key) { continue; }
                if master.master_type == PLAYER_TYPE {
                    let Some(source) = self.find_player(master.master_id) else { continue; };
                    if !self.live_skill_target_attackable(region, source.shape().identity(), identity) { continue; }
                }
                attack.apply(self, (region, identity), false, runtime);
                attacked.push(key);
            }
        }
        !attacked.is_empty()
    }

    pub(super) fn run_fire_ball_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, holder_region: i32, id: i32, runtime: &mut Runtime,
    ) -> bool {
        let lifetime_now = runtime.now_milliseconds();
        let Some(phalanx) = self.fire_ball_phalanx(holder_region, id) else { return false; };
        if phalanx.expired_at(lifetime_now) || phalanx.path_is_empty() {
            self.end_summoned_shape(holder_region, id);
            return true;
        }
        let cell_now = runtime.now_milliseconds();
        let Some(phalanx) = self.fire_ball_phalanx(holder_region, id) else { return false; };
        if phalanx.cell_due_at(cell_now) && phalanx.shape().is_assigned_to_server_region() {
            let region = phalanx.shape().get_region_id();
            if self.find_region(region).is_some() {
                if let Some(cell) = phalanx.current_cell() {
                    if let Some(phalanx) = self.fire_ball_phalanx_mut(holder_region, id) {
                        phalanx.set_cell_destination(cell);
                    }
                    if self.find_region(region).is_some_and(|owner| owner.base().block_at(cell.0, cell.1) == Some(3))
                        && self.apply_fire_ball_area(holder_region, id, cell, runtime)
                    {
                        self.end_summoned_shape(holder_region, id);
                    }
                    if let Some(phalanx) = self.fire_ball_phalanx_mut(holder_region, id) { phalanx.advance(); }
                } else {
                    self.end_summoned_shape(holder_region, id);
                }
            }
        }
        if let Some((x, y, duration)) = self.fire_ball_phalanx(holder_region, id)
            .and_then(CFireBallPhalanx::pending_force_move)
        {
            self.force_move_summoned_shape(holder_region, id, x, y, duration);
            if let Some(phalanx) = self.fire_ball_phalanx_mut(holder_region, id) { phalanx.mark_force_moved(); }
        }
        true
    }
}
