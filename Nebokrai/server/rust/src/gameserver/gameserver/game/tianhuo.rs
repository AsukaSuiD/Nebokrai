//! Межвладельческая координация небесного огня боевого духа.
//!
//! Владелец навыка хранит проверки, расход, формулу, выбор цели и жизненный
//! цикл. Здесь остаются регистрация в регионе и фактическая круговая доставка.

use super::*;

impl CGame {
    pub(crate) fn add_tianhuo_phalanx<Runtime: GameMainLoopRuntime>(
        &mut self, region_id: i32, phalanx: CTianhuoPhalanx, tile_x: i32,
        tile_y: i32, started_at_ms: u32, runtime: &mut Runtime,
    ) -> Option<Result<i32, RegionMembershipBlock>> {
        self.replace_tianhuo_phalanxes_in_cell(region_id, tile_x, tile_y);
        let mut owner = self.take_region_owner(region_id)?;
        let result = owner.base_mut().add_tianhuo_phalanx(
            phalanx, self.area_width, self.area_height,
            started_at_ms, runtime,
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

    pub(crate) fn send_tianhuo_phalanx_entry<Runtime: GameMainLoopRuntime>(
        &mut self, region_id: i32, phalanx_id: i32, runtime: &mut Runtime,
    ) -> Option<()> {
        let phalanx = self.find_region(region_id)?.base().find_skill_phalanx(phalanx_id)?;
        let SummonedSkillShape::Tianhuo(phalanx) = phalanx else { return None };
        let payload = phalanx.encode_client_snapshot(|| runtime.now_milliseconds())?;
        self.publish_battle_fairy_damage_phalanx_entry(phalanx.shape(), payload)
    }
}
