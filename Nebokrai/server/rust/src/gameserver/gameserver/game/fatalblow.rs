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
        started_at_ms: u32,
        runtime: &mut Runtime,
    ) -> Option<Result<i32, RegionMembershipBlock>> {
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_fatal_blow_phalanx(
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
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        self.publish_battle_fairy_damage_phalanx_entry(phalanx.shape(), payload)
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
