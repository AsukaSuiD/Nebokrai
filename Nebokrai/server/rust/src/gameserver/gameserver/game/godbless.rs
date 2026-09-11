//! Межвладельческая первичная установка `GodBlessState`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходные владельцы
//! `appserver/skills/godbless.cpp` и `godbless2.cpp`. GodBless1 сначала
//! вызывает DelExStateByType(0x12F) (0x005B070E → 0x004CEA30): первый
//! CExState этого subtype получает End и отдельный внешний UpdateProperty,
//! без принудительного destructor. У GodBless2 этого шага нет.
//! Оба AI (0x005B0721/0x00550CA0) выбирают первый ID 0x12F или 0x145
//! в живом порядке списка, без сравнения уровня, силы и срока. Затем End
//! (0x005B0810/0x00550D62) → destructor свежего остатка той же позиции →
//! новый ctor → Begin(user,sufferer) → append только при успешном Begin.
//! Общий UpdateProperty вызывается и после отказа Begin; старое состояние
//! не восстанавливается. Нехватка памяти использует стандартную политику
//! Rust, а не эмуляцию native allocation failure.
//! Между Begin и append нет callback: общий arena-owner создаёт silent
//! visual loop=1/0 при передаче payload. Первый BFE03 принадлежит следующему
//! OnUpdateProperties. Источник, цель и их настоящий AI остаются доступны
//! во время старого End и каждого property callback; регион не извлекается.

use super::*;
use crate::gameserver::appserver::moveshape::StateData;
use crate::gameserver::appserver::skills::godblessstate::{GodBlessState, GOD_BLESS_STATE_ID};
use crate::gameserver::appserver::skills::godblessstate2::GOD_BLESS_STATE_2_ID;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, end_move_shape_state,
    resolve_state_move_shape, resolve_state_move_shape_mut,
};

impl CGame {
    #[allow(clippy::too_many_arguments, reason = "граница хранит отдельно variant, actual user, sufferer и отложенный ctor")]
    pub(crate) fn install_god_bless_state<Runtime: GameMainLoopRuntime>(
        &mut self,
        region_id: i32,
        target: ShapeIdentity,
        user: ShapeIdentity,
        skill_id: u32,
        create: impl FnOnce() -> GodBlessState,
        runtime: &mut Runtime,
    ) -> bool {
        if skill_id == GOD_BLESS_STATE_ID {
            let previous = resolve_state_move_shape(self, region_id, target).and_then(|shape| {
                shape.find_state_position(|state| matches!(state, StateData::Extended(state)
                    if state.kind == ExtendedStateKind::Original
                        && u32::from(state.state_type) == GOD_BLESS_STATE_ID))
            });
            if let Some((_, key)) = previous {
                let _ = end_move_shape_state(self, region_id, target, key);
                let _ = self.update_move_shape_properties(region_id, target);
            }
        }
        let previous = resolve_state_move_shape(self, region_id, target).and_then(|shape| {
            shape.find_state_position(|state| matches!(state.state_id(),
                GOD_BLESS_STATE_ID | GOD_BLESS_STATE_2_ID))
        });
        if let Some((position, _)) = previous {
            let _ = end_and_destroy_state_at(self, region_id, target, position);
        }

        let mut state = create();
        let source = resolve_state_move_shape(self, region_id, user)
            .map(|shape| (shape.shape().get_region_id(), user));
        let sufferer_exists = resolve_state_move_shape(self, region_id, target).is_some();
        let begun = state.begin_for_install(source.is_some(), sufferer_exists, &mut || runtime.now_milliseconds());
        let installed = if begun {
            resolve_state_move_shape_mut(self, region_id, target).is_some_and(|shape| {
                let record = state.encoded_for_install();
                let key = shape.append_applied_state_record(state, &record);
                shape.mark_applied_state_begun(key);
                shape.set_applied_state_user(key, source);
                true
            })
        } else {
            false
        };
        let _ = self.update_move_shape_properties(region_id, target);
        installed
    }

}
