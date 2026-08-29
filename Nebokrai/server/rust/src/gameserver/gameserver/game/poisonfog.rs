//! Межвладельческая координация ядовитого тумана.
//!
//! Применение навыка, формулы состояния и жизненный цикл области принадлежат
//! владельцам навыка. Здесь остаются проверки PK и региона, каноническая
//! замена состояния, пересчёт свойств и фактическая доставка вокруг объекта.

use super::*;
use crate::gameserver::appserver::skills::curestate::CURE_STATE_SKILL_ID;
use crate::gameserver::appserver::skills::poisonfogphalanx::{poison_fog_targets, CPoisonFogPhalanx};
use crate::gameserver::appserver::skills::poisonfogstate::send_poison_fog_state_visual;

impl CGame {
    pub(crate) fn add_poison_fog_phalanx<Runtime: GameMainLoopRuntime>(&mut self, region_id: i32, phalanx: CPoisonFogPhalanx, x: i32, y: i32, now_ms: u32, runtime: &mut Runtime) -> Option<Result<i32, RegionMembershipBlock>> { let mut owner = self.take_region_owner(region_id)?; let result = owner.base_mut().add_poison_fog_phalanx(phalanx, x, y, self.area_width, self.area_height, now_ms, runtime); self.restore_region_owner(owner); Some(result) }
    pub(crate) fn send_poison_fog_phalanx_entry(&mut self, region_id: i32, id: i32) -> Option<()> { let phalanx = self.find_region(region_id)?.base().find_skill_phalanx(id)?; let SummonedSkillShape::PoisonFog(phalanx) = phalanx else { return None }; let identity = phalanx.shape().identity(); let x = phalanx.shape().get_tile_x().ok()?; let y = phalanx.shape().get_tile_y().ok()?; let payload = phalanx.encode_client_snapshot()?; let mut message = CMessage::new(0x000b_f502); message.add_long(identity.object_type); message.add_long(identity.id); message.base_mut().add_guid(identity.ex_id); message.add_long(i32::try_from(payload.len()).ok()?); message.base_mut().add(&payload); message.base_mut().add_char(0); let _ = self.send_shape_position_around(region_id, x, y, &message); Some(()) }
    fn poison_fog_target_attackable(&self, region_id: i32, phalanx: &CPoisonFogPhalanx, target: ShapeIdentity) -> bool {
        let master = phalanx.master();
        let Some(master_player) = self.find_player(master.master_id) else { return false };
        let Some(region) = self.find_region(region_id).map(ServerRegionOwner::base) else { return false };
        let (Ok(mx), Ok(my)) = (master_player.shape().get_tile_x(), master_player.shape().get_tile_y()) else { return false };
        if region.block_at(mx, my) == Some(2) { return false }
        match target.object_type {
            PLAYER_TYPE => self.find_player(target.id).is_some_and(|player| !player.is_dead() && !player.has_state_by_skill_id(CURE_STATE_SKILL_ID) && player.shape().get_tile_x().ok().zip(player.shape().get_tile_y().ok()).is_some_and(|(x, y)| region.block_at(x, y) != Some(2)) && self.player_base_attackable(master.master_id, target.id)),
            MONSTER_TYPE => {
                let Some(monster) = region.find_monster_by_id(target.id) else { return false };
                if monster.hit_points() == 0 || monster.move_shape().has_state_by_skill_id(CURE_STATE_SKILL_ID) { return false }
                let (Ok(x), Ok(y)) = (monster.move_shape().shape().get_tile_x(), monster.move_shape().shape().get_tile_y()) else { return false };
                if region.block_at(x, y) == Some(2) { return false }
                let Some(property) = monster.base_property_key().and_then(|key| self.find_monster_property_by_origin_name(key)) else { return false };
                if !self.guard_monster_attackable(master.master_id, region_id, property) { return false }
                let target_master = monster.master_info();
                if (monster.is_tamed() || monster.is_carriage(property)) && target_master.master_type == PLAYER_TYPE && target_master.master_id != 0 {
                    return if target_master.master_id == master.master_id { master.permitted_to_kill_criminal != 0 } else { self.player_base_attackable(master.master_id, target_master.master_id) };
                }
                true
            }
            _ => false,
        }
    }
    pub(super) fn apply_poison_fog_phalanx<Runtime: GameMainLoopRuntime>(&mut self, region_id: i32, phalanx: &CPoisonFogPhalanx, runtime: &mut Runtime) -> usize {
        let mut applied = 0usize;
        for target in poison_fog_targets(self, region_id, phalanx) {
            if !self.poison_fog_target_attackable(region_id, phalanx, target) { continue }
            let now = runtime.now_milliseconds(); let state = phalanx.state(now);
            match target.object_type {
                PLAYER_TYPE => {
                    let _ = self.player_on_first_skill(phalanx.master().master_id, target.id, Some(region_id), runtime);
                    let replaced = self.find_player_mut(target.id).and_then(|player| { let x = player.shape().get_tile_x().ok()?; let y = player.shape().get_tile_y().ok()?; let previous = player.replace_poison_fog_state(state, now); Some((x, y, previous)) });
                    if let Some((x, y, previous)) = replaced { if let Some(previous) = previous { send_poison_fog_state_visual(self, region_id, target, x, y, previous, false, now); } send_poison_fog_state_visual(self, region_id, target, x, y, state, true, now); let _ = self.update_player_properties(target.id, runtime); applied = applied.wrapping_add(1); }
                }
                MONSTER_TYPE => {
                    let replaced = if let Some(mut owner) = self.take_region_owner(region_id) { let result = owner.base_mut().find_monster_by_id_mut(target.id).and_then(|monster| { let x = monster.move_shape().shape().get_tile_x().ok()?; let y = monster.move_shape().shape().get_tile_y().ok()?; let previous = monster.move_shape_mut().replace_poison_fog_state(state, now); Some((x, y, previous)) }); self.restore_region_owner(owner); result } else { None };
                    if let Some((x, y, previous)) = replaced { if let Some(previous) = previous { send_poison_fog_state_visual(self, region_id, target, x, y, previous, false, now); } send_poison_fog_state_visual(self, region_id, target, x, y, state, true, now); applied = applied.wrapping_add(1); }
                }
                _ => {}
            }
        }
        applied
    }
}
