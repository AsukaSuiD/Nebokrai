//! Диспетчер достигнутого семейства сценарных состояний `CMoveShape::AddState`.
//!
//! Точная фабрика из `moveshape.cpp` выбирает семь конкретных классов-владельцев.
//! Этот модуль сохраняет её диапазон и порядок живого списка, но не содержит
//! формул: каждый вариант делегирует расчёт исходному владельцу состояния.
//! `CanonicalStateStorage` владеет экземплярами, а `CGame` только отправляет
//! построенное здесь общее визуальное сообщение `0xBFE03/0xBFE04`. Момент
//! `Begin` и `DWORD`-срок хранятся у общего адаптера: все семь vtable используют
//! один strict wrapping gate `started + keep < timeGetTime()` перед `End`.
//! Vtable-аудит exact EXE подтверждает `CBlindState::GetRemainedTime`
//! (`0x005F2CD0`) у AutoProtect и пяти `UseGoodsEnlarge*`, а `ImproveExp`
//! направляет тот же двухчтенийный контракт на `0x005D5F30`.

use super::autoprotectstate::{AutoProtectState, AUTO_PROTECT_STATE_ID};
use super::improveexpstate::{ImproveExpState, IMPROVE_EXP_STATE_ID};
use super::player::PlayerCombatProperties;
use super::shape::CShape;
use super::usegoodsenlargedefstate::{
    UseGoodsEnlargeDefState, USE_GOODS_ENLARGE_DEF_STATE_ID,
};
use super::usegoodsenlargeelmdefstate::{
    UseGoodsEnlargeElmDefState, USE_GOODS_ENLARGE_ELM_DEF_STATE_ID,
};
use super::usegoodsenlargefullmissstate::{
    UseGoodsEnlargeFullMissState, USE_GOODS_ENLARGE_FULL_MISS_STATE_ID,
};
use super::usegoodsenlargemaxhpstate::{
    UseGoodsEnlargeMaxHpState, USE_GOODS_ENLARGE_MAX_HP_STATE_ID,
};
use super::usegoodsenlargemaxmpstate::{
    UseGoodsEnlargeMaxMpState, USE_GOODS_ENLARGE_MAX_MP_STATE_ID,
};
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::nets::netserver::message::CMessage;

