//! Межвладельческое применение состояния и отбрасывания `CBossBlueQuake`.
//!
//! Формулу, длительность и геометрию задаёт владелец навыка. `CGame`
//! адресует опубликованную actual-цель для канонической замены состояния,
//! круговой доставки и последующего `ForceMove`.

use super::*;
use crate::gameserver::appserver::skills::bossbluequake::{quake_knockback_destination, replace_quake_state};
use crate::gameserver::appserver::skills::bossbluequakestate::BossBlueQuakeState;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;

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
        back_steps: u32,
        move_speed: u32,
        runtime: &mut Runtime,
    ) -> bool {
        let Some(source_region) = self.find_player(source_player_id)
            .and_then(CPlayer::server_region_id) else { return false; };
        if self.find_region(source_region).is_none() {
            return false;
        }
        let source = ShapeIdentity { object_type: 400, id: source_player_id, ex_id: CGuid::GUID_INVALID };
        let Some(target_region) = resolve_state_move_shape(self, region_id, target)
            .map(|shape| shape.shape().get_region_id()) else { return false; };
        replace_quake_state(self, target_region, target, (source_region, source), state, || {
            runtime.now_milliseconds()
        });
        self.increase_owned_player_rp(source_player_id, true, 0);
        let Some((destination_x, destination_y, moved)) = quake_knockback_destination(
            self, source_region, source, target_region, target, back_steps,
        ) else { return false; };
        let _ = self.force_move_skill_target(
            target_region,
            target,
            destination_x,
            destination_y,
            move_speed.wrapping_mul(moved),
        );
        true
    }
}
