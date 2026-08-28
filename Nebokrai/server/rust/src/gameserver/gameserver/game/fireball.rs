//! Межвладельческая координация огненного шара.
//!
//! Применение навыка, путь, часы, маска и формула принадлежат `fireball.rs` и
//! `fireballphalanx.rs`. Здесь остаются регистрация в регионе, публикация
//! снимка, пространственное перемещение `ForceMove`, точный обход «боевой дух перед фигурой» и
//! применение рассчитанной атаки к независимым владельцам.

use super::*;
use crate::gameserver::appserver::skills::fireballphalanx::CFireBallPhalanx;

impl CGame {
    pub(crate) fn add_fire_ball_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx: CFireBallPhalanx,
        tile_x: i32,
        tile_y: i32,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_fire_ball_phalanx(
            phalanx, tile_x, tile_y, self.area_width, self.area_height,
            started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        Some(result)
    }

    pub(crate) fn send_fire_ball_phalanx_entry<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx_id: i32,
        runtime: &mut Runtime,
    ) -> Option<()> {
        let phalanx = self.find_region(region_id)?.base().find_skill_phalanx(phalanx_id)?;
        let SummonedSkillShape::FireBall(phalanx) = phalanx else { return None };
        let identity = phalanx.shape().identity();
        let tile_x = phalanx.shape().get_tile_x().ok()?;
        let tile_y = phalanx.shape().get_tile_y().ok()?;
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
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

    pub(super) fn force_move_fire_ball(
        &mut self,
        region_id: i32,
        phalanx_id: i32,
        destination_x: i32,
        destination_y: i32,
        duration_ms: u32,
    ) -> bool {
        let Some(mut owner) = self.take_region_owner(region_id) else { return false };
        let region = owner.base_mut();
        let width = region.region.width;
        let height = region.region.height;
        let destination_x = destination_x.clamp(0, width.saturating_sub(1));
        let destination_y = destination_y.clamp(0, height.saturating_sub(1));
        let Some(SummonedSkillShape::FireBall(phalanx)) = region.find_skill_phalanx(phalanx_id) else {
            self.restore_region_owner(owner);
            return false;
        };
        let Ok(old_x) = phalanx.shape().get_tile_x() else { self.restore_region_owner(owner); return false };
        let Ok(old_y) = phalanx.shape().get_tile_y() else { self.restore_region_owner(owner); return false };
        let identity = phalanx.shape().identity();
        let mut message = CMessage::new(0x000b_f604);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.add_long(old_x);
        message.add_long(old_y);
        message.add_long(destination_x);
        message.add_long(destination_y);
        message.add_ulong(duration_ms);
        message.add_long(0);
        let _ = self.send_shape_position_around(region_id, old_x, old_y, &message);
        if let Some(SummonedSkillShape::FireBall(phalanx)) = region.find_skill_phalanx_mut(phalanx_id) {
            phalanx.shape_mut().set_pos_xy_move_order(destination_x as f32 + 0.5, destination_y as f32 + 0.5);
        }
        self.restore_region_owner(owner);
        true
    }

    pub(super) fn fire_ball_targets(
        &self,
        region_id: i32,
        phalanx: &CFireBallPhalanx,
        center_x: i32,
        center_y: i32,
    ) -> Vec<(ShapeIdentity, bool)> {
        let Some(region) = self.find_region(region_id).map(ServerRegionOwner::base) else {
            return Vec::new();
        };
        let mut targets = Vec::new();
        let mut ordinary = Vec::new();
        for (tile_x, tile_y) in CFireBallPhalanx::scope_cells(center_x, center_y) {
            for (&player_id, _) in &region.war_souls_at(tile_x, tile_y) {
                let player_id = player_id as i32;
                if player_id != phalanx.master().master_id
                    && self.find_player(player_id).is_some_and(|player| !player.is_dead())
                    && self.player_base_attackable(phalanx.master().master_id, player_id)
                {
                    targets.push((ShapeIdentity {
                        object_type: PLAYER_TYPE,
                        id: player_id,
                        ex_id: CGuid::GUID_INVALID,
                    }, true));
                }
            }
            let mut shapes = Vec::new();
            if region.get_shapes(
                tile_x, tile_y, self.area_width, self.area_height, self, &mut shapes,
            ).is_err() {
                continue;
            }
            for shape in shapes {
                let identity = shape.identity;
                if identity == phalanx.shape().identity()
                    || (identity.object_type == phalanx.master().master_type
                        && identity.id == phalanx.master().master_id)
                    || !matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
                    || ordinary.contains(&identity)
                {
                    continue;
                }
                if phalanx.master().master_type == PLAYER_TYPE
                    && identity.object_type == PLAYER_TYPE
                    && !self.player_base_attackable(phalanx.master().master_id, identity.id)
                {
                    continue;
                }
                ordinary.push(identity);
                targets.push((identity, false));
            }
        }
        targets
    }
}
