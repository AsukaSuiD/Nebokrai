//! Первичная установка состояния благословения между живыми владельцами.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/godbless{,2}.cpp.
//! God1 сначала завершает первый CExState Original subtype12F и отдельно
//! пересчитывает свойства, без принудительного destructor. Затем оба варианта
//! выбирают первый ID12F/145 без RTTI, ended, уровня или сравнения силы.
//! End → destructor свежего остатка той же позиции → отложенный ctor →
//! Begin(U,S) → append при успехе. Последний UpdateProperty безусловен.
//! Общая арена публикует silent loop1/0 вместе с каноническими U/S до первого
//! property callback. U и S сохраняют независимые регионы; их фактические регионы
//! читаются после часов Begin. Vec/SlotMap заменяют STL/владение указателями.

use super::*;
use crate::gameserver::appserver::moveshape::StateData;
use crate::gameserver::appserver::skills::godblessstate::{GodBlessState, GOD_BLESS_STATE_ID};
use crate::gameserver::appserver::skills::godblessstate2::GOD_BLESS_STATE_2_ID;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, end_move_shape_state,
    resolve_state_move_shape, resolve_state_move_shape_mut,
};

impl CGame {
    pub(crate) fn install_god_bless_state<Runtime: GameMainLoopRuntime>(
        &mut self,
        source: (i32, ShapeIdentity),
        target: (i32, ShapeIdentity),
        skill_id: u32,
        create: impl FnOnce() -> GodBlessState,
        runtime: &mut Runtime,
    ) -> bool {
        if skill_id == GOD_BLESS_STATE_ID {
            let previous = resolve_state_move_shape(self, target.0, target.1).and_then(|shape| {
                shape.find_state_position(|state| matches!(state, StateData::Extended(state)
                    if state.kind == ExtendedStateKind::Original
                        && u32::from(state.state_type) == GOD_BLESS_STATE_ID))
            });
            if let Some((_, key)) = previous {
                let _ = end_move_shape_state(self, target.0, target.1, key);
                let _ = self.update_move_shape_properties(target.0, target.1);
            }
        }
        let previous = resolve_state_move_shape(self, target.0, target.1).and_then(|shape| {
            shape.find_state_position(|state| matches!(state.state_id(),
                GOD_BLESS_STATE_ID | GOD_BLESS_STATE_2_ID))
        });
        if let Some((position, _)) = previous {
            let _ = end_and_destroy_state_at(self, target.0, target.1, position);
        }

        let mut state = create();
        let user_exists = resolve_state_move_shape(self, source.0, source.1).is_some();
        let sufferer_exists = resolve_state_move_shape(self, target.0, target.1).is_some();
        let begun = state.begin_for_install(user_exists, sufferer_exists, &mut || runtime.now_milliseconds());
        let installed = if begun {
            let user = resolve_state_move_shape(self, source.0, source.1).map(|shape| {
                let shape = shape.shape();
                (shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() })
            });
            let sufferer = resolve_state_move_shape(self, target.0, target.1).map(|shape| {
                let shape = shape.shape();
                (shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() })
            });
            resolve_state_move_shape_mut(self, target.0, target.1).is_some_and(|shape| {
                let record = state.encoded_for_install();
                let key = shape.append_applied_state_record(state, &record);
                shape.set_applied_state_user(key, user);
                shape.set_applied_state_sufferer(key, sufferer);
                true
            })
        } else {
            false
        };
        let _ = self.update_move_shape_properties(target.0, target.1);
        installed
    }

}
