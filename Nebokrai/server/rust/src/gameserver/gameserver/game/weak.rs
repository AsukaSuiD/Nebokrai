//! Межвладельческая координация области ослабления.
//!
//! Условия cast, формула срока, прямоугольник и изменение атаки принадлежат
//! `weak.rs`, `weakphalanx.rs` и `weakstate.rs`. Здесь остаются ordered
//! `GetShape`, временное извлечение региона, каноническая установка состояния,
//! перерасчёт player owner-а и фактическая around-доставка.

use super::*;
use crate::gameserver::appserver::skills::weakphalanx::CWeakPhalanx;
use crate::gameserver::appserver::skills::weakstate::{WeakState, send_weak_state_visual};
use crate::gameserver::appserver::skills::weak::WEAK_SKILL_ID;
use crate::gameserver::appserver::masterinfo::MasterInfo;

impl CGame {
    pub(crate) fn add_weak_phalanx<Runtime: GameMainLoopRuntime>(&mut self, region_id: i32, phalanx: CWeakPhalanx, tile_x: i32, tile_y: i32, started_at_ms: u32, runtime: &mut Runtime) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_weak_phalanx(phalanx, tile_x, tile_y, self.area_width, self.area_height, started_at_ms, runtime);
        self.restore_region_owner(owner);
        Some(result)
    }

    pub(crate) fn send_weak_phalanx_entry<Runtime: GameMainLoopRuntime>(&mut self, region_id: i32, phalanx_id: i32, _runtime: &mut Runtime) -> Option<()> {
        let phalanx = self.find_region(region_id)?.base().find_skill_phalanx(phalanx_id)?;
        let SummonedSkillShape::Weak(phalanx) = phalanx else { return None };
        let identity = phalanx.shape().identity();
        let tile_x = phalanx.shape().get_tile_x().ok()?;
        let tile_y = phalanx.shape().get_tile_y().ok()?;
        let payload = phalanx.encode_client_snapshot()?;
        let mut message = CMessage::new(0x000b_f502);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.base_mut().add_guid(identity.ex_id);
        message.add_long(i32::try_from(payload.len()).ok()?);
        message.base_mut().add(&payload);
        message.base_mut().add_char(0);
        let _ = self.send_shape_position_around(region_id, tile_x, tile_y, &message);
        Some(())
    }

    pub(super) fn weak_targets(&self, region_id: i32, phalanx: &CWeakPhalanx) -> Vec<ShapeIdentity> {
        let Some(region) = self.find_region(region_id).map(ServerRegionOwner::base) else { return Vec::new() };
        let mut targets = Vec::new();
        for (tile_x, tile_y) in phalanx.active_cells() {
            let mut shapes = Vec::new();
            if region.get_shapes(tile_x, tile_y, self.area_width, self.area_height, self, &mut shapes).is_err() { break; }
            for shape in shapes {
                if shape.identity == phalanx.shape().identity()
                    || (shape.identity.object_type == phalanx.master().master_type && shape.identity.id == phalanx.master().master_id)
                    || !matches!(shape.identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
                { continue; }
                targets.push(shape.identity);
            }
        }
        targets
    }

    fn weak_target_attackable(&self, region_id: i32, master: MasterInfo, target: ShapeIdentity) -> bool {
        match target.object_type {
            PLAYER_TYPE => self.find_player(target.id).is_some_and(|player| {
                !player.has_state_by_skill_id(WEAK_SKILL_ID)
                    && self.player_base_attackable(master.master_id, target.id)
            }),
            MONSTER_TYPE => {
                let Some(region) = self.find_region(region_id).map(ServerRegionOwner::base) else { return false };
                let Some(monster) = region.find_monster_by_id(target.id) else { return false };
                if monster.hit_points() == 0 || monster.move_shape().is_god() || monster.move_shape().weak_state().is_some() { return false; }
                let Some(property) = monster.base_property_key().and_then(|key| self.find_monster_property_by_origin_name(key)) else { return false };
                if !self.guard_monster_attackable(master.master_id, region_id, property) { return false; }
                let target_master = monster.master_info();
                if (monster.is_tamed() || monster.is_carriage(property)) && target_master.master_type == PLAYER_TYPE && target_master.master_id != 0 {
                    if target_master.master_id == master.master_id {
                        return master.permitted_to_kill_criminal != 0;
                    }
                    return self.player_base_attackable(master.master_id, target_master.master_id);
                }
                true
            }
            _ => false,
        }
    }

    pub(super) fn apply_weak_phalanx<Runtime: GameMainLoopRuntime>(&mut self, region_id: i32, phalanx: &CWeakPhalanx, runtime: &mut Runtime) -> usize {
        let center_x = phalanx.shape().get_tile_x().unwrap_or_default();
        let center_y = phalanx.shape().get_tile_y().unwrap_or_default();
        let state = WeakState::new(phalanx.attack_loss(), center_x, center_y, phalanx.length(), phalanx.height());
        let mut applied = 0usize;
        for target in self.weak_targets(region_id, phalanx) {
            if !self.weak_target_attackable(region_id, phalanx.master(), target) { continue; }
            match target.object_type {
                PLAYER_TYPE => {
                    let position = self.find_player_mut(target.id).and_then(|player| {
                        let x = player.shape().get_tile_x().ok()?;
                        let y = player.shape().get_tile_y().ok()?;
                        player.replace_weak_state(state);
                        Some((x, y))
                    });
                    if let Some((x, y)) = position {
                        send_weak_state_visual(self, region_id, target, x, y, state, true);
                        let _ = self.update_player_properties(target.id, runtime);
                        applied = applied.wrapping_add(1);
                    }
                }
                MONSTER_TYPE => {
                    let position = if let Some(mut owner) = self.take_region_owner(region_id) {
                        let result = owner.base_mut().find_monster_by_id_mut(target.id).and_then(|monster| {
                            let x = monster.move_shape().shape().get_tile_x().ok()?;
                            let y = monster.move_shape().shape().get_tile_y().ok()?;
                            monster.move_shape_mut().replace_weak_state(state);
                            Some((x, y))
                        });
                        self.restore_region_owner(owner);
                        result
                    } else { None };
                    if let Some((x, y)) = position {
                        send_weak_state_visual(self, region_id, target, x, y, state, true);
                        applied = applied.wrapping_add(1);
                    }
                }
                _ => {}
            }
        }
        applied
    }

    pub(super) fn finish_player_weak_outside<Runtime: GameMainLoopRuntime>(&mut self, player_id: i32, runtime: &mut Runtime) -> bool {
        let ended = self.find_player_mut(player_id).and_then(|player| {
            let region_id = player.server_region_id()?;
            let x = player.shape().get_tile_x().ok()?;
            let y = player.shape().get_tile_y().ok()?;
            let state = player.take_weak_state_outside(x, y)?;
            Some((region_id, x, y, state))
        });
        let Some((region_id, x, y, state)) = ended else { return false };
        send_weak_state_visual(self, region_id, ShapeIdentity { object_type: PLAYER_TYPE, id: player_id, ex_id: CGuid::GUID_INVALID }, x, y, state, false);
        let _ = self.update_player_properties(player_id, runtime);
        true
    }

    pub(super) fn finish_weak_phalanx_targets<Runtime: GameMainLoopRuntime>(&mut self, region_id: i32, phalanx: &CWeakPhalanx, runtime: &mut Runtime) -> usize {
        let mut ended = 0usize;
        for target in self.weak_targets(region_id, phalanx) {
            match target.object_type {
                PLAYER_TYPE => {
                    let removed = self.find_player_mut(target.id).and_then(|player| {
                        let x = player.shape().get_tile_x().ok()?;
                        let y = player.shape().get_tile_y().ok()?;
                        let state = player.take_weak_state()?;
                        Some((x, y, state))
                    });
                    if let Some((x, y, state)) = removed {
                        send_weak_state_visual(self, region_id, target, x, y, state, false);
                        let _ = self.update_player_properties(target.id, runtime);
                        ended = ended.wrapping_add(1);
                    }
                }
                MONSTER_TYPE => {
                    let removed = if let Some(mut owner) = self.take_region_owner(region_id) {
                        let result = owner.base_mut().find_monster_by_id_mut(target.id).and_then(|monster| {
                            let x = monster.move_shape().shape().get_tile_x().ok()?;
                            let y = monster.move_shape().shape().get_tile_y().ok()?;
                            let state = monster.move_shape_mut().take_weak_state()?;
                            Some((x, y, state))
                        });
                        self.restore_region_owner(owner);
                        result
                    } else { None };
                    if let Some((x, y, state)) = removed {
                        send_weak_state_visual(self, region_id, target, x, y, state, false);
                        ended = ended.wrapping_add(1);
                    }
                }
                _ => {}
            }
        }
        ended
    }

    pub(super) fn finish_monster_weak_outside(&mut self, region_id: i32, monster_id: i32) -> bool {
        let ended = if let Some(mut owner) = self.take_region_owner(region_id) {
            let result = owner.base_mut().find_monster_by_id_mut(monster_id).and_then(|monster| {
                let x = monster.move_shape().shape().get_tile_x().ok()?;
                let y = monster.move_shape().shape().get_tile_y().ok()?;
                let state = monster.move_shape_mut().take_weak_state_outside(x, y)?;
                Some((x, y, state))
            });
            self.restore_region_owner(owner);
            result
        } else { None };
        let Some((x, y, state)) = ended else { return false };
        send_weak_state_visual(self, region_id, ShapeIdentity { object_type: MONSTER_TYPE, id: monster_id, ex_id: CGuid::GUID_INVALID }, x, y, state, false);
        true
    }
}
