//! Межвладельческая координация громового удара.
//!
//! Применение навыка, срок жизни и формула принадлежат владельцам навыка и формы.
//! Здесь остаются регистрация формы, публикация снимка, чтение упорядоченного
//! пространственного индекса и применение атаки к независимому владельцу.

use super::*;
use crate::gameserver::appserver::skills::thunderblowphalanx::CThunderBlowPhalanx;

impl CGame {
    pub(crate) fn add_thunder_blow_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region_id: i32, phalanx: CThunderBlowPhalanx,
        tile_x: i32, tile_y: i32, started_at_ms: u32, runtime: &mut Runtime,
    ) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_thunder_blow_phalanx(
            phalanx, tile_x, tile_y, self.area_width, self.area_height, started_at_ms, runtime,
        );
        self.restore_region_owner(owner);
        Some(result)
    }

    pub(crate) fn send_thunder_blow_phalanx_entry<Runtime: GameMainLoopRuntime>(
        &mut self, region_id: i32, phalanx_id: i32, runtime: &mut Runtime,
    ) -> Option<()> {
        let phalanx = self.find_region(region_id)?.base().find_skill_phalanx(phalanx_id)?;
        let SummonedSkillShape::ThunderBlow(phalanx) = phalanx else { return None };
        let identity = phalanx.shape().identity();
        let x = phalanx.shape().get_tile_x().ok()?;
        let y = phalanx.shape().get_tile_y().ok()?;
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let mut message = CMessage::new(0x000b_f502);
        message.add_long(identity.object_type); message.add_long(identity.id);
        message.base_mut().add_guid(identity.ex_id);
        message.add_long(i32::try_from(payload.len()).ok()?);
        message.base_mut().add(&payload); message.base_mut().add_char(0);
        let _ = self.send_shape_position_around(region_id, x, y, &message);
        Some(())
    }

    pub(super) fn thunder_blow_targets(
        &self, region_id: i32, phalanx: &CThunderBlowPhalanx,
    ) -> Vec<ShapeIdentity> {
        let Some(region) = self.find_region(region_id).map(|owner| owner.base()) else {
            return Vec::new();
        };
        let (Ok(x), Ok(y)) = (phalanx.shape().get_tile_x(), phalanx.shape().get_tile_y()) else {
            return Vec::new();
        };
        let mut shapes = Vec::new();
        if region
            .get_shapes(x, y, self.area_width, self.area_height, self, &mut shapes)
            .is_err()
        {
            return Vec::new();
        }
        shapes.into_iter().map(|shape| shape.identity).filter(|identity| {
            *identity != phalanx.shape().identity()
                && !(identity.object_type == phalanx.master().master_type && identity.id == phalanx.master().master_id)
                && matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
                && self.owned_player_skill_target_attackable(
                    phalanx.master(), *identity, region_id,
                )
        }).collect()
    }
}
