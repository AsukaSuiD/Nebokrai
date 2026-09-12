//! Межвладельческое применение оглушения и отбрасывания `CMosou`.
//!
//! Формула, вероятность и выбор цели принадлежат skill-owner-у. `CGame`
//! временно извлекает регион только для атомарной работы канонического
//! состояния player/monster, пространственного индекса, сети и AI ожидания.

use super::*;
use crate::gameserver::appserver::skills::knockoutstate::{
    KnockOutState, replace_monster_knock_out_state, replace_player_knock_out_state,
};

impl CGame {
    pub(crate) fn apply_mosou_control(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        state: KnockOutState,
        destination_x: i32,
        destination_y: i32,
        duration_ms: u32,
        now_ms: u32,
    ) -> bool {
        if target.object_type == PLAYER_TYPE {
            if !replace_player_knock_out_state(self, target.id, state, now_ms) {
                return false;
            }
            return self.force_move_skill_target(
                region_id, target, destination_x, destination_y, duration_ms,
            ).is_some();
        }
        if target.object_type != MONSTER_TYPE { return false }
        let Some(mut owner) = self.take_region_owner(region_id) else { return false };
        let installed = replace_monster_knock_out_state(self, owner.base_mut(), target.id, state, now_ms);
        let moved = installed && self.force_move_owned_monster(
            owner.base_mut(), target.id, destination_x, destination_y, duration_ms,
        ).is_some();
        self.restore_region_owner(owner);
        moved
    }
}
