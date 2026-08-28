//! Межвладельческая координация печати `CSeal`.
//!
//! Конкретная семантика навыка и состояния остаётся у `seal.rs` и
//! `sealstate.rs`; `CGame` здесь только временно извлекает регион, чтобы
//! атомарно заменить состояние монстра и сохранить порядок сетевой доставки.

use super::CGame;
use crate::gameserver::appserver::skills::sealstate::{
    SealState, replace_monster_seal_state,
};

impl CGame {
    pub(crate) fn replace_owned_monster_seal_state(
        &mut self,
        region_id: i32,
        monster_id: i32,
        state: SealState,
        now_ms: u32,
    ) -> bool {
        let Some(mut owner) = self.take_region_owner(region_id) else {
            return false;
        };
        let replaced = replace_monster_seal_state(self, owner.base_mut(), monster_id, state, now_ms);
        self.restore_region_owner(owner);
        replaced
    }
}
