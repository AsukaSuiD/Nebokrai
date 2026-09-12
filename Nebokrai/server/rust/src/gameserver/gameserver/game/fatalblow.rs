//! Межвладельческая координация смертельного удара боевого духа.
//!
//! Формула, стадии и двоичное представление принадлежат `fatalblow.rs` и
//! `fatalblowphalanx.rs`. Здесь остаются только временное извлечение региона,
//! проверка общей `IsAttackAble`-семантики, регистрация снаряда и фактическая
//! круговая доставка, которым нужен доступ к нескольким владельцам `CGame`.

use super::*;

impl CGame {
    pub(crate) fn add_fatal_blow_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx: CFatalBlowPhalanx,
        tile_x: i32,
        tile_y: i32,
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_fatal_blow_phalanx(
            phalanx,
            tile_x,
            tile_y,
            self.area_width,
            self.area_height,
            started_at_ms,
            runtime,
        );
        self.restore_region_owner(owner);
        Some(result)
    }

    pub(crate) fn send_fatal_blow_phalanx_entry<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        phalanx_id: i32,
        runtime: &mut Runtime,
    ) -> Option<()> {
        let phalanx = self
            .find_region(region_id)?
            .base()
            .find_skill_phalanx(phalanx_id)?;
        let SummonedSkillShape::FatalBlow(phalanx) = phalanx else {
            return None;
        };
        let identity = phalanx.shape().identity();
        let tile_x = phalanx.shape().get_tile_x().ok()?;
        let tile_y = phalanx.shape().get_tile_y().ok()?;
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        let payload_size = i32::try_from(payload.len()).ok()?;
        let mut message = CMessage::new(0x000b_f502);
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.base_mut().add_guid(identity.ex_id);
        message.add_long(payload_size);
        message.base_mut().add(&payload);
        message.base_mut().add_char(0);
        let _ = self.send_shape_position_around(region_id, tile_x, tile_y, &message);
        Some(())
    }

    pub(super) fn fatal_blow_attack_ready(
        &self,
        phalanx: &CFatalBlowPhalanx,
        target: ShapeIdentity,
        region_id: i32,
    ) -> Option<bool> {
        self.find_shape_in_region(region_id, target)?;
        crate::gameserver::appserver::states::state::resolve_state_move_shape(
            self, region_id, target,
        )?;
        let master = phalanx.master();
        if master.master_type != PLAYER_TYPE { return Some(true); }
        let Some(source) = self.find_player(master.master_id).map(|player| player.shape().identity())
        else { return Some(false); };
        // В отличие от сканирования области FatalBlow не исключает самого master.
        Some(self.live_skill_target_attackable(region_id, source, target))
    }

}
