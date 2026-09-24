//! Данные и записи семи состояний `CMoveShape::AddState` в Zone:
//! пять UseGoods, ImproveExp и AutoProtect.
//! Источник: gameserver.exe + GameServer.pdb, moveshape.cpp и
//! other states/{usegoodsenlarge*,improveexp,autoprotect}state.cpp/.h.
//! Конструкторы не читают часы (started и счётчик нулевые). Vtable:
//! AutoProtect `0x0065E00C` (End `0x005D44E0`, Serialize `0x005F51E0` —
//! 8 байт ID/remaining), пять UseGoods (End `0x005D5B80` с ended-флагом,
//! Serialize `0x005D4D10` — 12 байт ID/remaining/coefficient, getter
//! `0x005F2CD0`) и ImproveExp (getter `0x005D5F30`, Serialize
//! `0x005E7330` — 12 байт, AI `0x005D60B0`). Readers
//! `0x004F9D80`/`0x005D6190`/`0x005EAAC0` читают clock после внешнего ID,
//! до срока и коэффициента. Save не меняет живой keep. Живые
//! Begin/restart/AI/End, property-формулы и пакеты остаются у
//! переходного Game.

use super::time::timed_client_state_time;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};

pub const USE_GOODS_ENLARGE_MAX_HP_STATE_ID: i32 = 100_007;
pub const USE_GOODS_ENLARGE_MAX_MP_STATE_ID: i32 = 100_008;
pub const IMPROVE_EXP_STATE_ID: i32 = 100_009;
pub const USE_GOODS_ENLARGE_DEF_STATE_ID: i32 = 100_010;
pub const USE_GOODS_ENLARGE_ELM_DEF_STATE_ID: i32 = 100_011;
pub const USE_GOODS_ENLARGE_FULL_MISS_STATE_ID: i32 = 100_012;
pub const AUTO_PROTECT_STATE_ID: i32 = 110_000;

pub const SCRIPT_STATE_TIMED_BYTES: usize = 12;
pub const AUTO_PROTECT_STATE_BYTES: usize = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ScriptStateKind {
    EnlargeMaxHp(u32),
    EnlargeMaxMp(u32),
    ImproveExp(u32),
    EnlargeDefense(u32),
    EnlargeElementDefense(u32),
    EnlargeFullMiss(u32),
    AutoProtect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScriptMoveState {
    kind: ScriptStateKind,
    started_at_ms: u32,
    time_to_keep_ms: u32,
}

impl ScriptMoveState {
    pub fn from_factory(state_id: i32, value1: i32, value2: i32) -> Option<Self> {
        let coefficient = value2 as u32;
        let kind = match state_id {
            USE_GOODS_ENLARGE_MAX_HP_STATE_ID => ScriptStateKind::EnlargeMaxHp(coefficient),
            USE_GOODS_ENLARGE_MAX_MP_STATE_ID => ScriptStateKind::EnlargeMaxMp(coefficient),
            IMPROVE_EXP_STATE_ID => ScriptStateKind::ImproveExp(coefficient),
            USE_GOODS_ENLARGE_DEF_STATE_ID => ScriptStateKind::EnlargeDefense(coefficient),
            USE_GOODS_ENLARGE_ELM_DEF_STATE_ID => {
                ScriptStateKind::EnlargeElementDefense(coefficient)
            }
            USE_GOODS_ENLARGE_FULL_MISS_STATE_ID => ScriptStateKind::EnlargeFullMiss(coefficient),
            AUTO_PROTECT_STATE_ID => ScriptStateKind::AutoProtect,
            _ => return None,
        };
        Some(Self {
            kind,
            started_at_ms: 0,
            time_to_keep_ms: value1 as u32,
        })
    }

    /// Reader читает clock после внешнего ID и до срока с коэффициентом.
    pub fn decode(
        payload: &[u8],
        offset: usize,
        now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let state_id = reader.read_i32()?;
        let started_at_ms = now();
        let keep_time = reader.read_u32()?;
        let value = if state_id == AUTO_PROTECT_STATE_ID {
            0
        } else {
            reader.read_u32()?
        };
        let mut state = Self::from_factory(state_id, keep_time as i32, value as i32).ok_or(
            LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            },
        )?;
        state.started_at_ms = started_at_ms;
        Ok(state)
    }

    pub const fn serialized_size(state_id: i32) -> Option<usize> {
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

    pub fn kind(&self) -> &ScriptStateKind {
        &self.kind
    }

    fn coefficient(&self) -> Option<u32> {
        match &self.kind {
            ScriptStateKind::EnlargeMaxHp(value)
            | ScriptStateKind::EnlargeMaxMp(value)
            | ScriptStateKind::ImproveExp(value)
            | ScriptStateKind::EnlargeDefense(value)
            | ScriptStateKind::EnlargeElementDefense(value)
            | ScriptStateKind::EnlargeFullMiss(value) => Some(*value),
            ScriptStateKind::AutoProtect => None,
        }
    }

    fn encoded_with_remaining(&self, remaining: u32) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(if self.is_auto_protect() {
            AUTO_PROTECT_STATE_BYTES
        } else {
            SCRIPT_STATE_TIMED_BYTES
        });
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_i32(self.state_id());
        writer.write_u32(remaining);
        if let Some(value) = self.coefficient() {
            writer.write_u32(value);
        }
        bytes
    }

    pub fn encoded(&self, now: impl FnMut() -> u32) -> Vec<u8> {
        self.encoded_with_remaining(self.client_state_time(now) as u32)
    }

    pub fn encoded_for_install(&self) -> Vec<u8> {
        self.encoded_with_remaining(self.time_to_keep_ms)
    }

    pub const fn state_id(&self) -> i32 {
        match &self.kind {
            ScriptStateKind::EnlargeMaxHp(_) => USE_GOODS_ENLARGE_MAX_HP_STATE_ID,
            ScriptStateKind::EnlargeMaxMp(_) => USE_GOODS_ENLARGE_MAX_MP_STATE_ID,
            ScriptStateKind::ImproveExp(_) => IMPROVE_EXP_STATE_ID,
            ScriptStateKind::EnlargeDefense(_) => USE_GOODS_ENLARGE_DEF_STATE_ID,
            ScriptStateKind::EnlargeElementDefense(_) => USE_GOODS_ENLARGE_ELM_DEF_STATE_ID,
            ScriptStateKind::EnlargeFullMiss(_) => USE_GOODS_ENLARGE_FULL_MISS_STATE_ID,
            ScriptStateKind::AutoProtect => AUTO_PROTECT_STATE_ID,
        }
    }

    pub const fn is_auto_protect(&self) -> bool {
        matches!(self.kind, ScriptStateKind::AutoProtect)
    }

    pub const fn is_improve_exp(&self) -> bool {
        matches!(self.kind, ScriptStateKind::ImproveExp(_))
    }

    pub fn begin_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub fn client_state_time(&self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.time_to_keep_ms, now) as i32
    }

    /// Общий AI: unsigned `start + keep < now`, без особого случая keep0.
    pub const fn expired(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.time_to_keep_ms) < now_ms
    }
}
