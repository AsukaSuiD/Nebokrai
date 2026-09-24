//! Данные и записи extended-state `CExState` (0x32) / `CExStateNew` (0x33) в Zone.
//! Источник: gameserver.exe + GameServer.pdb,
//! appserver/other states/{exstate,exstatenew}.cpp/.h.
//! Конструкторы VA `0x005D9230`/`0x005D9990` часов не читают (base timestamp
//! и New.last_item_tick нулевые). Serialize VA `0x005D9510`/`0x005D9BB0`
//! записывают ID, вызывают GetRemainedTime и кладут остаток в живой keep
//! (+0x40), затем добавляют блок 0x28/0x34 байта. Unserialize VA
//! `0x005D9550`/`0x005D9BF0` после внешнего ID читают clock до блока;
//! New ставит его и в started, и в item timestamp. AI VA
//! `0x005D94E0`/`0x005D9FB0` сравнивают абсолютный wrapping-deadline строго
//! с текущим tick. Getter Original — общий с CHBYState (после истечения
//! ненулевого срока возвращает 1), New — `0x005D6320` (нулевой срок без
//! часов). Живые Add/Del/Begin/End и periodic use_item остаются у
//! переходного Game.

use super::time::{change_body_client_state_time, guarded_client_state_time};

pub const EX_STATE_ID: u32 = 0x32;
pub const EX_STATE_NEW_ID: u32 = 0x33;
pub const EX_STATE_BYTES: usize = 40;
pub const EX_STATE_NEW_BYTES: usize = 52;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExtendedStateKind {
    Original,
    New,
}

impl ExtendedStateKind {
    pub const fn state_id(self) -> u32 {
        match self {
            Self::Original => EX_STATE_ID,
            Self::New => EX_STATE_NEW_ID,
        }
    }

