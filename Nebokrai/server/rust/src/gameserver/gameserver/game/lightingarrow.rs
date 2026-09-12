//! Межвладельческая координация световой стрелы.
//!
//! Execution, путь, часы, порядок клеток, яд и формула принадлежат
//! `lightingarrow.rs` и `lightingarrowphalanx.rs`. Здесь остаются региональная
//! регистрация, `ForceMove`, разрешение независимых владельцев и применение
//! результата к цели.

use super::*;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::skills::lightingarrowphalanx::{lighting_arrow_targets, CLightingArrowPhalanx};
use crate::gameserver::appserver::states::state::resolve_state_move_shape;

impl CGame {
    pub(crate) fn add_lighting_arrow_phalanx<Runtime: GameMainLoopRuntime>(&mut self, region_id: i32,
        phalanx: CLightingArrowPhalanx, tile_x: i32, tile_y: i32, started_at_ms: u32,
        runtime: &mut Runtime) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_lighting_arrow_phalanx(phalanx, tile_x, tile_y,
            self.area_width, self.area_height, started_at_ms, runtime);
        self.restore_region_owner(owner); Some(result)
    }

    pub(crate) fn send_lighting_arrow_phalanx_entry<Runtime: GameMainLoopRuntime>(&mut self,
        region_id: i32, phalanx_id: i32, runtime: &mut Runtime) -> Option<()> {
        let phalanx = self.find_region(region_id)?.base().find_skill_phalanx(phalanx_id)?;
        let SummonedSkillShape::LightingArrow(phalanx) = phalanx else { return None };
        let identity = phalanx.shape().identity(); let x = phalanx.shape().get_tile_x().ok()?;
        let y = phalanx.shape().get_tile_y().ok()?; let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let mut message = CMessage::new(0x000b_f502); message.add_long(identity.object_type); message.add_long(identity.id);
        message.base_mut().add_guid(identity.ex_id); message.add_long(i32::try_from(payload.len()).ok()?);
        message.base_mut().add(&payload); message.base_mut().add_char(0);
        let _ = self.send_shape_position_around(region_id, x, y, &message); Some(())
    }

    pub(super) fn force_move_lighting_arrow(&mut self, region_id: i32, phalanx_id: i32,
        destination_x: i32, destination_y: i32, duration_ms: u32) -> bool {
        let Some(mut owner) = self.take_region_owner(region_id) else { return false };
        let region = owner.base_mut(); let destination_x = destination_x.clamp(0, region.region.width.saturating_sub(1));
        let destination_y = destination_y.clamp(0, region.region.height.saturating_sub(1));
        let Some(SummonedSkillShape::LightingArrow(phalanx)) = region.find_skill_phalanx(phalanx_id) else { self.restore_region_owner(owner); return false };
        let (Ok(old_x), Ok(old_y)) = (phalanx.shape().get_tile_x(), phalanx.shape().get_tile_y()) else { self.restore_region_owner(owner); return false };
        let identity = phalanx.shape().identity(); let mut message = CMessage::new(0x000b_f604);
        message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(old_x); message.add_long(old_y);
        message.add_long(destination_x); message.add_long(destination_y); message.add_ulong(duration_ms); message.add_long(0);
        let _ = self.send_shape_position_around(region_id, old_x, old_y, &message);
        if let Some(SummonedSkillShape::LightingArrow(phalanx)) = region.find_skill_phalanx_mut(phalanx_id) {
            phalanx.shape_mut().set_pos_xy_move_order(destination_x as f32 + 0.5, destination_y as f32 + 0.5);
        }
        self.restore_region_owner(owner); true
    }

    // Сканирование проверяет каждый RTTI CMoveShape после предыдущего
    // попадания. У non-player master нет IsAttackAble; HP проверяет сам Attack.
    pub(crate) fn summoned_skill_scan_target_allowed(&self, region_id: i32,
        master: MasterInfo, target: ShapeIdentity) -> bool {
        if (target.object_type == master.master_type && target.id == master.master_id)
            || resolve_state_move_shape(self, region_id, target).is_none()
        { return false }
        if master.master_type != PLAYER_TYPE { return true }
        let Some(source) = self.find_player(master.master_id).map(|player| player.shape().identity())
        else { return false };
        self.live_skill_target_attackable(region_id, source, target)
    }

    pub(super) fn apply_lighting_arrow_cell<Runtime: GameMainLoopRuntime>(&mut self, region_id: i32,
        phalanx_id: i32, tile_x: i32, tile_y: i32, _sampled_at_ms: u32, runtime: &mut Runtime) {
        let Some(phalanx) = self.find_region(region_id).and_then(|r| r.base().find_skill_phalanx(phalanx_id)).cloned() else { return };
        let SummonedSkillShape::LightingArrow(snapshot) = &phalanx else { return };
        let targets = lighting_arrow_targets(self, region_id, snapshot, tile_x, tile_y);
        for target in targets {
            let Some(SummonedSkillShape::LightingArrow(current)) = self.find_region(region_id)
                .and_then(|region| region.base().find_skill_phalanx(phalanx_id))
            else { return };
            if current.was_attacked(target)
                || !self.summoned_skill_scan_target_allowed(region_id, snapshot.master(), target)
                || self.base_magic_target_dead(region_id, target)
            { continue }
            let marked = if let Some(mut owner) = self.take_region_owner(region_id) {
                let marked = match owner.base_mut().find_skill_phalanx_mut(phalanx_id) {
                    Some(SummonedSkillShape::LightingArrow(current)) => current.mark_attacked(target),
                    _ => false,
                };
                self.restore_region_owner(owner); marked
            } else { false };
            if !marked { continue }
            self.apply_summoned_skill_to_target(&phalanx, target, region_id, false, runtime);
        }
    }
}
