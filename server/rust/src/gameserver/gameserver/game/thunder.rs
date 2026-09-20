//! Межвладельческая координация грома боевого духа.
//!
//! Формула, маска и жизненный цикл принадлежат `thunder.rs` и
//! `thunderphalanx.rs`. Здесь остаются регистрация в регионе, применение атак
//! к независимым владельцам и фактическая круговая доставка.

use super::*;

impl CGame {
    pub(crate) fn add_thunder_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx: CThunderPhalanx,
        _tile_x: i32,
        _tile_y: i32,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_thunder_phalanx(
            phalanx,
            self.area_width,
            self.area_height,
            started_at_ms,
            runtime,
        );
        self.restore_region_owner(owner);
        Some(match result {
            Ok(id) => Ok(id),
            Err((block, phalanx)) => {
                if let Some(payload) = phalanx.encode_client_snapshot(|| runtime.now_milliseconds()) {
                    let _ = self.publish_battle_fairy_damage_phalanx_entry(phalanx.shape(), payload);
                }
                Err(block)
            }
        })
    }

    pub(crate) fn send_thunder_phalanx_entry<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx_id: i32,
        runtime: &mut Runtime,
    ) -> Option<()> {
        let phalanx = self.find_region(region_id)?.base().find_skill_phalanx(phalanx_id)?;
        let SummonedSkillShape::Thunder(phalanx) = phalanx else {
            return None;
        };
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        self.publish_battle_fairy_damage_phalanx_entry(phalanx.shape(), payload)
    }

    pub(super) fn publish_battle_fairy_damage_phalanx_entry(
        &self, shape: &CShape, payload: Vec<u8>,
    ) -> Option<()> {
        let identity = shape.identity();
        let mut message = CMessage::new(0x000b_f502);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.base_mut().add_guid(identity.ex_id);
        message.add_long(i32::try_from(payload.len()).ok()?);
        message.base_mut().add(&payload);
        message.base_mut().add_char(0);
        if !shape.is_assigned_to_server_region() { return Some(()); }
        let region = self.find_region(shape.get_region_id())?;
        let _ = self.send_game_shape_around(region.base(), shape, None, &message);
        Some(())
    }

}
