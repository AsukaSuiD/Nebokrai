//! Межвладельческое применение состояния и отбрасывания `CBossBlueQuake`.
//!
//! Формулу, длительность и конечную клетку вычисляет владелец навыка. `CGame`
//! временно извлекает регион только для атомарного доступа к цели, канонической
//! замены состояния, круговой доставки и последующего `ForceMove`.

use super::*;
use crate::gameserver::appserver::skills::bossbluequake::replace_quake_state;
use crate::gameserver::appserver::skills::bossbluequakestate::BossBlueQuakeState;

impl CGame {
    #[allow(
        clippy::too_many_arguments,
        reason = "граница сохраняет состояние и ForceMove одной цели в исходном порядке"
    )]
    pub(crate) fn apply_boss_blue_quake_control<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        source_player_id: i32,
        target: ShapeIdentity,
        state: BossBlueQuakeState,
        destination_x: i32,
        destination_y: i32,
        duration_ms: u32,
        runtime: &mut Runtime,
    ) -> bool {
        let Some(mut owner) = self.take_region_owner(region_id) else {
            return false;
        };
        replace_quake_state(self, owner.base_mut(), target, state, || {
            runtime.now_milliseconds()
        });
        self.increase_owned_player_rp(source_player_id, true, 0);
        let _ = self.force_move_owned_shape(
            owner.base_mut(),
            target,
            destination_x,
            destination_y,
            duration_ms,
        );
        self.restore_region_owner(owner);
        true
    }
}