    pub const fn parameter_bytes(self) -> usize {
        match self {
            Self::Original => EX_STATE_BYTES,
            Self::New => EX_STATE_NEW_BYTES,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtendedState {
    pub kind: ExtendedStateKind,
    pub state_type: u16,
    pub level: u32,
    pub keep_time_ms: u32,
    pub maximum_hp: u16,
    pub maximum_mp: u16,
    pub minimum_attack: u16,
    pub maximum_attack: u16,
    pub element_modify: u16,
    pub defense: u16,
    pub element_resistance: u16,
    pub cch: u16,
    pub full_miss: u16,
    pub attack_avoid: u16,
    pub element_avoid: u16,
    pub hit: u16,
    pub dodge: u16,
    pub item_index: u32,
    pub item_amount: u32,
    pub frequency_ms: u32,
    pub started_ms: u32,
    pub last_item_tick_ms: u32,
    serialized_offset: Option<usize>,
}

impl ExtendedState {
    /// Factory snapshot; порядок запросов свойств сохранён. Нулевой уровень
    /// не создаёт состояния; отсутствие записи навыка решает вызывающий.
    pub fn from_properties(
        kind: ExtendedStateKind,
        level: u32,
        mut query_property: impl FnMut(u32) -> u32,
    ) -> Option<Self> {
        if level == 0 {
            return None;
        }
        let p = &mut query_property;
        Some(Self {
            kind,
            state_type: p(20_010) as u16,
            level,
            keep_time_ms: p(10_002),
            maximum_hp: p(118) as u16,
            maximum_mp: p(119) as u16,
            minimum_attack: p(116) as u16,
            maximum_attack: p(117) as u16,
            element_modify: p(115) as u16,
            defense: p(109) as u16,
            element_resistance: p(112) as u16,
            cch: p(108) as u16,
            full_miss: p(127) as u16,
            attack_avoid: p(128) as u16,
            element_avoid: p(129) as u16,
            hit: p(20_001) as u16,
            dodge: p(110) as u16,
            item_index: (kind == ExtendedStateKind::New)
                .then(|| p(50_001))
                .unwrap_or(0),
            item_amount: (kind == ExtendedStateKind::New)
                .then(|| p(50_002))
                .unwrap_or(0),
            frequency_ms: (kind == ExtendedStateKind::New)
                .then(|| p(6_001))
                .unwrap_or(0),
            started_ms: 0,
            last_item_tick_ms: 0,
            serialized_offset: None,
        })
    }

    pub fn begin_primary_at(&mut self, now_ms: u32) {
        self.started_ms = now_ms;
    }

    /// Unserialize: внешний ID уже прочитан factory; один clock до блока.
    pub fn decode_at(payload: &[u8], offset: usize, now: &mut dyn FnMut() -> u32) -> Option<Self> {
        let kind = match read_u32(payload, offset)? {
            EX_STATE_ID => ExtendedStateKind::Original,
            EX_STATE_NEW_ID => ExtendedStateKind::New,
            _ => return None,
        };
        let base = offset.checked_add(4)?;
        let block = payload.get(base..base.checked_add(kind.parameter_bytes())?)?;
        let level = read_u32(block, 4)?;
        if level == 0 {
            return None;
        }
        let now_ms = now();
        Some(Self {
            kind,
            state_type: read_u16(block, 0)?,
            level,
            keep_time_ms: read_u32(block, 8)?,
            maximum_hp: read_u16(block, 12)?,
            maximum_mp: read_u16(block, 14)?,
            minimum_attack: read_u16(block, 16)?,
            maximum_attack: read_u16(block, 18)?,
            element_modify: read_u16(block, 20)?,
            defense: read_u16(block, 22)?,
            element_resistance: read_u16(block, 24)?,
            cch: read_u16(block, 26)?,
            full_miss: read_u16(block, 28)?,
            attack_avoid: read_u16(block, 30)?,
            element_avoid: read_u16(block, 32)?,
            hit: read_u16(block, 34)?,
            dodge: read_u16(block, 36)?,
            item_index: (kind == ExtendedStateKind::New)
                .then(|| read_u32(block, 40).unwrap_or_default())
                .unwrap_or(0),
            item_amount: (kind == ExtendedStateKind::New)
                .then(|| read_u32(block, 44).unwrap_or_default())
                .unwrap_or(0),
            frequency_ms: (kind == ExtendedStateKind::New)
                .then(|| read_u32(block, 48).unwrap_or_default())
                .unwrap_or(0),
            started_ms: now_ms,
            last_item_tick_ms: now_ms,
            serialized_offset: Some(offset),
        })
    }

    pub const fn state_id(&self) -> u32 {
        self.kind.state_id()
    }

    pub fn serialized_span(&self) -> Option<(usize, usize)> {
        self.serialized_offset
            .map(|offset| (offset, 4 + self.kind.parameter_bytes()))
    }

    /// AI: строгий абсолютный wrapping-deadline; нулевой keep не истекает.
    pub fn expired(&self, now_ms: u32) -> bool {
        self.keep_time_ms != 0 && self.started_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    /// Клиентский остаток различает владельцев: Original делит getter с
    /// CHBYState (1 после истечения ненулевого срока), New возвращает 0.
    pub fn client_state_time(&self, now: &mut dyn FnMut() -> u32) -> u32 {
        match self.kind {
            ExtendedStateKind::Original => {
                change_body_client_state_time(self.started_ms, self.keep_time_ms, now)
            }
            ExtendedStateKind::New => {
                guarded_client_state_time(self.started_ms, self.keep_time_ms, now)
            }
        }
    }

    pub fn remaining_time_ms(&self, now_ms: u32) -> u32 {
        let deadline = self.started_ms.wrapping_add(self.keep_time_ms);
        match self.kind {
            ExtendedStateKind::Original => {
                if self.keep_time_ms != 0 && deadline <= now_ms {
                    1
                } else if deadline <= now_ms {
                    0
                } else {
                    deadline.wrapping_sub(now_ms)
                }
            }
            ExtendedStateKind::New => {
                if self.keep_time_ms == 0 || deadline <= now_ms {
                    0
                } else {
                    deadline.wrapping_sub(now_ms)
                }
            }
        }
    }

    pub fn item_due(&mut self, now_ms: u32) -> bool {
        if self.kind == ExtendedStateKind::New && self.last_item_tick_ms == 0 {
            self.last_item_tick_ms = self.started_ms;
        }
        self.kind == ExtendedStateKind::New
            && self.frequency_ms != 0
            && self.item_index != 0
            && self.item_amount != 0
            && self.last_item_tick_ms.wrapping_add(self.frequency_ms) < now_ms
    }

    pub fn restart_item_clock(&mut self, now_ms: u32) {
        self.last_item_tick_ms = now_ms;
    }

    pub fn encoded_for_install(&self) -> Vec<u8> {
        let mut payload = vec![0; 4 + self.kind.parameter_bytes()];
        let base = 4;
        write_u32(&mut payload, 0, self.state_id());
        write_u16(&mut payload, base, self.state_type);
        write_u32(&mut payload, base + 4, self.level);
        write_u32(&mut payload, base + 8, self.keep_time_ms);
        for (position, value) in [
            (12, self.maximum_hp),
            (14, self.maximum_mp),
            (16, self.minimum_attack),
            (18, self.maximum_attack),
            (20, self.element_modify),
            (22, self.defense),
            (24, self.element_resistance),
            (26, self.cch),
            (28, self.full_miss),
            (30, self.attack_avoid),
            (32, self.element_avoid),
            (34, self.hit),
            (36, self.dodge),
        ] {
            write_u16(&mut payload, base + position, value);
        }
        if self.kind == ExtendedStateKind::New {
            write_u32(&mut payload, base + 40, self.item_index);
            write_u32(&mut payload, base + 44, self.item_amount);
            write_u32(&mut payload, base + 48, self.frequency_ms);
        }
        payload
    }

    /// Save меняет только остаток; padding-байты загруженной записи
    /// не заменяются нулями.
    pub fn update_serialized_record(&self, payload: &mut [u8], offset: usize, remaining: u32) {
        if offset
            .checked_add(4 + self.kind.parameter_bytes())
            .is_some_and(|end| end <= payload.len())
        {
            write_u32(payload, offset + 12, remaining);
        }
    }

    pub fn commit_serialized_time(&mut self, remaining: u32) {
        self.keep_time_ms = remaining;
    }

    pub fn shift_serialized_offset_for_insert(&mut self, inserted_offset: usize, amount: usize) {
        if let Some(offset) = &mut self.serialized_offset
            && *offset >= inserted_offset
        {
            *offset += amount;
        }
    }

    pub fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) {
        if self
            .serialized_offset
            .is_some_and(|offset| removed_offset < offset)
        {
            self.serialized_offset = self.serialized_offset.map(|offset| offset - amount);
        }
    }
}

fn read_u16(source: &[u8], offset: usize) -> Option<u16> {
    let bytes = source.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_le_bytes(bytes.try_into().ok()?))
}

fn read_u32(source: &[u8], offset: usize) -> Option<u32> {
    let bytes = source.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

fn write_u16(destination: &mut [u8], offset: usize, value: u16) {
    destination[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(destination: &mut [u8], offset: usize, value: u32) {
    destination[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}
