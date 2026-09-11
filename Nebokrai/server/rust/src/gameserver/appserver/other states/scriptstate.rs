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
//! AutoProtect использует общий 8-байтный DB-кодек `CBlindState`, остальные
//! шесть concrete vtable — 12-байтный кодек `ID + остаток + DWORD`.
//! restart_script_move_state переносит только Begin(NULL, holder):
//! шесть UseGoods/ImproveExp object Begin (0x005D52F0/0x005D4E30/
//! 0x005D4930/0x005D5C90/0x005D57A0/0x005D60F0) сразу возвращают false
//! при NULL user, не трогая base/visual. AutoProtect0x005D4290 требует
//! sufferer и отклоняет только CPlayer GM: base Begin → visual SetRun(1),
//! без Update/пакета и часов. Timestamp сохраняет отдельный clock Unserialize.
//! OnUpdateProperties пяти UseGoods получает sufferer, обновляет visual и
//! лишь затем проверяет player type для формулы; user не является gate.
//! ImproveExp0x005D5F10 вызывает только visual, без EXP-формулы и owner-gate.
//! AutoProtect0x005D4240 сначала ставит флаг игроку не-GM, затем visual.
//! Все эти visual проверяют собственный ended и сохраняют base tail даже
//! без цели/пакета; очередь отложенных visual не заменяет живой вызов +0x24.

use super::autoprotectstate::{AutoProtectState, AUTO_PROTECT_STATE_ID};
use super::improveexpstate::{ImproveExpState, IMPROVE_EXP_STATE_ID};
use super::player::PlayerCombatProperties;
use super::shape::{CShape, ShapeIdentity};
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
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::states::state::{
    timed_client_state_time, begin_base_applied_state, begin_applied_state_visual,
    resolve_state_move_shape, resolve_applied_state_sufferer,
    update_property_state_visual, StatePropertyTarget,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

const SCRIPT_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const SCRIPT_STATE_END_MESSAGE: i32 = 0x000b_fe04;
pub(crate) const SCRIPT_STATE_TIMED_BYTES: usize = 12;
pub(crate) const AUTO_PROTECT_STATE_BYTES: usize = 8;

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
    serialized_value: Option<u32>,
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
            serialized_value: (!matches!(kind, ScriptStateKind::AutoProtect(_)))
                .then_some(coefficient),
            kind,
        })
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let state_id = reader.read_i32()?;
        let keep_time = reader.read_u32()?;
        let value = if state_id == AUTO_PROTECT_STATE_ID {
            0
        } else {
            reader.read_u32()?
        };
        Self::from_factory(state_id, keep_time as i32, value as i32, false, now_ms)
            .ok_or(
            LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            },
        )
    }

    pub(crate) const fn serialized_size(state_id: i32) -> Option<usize> {
        match state_id {
            AUTO_PROTECT_STATE_ID => Some(AUTO_PROTECT_STATE_BYTES),
            USE_GOODS_ENLARGE_MAX_HP_STATE_ID
            | USE_GOODS_ENLARGE_MAX_MP_STATE_ID
            | IMPROVE_EXP_STATE_ID
            | USE_GOODS_ENLARGE_DEF_STATE_ID
            | USE_GOODS_ENLARGE_ELM_DEF_STATE_ID
            | USE_GOODS_ENLARGE_FULL_MISS_STATE_ID => Some(SCRIPT_STATE_TIMED_BYTES),
            _ => None,
        }
    }


    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::serialized_size(self.state_id()).unwrap_or(0));
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_i32(self.state_id());
        writer.write_u32(self.client_state_time(now_milliseconds) as u32);
        if let Some(value) = self.serialized_value {
            writer.write_u32(value);
        }
        bytes
    }

    pub(crate) fn encoded_for_install(self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(Self::serialized_size(self.state_id()).unwrap_or(0));
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_i32(self.state_id());
        writer.write_u32(self.time_to_keep_ms);
        if let Some(value) = self.serialized_value {
            writer.write_u32(value);
        }
        bytes
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

    fn apply_combat_properties(self, properties: &mut PlayerCombatProperties) {
        match self.kind {
            ScriptStateKind::EnlargeMaxHp(state) => state.apply(properties),
            ScriptStateKind::EnlargeMaxMp(state) => state.apply(properties),
            ScriptStateKind::ImproveExp(_) => {}
            ScriptStateKind::EnlargeDefense(state) => state.apply(properties),
            ScriptStateKind::EnlargeElementDefense(state) => state.apply(properties),
            ScriptStateKind::EnlargeFullMiss(state) => state.apply(properties),
            ScriptStateKind::AutoProtect(_) => {}
        }
    }

    pub(crate) fn experience_multiplier_delta(self) -> f64 {
        match self.kind {
            ScriptStateKind::ImproveExp(state) => state.multiplier_delta(),
            _ => 0.0,
        }
    }

}

pub(crate) fn update_script_move_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ScriptMoveState>(key)).copied()
    else { return false };
    match state.kind {
        ScriptStateKind::AutoProtect(_) => {
            let Some((_, target)) = resolve_applied_state_sufferer(
                game, region_id, holder, key,
            ) else { return true };
            if target.object_type != 400
                || game.script_player_gm_level(target.id).unwrap_or(0) != 0
            {
                return true;
            }
            let Some(player) = game.find_player_mut(target.id) else { return true };
            player.set_auto_protected(true);
            let _ = update_property_state_visual::<ScriptMoveState>(
                game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
                |state, now| state.client_state_time(now) as u32,
            );
        }
        ScriptStateKind::ImproveExp(_) => {
            let _ = update_property_state_visual::<ScriptMoveState>(
                game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
                |state, now| state.client_state_time(now) as u32,
            );
        }
        _ => {
            if resolve_applied_state_sufferer(game, region_id, holder, key).is_none() {
                return false;
            }
            let _ = update_property_state_visual::<ScriptMoveState>(
                game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
                |state, now| state.client_state_time(now) as u32,
            );
            let Some(state) = resolve_state_move_shape(game, region_id, holder)
                .and_then(|shape| shape.applied_state::<ScriptMoveState>(key)).copied()
            else { return false };
            let Some((_, target)) = resolve_applied_state_sufferer(
                game, region_id, holder, key,
            ) else { return false };
            if target.object_type == 400 {
                let Some(player) = game.find_player_mut(target.id) else { return false };
                player.update_state_combat_properties(|mut properties| {
                    state.apply_combat_properties(&mut properties);
                    properties
                });
            }
        }
    }
    true
}

pub(crate) fn restart_script_move_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<ScriptMoveState>(key))
    else { return false };
    if !state.is_auto_protect()
        || (holder.object_type == 400
            && game.script_player_gm_level(holder.id).unwrap_or(0) != 0)
    {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) { return false }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    true
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
