//! Данные и сохраняемая запись CTianShenXiaFanState (0x335) в Zone.
//! Источник: gameserver.exe + GameServer.pdb,
//! appserver/skills/tianshenxiafanstate.cpp/.h.
//! Конструктор VA 0x00605780 записывает ID 0x335, срок (+0x38) и уровень
//! (+0x3C) из аргументов без чтения часов. Vtable 0x0066202C: Begin
//! 0x00605B70, AI 0x005D5BA0 (общее истечение), End 0x006059A0,
//! OnUpdateProperties 0x00605A10, GetRemainedTime 0x00601200 (константный
//! ноль), Serialize 0x006059C0 и Unserialize 0x00605C20.
//! Serialize пишет ID, поле +0x2C как есть и уровень — три DWORD (12 байт).
//! Unserialize асимметричен и часов не читает: WORD в timestamp (+0x2C),
//! затем DWORD со смещением +6 записи в level; keep не меняет, поэтому
//! загруженная запись несёт keep=0 от factory, а вход продвигается на
//! 10 байт при 12 байтах записи. Этот исходный дефект сохранён.

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const TIAN_SHEN_XIA_FAN_STATE_ID: u32 = 0x335;
pub const TIAN_SHEN_XIA_FAN_STATE_BYTES: usize = 12;

const TARGET_ELEMENT_MODIFY_GAIN: u32 = 115;
const TARGET_MINIMUM_ATTACK_GAIN: u32 = 116;
const TARGET_MAXIMUM_ATTACK_GAIN: u32 = 117;
const TARGET_DEFENSE_GAIN: u32 = 109;
const TARGET_ELEMENT_RESISTANCE_GAIN: u32 = 112;
const PHYSICAL_AVOID_GAIN: u32 = 130;
const MAGIC_AVOID_GAIN: u32 = 131;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TianShenXiaFanState {
    started_at_ms: u32,
    keep_time_ms: u32,
    level: i32,
}

/// Семь свойств игрока, которые меняет OnUpdateProperties.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TianShenXiaFanPlayerView {
    pub element_modify: i32,
    pub minimum_attack: u32,
    pub maximum_attack: u32,
    pub defense: u32,
    pub element_resistance: u32,
    pub attack_avoid: u16,
    pub element_avoid: u16,
}

impl TianShenXiaFanState {
    pub const fn new(started_at_ms: u32, keep_time_ms: u32, level: i32) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
            level,
        }
    }

    /// Исходный асимметричный Unserialize: WORD в timestamp, DWORD level
    /// со смещением +6; часов нет, keep остаётся заводским нулём factory.
    pub fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != TIAN_SHEN_XIA_FAN_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        let started_at_ms = u32::from(reader.read_u16()?);
        let level = reader.read_i32()?;
        Ok(Self::new(started_at_ms, 0, level))
    }

    pub const fn state_id(self) -> u32 {
        TIAN_SHEN_XIA_FAN_STATE_ID
    }

    pub const fn level(self) -> i32 {
        self.level
    }

    pub const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    /// GetRemainedTime константно нулевой: клиентский пакет пишет нули.
    pub const fn client_time(self) -> u32 {
        0
    }

    /// Serialize записывает поле timestamp как есть (эхо загруженного WORD),
    /// без вызова GetRemainedTime.
    pub fn encoded(self) -> [u8; TIAN_SHEN_XIA_FAN_STATE_BYTES] {
        let mut bytes = [0; TIAN_SHEN_XIA_FAN_STATE_BYTES];
        bytes[0..4].copy_from_slice(&TIAN_SHEN_XIA_FAN_STATE_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&self.started_at_ms.to_le_bytes());
        bytes[8..12].copy_from_slice(&self.level.to_le_bytes());
        bytes
    }

    /// Формулы OnUpdateProperties: семь запросов свойств навыка, затем
    /// прибавки с потолком INT_MAX для четырёх DWORD и wrapping u16 для
    /// двух уклонений. Отсутствующая запись навыка оставляет вид без изменений.
    pub fn apply_to_player_view(
        self,
        mut view: TianShenXiaFanPlayerView,
        query_property: Option<impl FnMut(u32) -> u32>,
    ) -> TianShenXiaFanPlayerView {
        let Some(mut query_property) = query_property else {
            return view;
        };
        let element = query_property(TARGET_ELEMENT_MODIFY_GAIN);
        let minimum = query_property(TARGET_MINIMUM_ATTACK_GAIN);
        let maximum = query_property(TARGET_MAXIMUM_ATTACK_GAIN);
        let defense = query_property(TARGET_DEFENSE_GAIN);
        let resistance = query_property(TARGET_ELEMENT_RESISTANCE_GAIN);
        let physical_avoid = query_property(PHYSICAL_AVOID_GAIN);
        let magic_avoid = query_property(MAGIC_AVOID_GAIN);
        view.element_modify = view.element_modify.wrapping_add(element as i32);
        view.minimum_attack = capped_gain(view.minimum_attack, minimum);
        view.maximum_attack = capped_gain(view.maximum_attack, maximum);
        view.defense = capped_gain(view.defense, defense);
        view.element_resistance = capped_gain(view.element_resistance, resistance);
        view.attack_avoid = view.attack_avoid.wrapping_add(physical_avoid as u16);
        view.element_avoid = view.element_avoid.wrapping_add(magic_avoid as u16);
        view
    }
}

const fn capped_gain(value: u32, gain: u32) -> u32 {
    let result = value.wrapping_add(gain);
    if result > i32::MAX as u32 {
        i32::MAX as u32
    } else {
        result
    }
}
