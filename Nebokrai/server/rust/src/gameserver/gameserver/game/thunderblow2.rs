//! Межвладельческая граница второго громового удара.
//!
//! Конкретные проверки, формула, отбрасывание и пакет навыка принадлежат
//! `appserver/skills/thunderblow2.rs`. Общий ForceMove разрешает каноническую
//! цель и сохраняет виртуальный SetTileXY перед свежим ожиданием AI.

use super::*;

impl CGame {
    pub(crate) fn force_move_thunder_blow_2_target(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        destination_x: i32,
        destination_y: i32,
        duration_ms: u32,
    ) -> Option<Result<bool, MoveShapeCommandBlock>> {
        self.force_move_skill_target(
            region_id,
            target,
            destination_x,
            destination_y,
            duration_ms,
        )
    }
}