const SCRIPT_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const SCRIPT_STATE_END_MESSAGE: i32 = 0x000b_fe04;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ScriptStateKind {
    EnlargeMaxHp(UseGoodsEnlargeMaxHpState),
    EnlargeMaxMp(UseGoodsEnlargeMaxMpState),
    ImproveExp(ImproveExpState),
    EnlargeDefense(UseGoodsEnlargeDefState),
    EnlargeElementDefense(UseGoodsEnlargeElmDefState),
    EnlargeFullMiss(UseGoodsEnlargeFullMissState),
    AutoProtect(AutoProtectState),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ScriptMoveState {
    kind: ScriptStateKind,
    started_at_ms: u32,
    time_to_keep_ms: u32,
    visual_pending: bool,
}

impl ScriptMoveState {
    /// Повторяет точный диапазон фабрики `CMoveShape::AddState`; знаковые
    /// аргументы сценария сначала сужаются к полям исходных `DWORD`.
    pub(crate) fn from_factory(
        state_id: i32,
        value1: i32,
        value2: i32,
        sufferer_is_gm: bool,
        started_at_ms: u32,
    ) -> Option<Self> {
        let keep_time = value1 as u32;
        let coefficient = value2 as u32;
        let kind = match state_id {
            USE_GOODS_ENLARGE_MAX_HP_STATE_ID => ScriptStateKind::EnlargeMaxHp(
                UseGoodsEnlargeMaxHpState::new(keep_time, coefficient),
            ),
            USE_GOODS_ENLARGE_MAX_MP_STATE_ID => ScriptStateKind::EnlargeMaxMp(
                UseGoodsEnlargeMaxMpState::new(keep_time, coefficient),
            ),
            IMPROVE_EXP_STATE_ID => {
                ScriptStateKind::ImproveExp(ImproveExpState::new(keep_time, coefficient))
            }
            USE_GOODS_ENLARGE_DEF_STATE_ID => ScriptStateKind::EnlargeDefense(
                UseGoodsEnlargeDefState::new(keep_time, coefficient),
            ),
            USE_GOODS_ENLARGE_ELM_DEF_STATE_ID => ScriptStateKind::EnlargeElementDefense(
                UseGoodsEnlargeElmDefState::new(keep_time, coefficient),
            ),
            USE_GOODS_ENLARGE_FULL_MISS_STATE_ID => ScriptStateKind::EnlargeFullMiss(
                UseGoodsEnlargeFullMissState::new(keep_time, coefficient),
            ),
            AUTO_PROTECT_STATE_ID => ScriptStateKind::AutoProtect(AutoProtectState::new(
                value1,
                sufferer_is_gm,
            )?),
            _ => return None,
        };
        Some(Self {
            started_at_ms,
            time_to_keep_ms: keep_time,
            visual_pending: !matches!(kind, ScriptStateKind::AutoProtect(_)),
            kind,
        })
    }

    pub(crate) const fn state_id(self) -> i32 {
        match self.kind {
            ScriptStateKind::EnlargeMaxHp(state) => state.state_id(),
            ScriptStateKind::EnlargeMaxMp(state) => state.state_id(),
            ScriptStateKind::ImproveExp(state) => state.state_id(),
            ScriptStateKind::EnlargeDefense(state) => state.state_id(),
            ScriptStateKind::EnlargeElementDefense(state) => state.state_id(),
            ScriptStateKind::EnlargeFullMiss(state) => state.state_id(),
            ScriptStateKind::AutoProtect(state) => state.state_id(),
        }
    }

    pub(crate) const fn is_auto_protect(self) -> bool {
        matches!(self.kind, ScriptStateKind::AutoProtect(_))
    }

    pub(crate) fn client_state_time(
        self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> i32 {
        match self.kind {
            ScriptStateKind::ImproveExp(_)
            | ScriptStateKind::EnlargeMaxHp(_)
            | ScriptStateKind::EnlargeMaxMp(_)
            | ScriptStateKind::EnlargeDefense(_)
            | ScriptStateKind::EnlargeElementDefense(_)
            | ScriptStateKind::EnlargeFullMiss(_)
            | ScriptStateKind::AutoProtect(_) => timed_client_state_time(
                self.started_at_ms,
                self.time_to_keep_ms,
                now_milliseconds,
            ) as i32,
        }
    }

    /// Exact общий AI-gate сценарных состояний. Сложение и сравнение остаются
    /// `DWORD`, а равенство deadline текущему tick ещё не завершает состояние.
    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.time_to_keep_ms) < now_ms
    }

    pub(crate) fn apply_properties(
        self,
        properties: &mut PlayerCombatProperties,
        auto_protected: &mut bool,
    ) {
        match self.kind {
            ScriptStateKind::EnlargeMaxHp(state) => state.apply(properties),
            ScriptStateKind::EnlargeMaxMp(state) => state.apply(properties),
            ScriptStateKind::ImproveExp(_) => {}
            ScriptStateKind::EnlargeDefense(state) => state.apply(properties),
            ScriptStateKind::EnlargeElementDefense(state) => state.apply(properties),
            ScriptStateKind::EnlargeFullMiss(state) => state.apply(properties),
            ScriptStateKind::AutoProtect(state) => state.apply(auto_protected),
        }
    }

    pub(crate) const fn experience_multiplier_delta(self) -> f32 {
        match self.kind {
            ScriptStateKind::ImproveExp(state) => state.multiplier_delta(),
            _ => 0.0,
        }
    }

    pub(crate) fn take_pending_visual(&mut self) -> bool {
        let pending = self.visual_pending;
        self.visual_pending = false;
        pending
    }
}

pub(crate) fn script_state_visual_message(
    sufferer: &CShape,
    state: ScriptMoveState,
    begin: bool,
    now_milliseconds: impl FnMut() -> u32,
) -> CMessage {
    let identity = sufferer.identity();
    let mut message = CMessage::new(if begin {
        SCRIPT_STATE_BEGIN_MESSAGE
    } else {
        SCRIPT_STATE_END_MESSAGE
    });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.state_id());
    if begin {
        message.add_long(state.client_state_time(now_milliseconds));
        message.add_long(0);
    }
    message
}
