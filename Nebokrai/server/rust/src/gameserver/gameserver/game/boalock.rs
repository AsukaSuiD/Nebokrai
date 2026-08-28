//! Межвладельческое применение состояний `CBoaLock`.
//!
//! Длительности и условие уровня вычисляет skill-owner. `CGame` временно
//! извлекает регион только для упорядоченной замены канонических состояний
//! игрока либо монстра и их фактической around-доставки.

use super::*;
use crate::gameserver::appserver::skills::boalockstate::{BoaLockState, replace_monster_boa_lock_state, replace_player_boa_lock_state};
use crate::gameserver::appserver::skills::knockoutstate::{KnockOutState, replace_monster_knock_out_state, replace_player_knock_out_state};

impl CGame {
    pub(crate) fn apply_boa_lock_control(&mut self, region_id: i32, target: ShapeIdentity, lock: Option<BoaLockState>, knock_out: KnockOutState, now_ms: u32) -> bool {
        if target.object_type == PLAYER_TYPE { if let Some(lock) = lock { let _ = replace_player_boa_lock_state(self, target.id, lock, now_ms); } return replace_player_knock_out_state(self, target.id, knock_out, now_ms) }
        if target.object_type != MONSTER_TYPE { return false }
        let Some(mut owner) = self.take_region_owner(region_id) else { return false };
        if let Some(lock) = lock { let _ = replace_monster_boa_lock_state(self, owner.base_mut(), target.id, lock, now_ms); }
        let installed = replace_monster_knock_out_state(self, owner.base_mut(), target.id, knock_out, now_ms);
        self.restore_region_owner(owner); installed
    }
}
