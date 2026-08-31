//! Межвладельческая граница второго громового удара.
//!
//! Конкретные проверки, формула, отбрасывание и пакет навыка принадлежат
//! `appserver/skills/thunderblow2.rs`. Здесь остаётся только атомарное
//! извлечение региона, разрешение канонического владельца player/monster и
//! восстановление региона после `ForceMove`.

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
        let mut owner = self.take_region_owner(region_id)?;
        let result = self.force_move_owned_shape(
            owner.base_mut(),
            target,
            destination_x,
            destination_y,
            duration_ms,
        );
        self.restore_region_owner(owner);
        result
    }
}
