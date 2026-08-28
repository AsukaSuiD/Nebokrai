//! Межвладельческое применение `RushState` и отбрасывания `CRush`.
//!
//! Выбор цели, длительность и конечная клетка принадлежат владельцу навыка.
//! `CGame` временно извлекает регион только для атомарной работы канонического
//! состояния игрока либо монстра, пространственного индекса, сети и ожидания ИИ.

use super::*;
use crate::gameserver::appserver::skills::rushstate::{
    RushState, replace_monster_rush_state, replace_player_rush_state,
};

impl CGame {
    #[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца состояния и пространственный эффект")]
    pub(crate) fn apply_rush_control<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        state: RushState,
        destination_x: i32,
        destination_y: i32,
        duration_ms: u32,
        now_ms: u32,
        runtime: &mut Runtime,
    ) -> bool {
        if target.object_type == PLAYER_TYPE {
            if !replace_player_rush_state(self, target.id, state, now_ms) {
                return false;
            }
            let Some(mut owner) = self.take_region_owner(region_id) else { return false };
            let moved = self.force_move_owned_shape(
                owner.base_mut(), target, destination_x, destination_y, duration_ms, runtime,
            ).is_some();
            self.restore_region_owner(owner);
            return moved;
        }
        if target.object_type != MONSTER_TYPE { return false }
        let Some(mut owner) = self.take_region_owner(region_id) else { return false };
        let installed = replace_monster_rush_state(
            self, owner.base_mut(), target.id, state, now_ms,
        );
        let moved = installed && self.force_move_owned_shape(
            owner.base_mut(), target, destination_x, destination_y, duration_ms, runtime,
        ).is_some();
        self.restore_region_owner(owner);
        moved
    }
}
