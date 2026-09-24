//! Данные и 76-байтная запись `CNotDisappearAfterDead` (внешний ID `0x38`) в Zone.
//! Источник: gameserver.exe + GameServer.pdb,
//! appserver/other states/notdisappearafterdead.cpp/.h.
//! Конструктор VA `0x005D62A0` копирует 72 байта без часов. GetRemainedTime
//! VA `0x005D6320` при нулевом сроке не читает часы, иначе берёт одно значение
//! для проверки границы и второе для вычитания. Serialize VA `0x005D64F0`
//! записывает вычисленный getter-ом остаток в живой keep (+0x40), не меняя
//! started/item timestamp, и затем добавляет 72-байтный блок; writer получает
//! этот же остаток без второго clock. Unserialize VA `0x005D6530` одним clock
//! ставит оба timestamp (+0x2C и +0x80) и копирует все 72 байта, включая
//! innerID 0. Общий AI VA `0x005D7C80` использует строгие wrapping-сроки.
//! Codec сохраняет padding +2..3 и исходный ненулевой BOOL-байт, а не
//! нормализует загруженную запись при сохранении. Живые Begin/AI/End и
//! visual остаются у переходного Game.

use super::time::guarded_client_state_time;

pub const UNDEAD_STATE_ID: u32 = 0x38;
pub const UNDEAD_STATE_PARAMETER_BYTES: usize = 72;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UndeadState {
    state_id: u32,
    state_type: u16,
    keep_time_ms: u32,
    started_ms: u32,
    last_item_tick_ms: u32,
    // ClearAllStates сравнивает сохранённый байт строго с 1, не с нулём.
    pub disappear_after_dead: u8,
    pub percentage: bool,
    pub maximum_hp: i16,
    pub maximum_mp: i16,
    pub minimum_attack: i16,
    pub maximum_attack: i16,
    pub element_modify: i16,
    pub defense: i16,
    pub element_resistance: i16,
    pub blast_attack: i16,
    pub blast_element_attack: i16,
    pub strength: i32,
    pub dexterity: i32,
    pub constitution: i32,
    pub intelligence: i32,
    pub cch: i16,
    pub full_miss: i16,
    pub attack_avoid: i16,
    pub element_avoid: i16,
    pub hit: i16,
    pub dodge: i16,
    item_index: u32,
    item_amount: u32,
    frequency_ms: u32,
    serialized_offset: Option<usize>,
}

impl UndeadState {
    pub const SERIALIZED_BYTES: usize = 4 + UNDEAD_STATE_PARAMETER_BYTES;

    pub const fn state_id(&self) -> u32 {
        self.state_id
    }

    pub const fn state_type(&self) -> u16 {
        self.state_type
    }

    pub const fn keep_time_ms(&self) -> u32 {
        self.keep_time_ms
    }

    pub const fn started_ms(&self) -> u32 {
        self.started_ms
    }

    pub const fn last_item_tick_ms(&self) -> u32 {
        self.last_item_tick_ms
    }

    pub const fn item_index(&self) -> u32 {
        self.item_index
    }

    pub const fn item_amount(&self) -> u32 {
        self.item_amount
    }

    pub const fn frequency_ms(&self) -> u32 {
        self.frequency_ms
    }

    pub fn begin_primary_at(&mut self, now_ms: u32) {
        self.started_ms = now_ms;
    }

    /// AI: нулевой item timestamp получает started до проверки трёх параметров.
    pub fn ensure_item_clock_started(&mut self) {
        if self.last_item_tick_ms == 0 {
            self.last_item_tick_ms = self.started_ms;
        }
    }

    /// AI: строгий wrapping-срок; нулевой keep не истекает.
    pub fn expired(&self, now_ms: u32) -> bool {
        self.keep_time_ms != 0 && self.started_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    /// Тик предмета: нулевой timestamp получает started, условие — тройка
    /// ненулевых параметров и строгая граница.
    pub fn item_due(&mut self, now_ms: u32) -> bool {
        if self.last_item_tick_ms == 0 {
            self.last_item_tick_ms = self.started_ms;
        }
        self.frequency_ms != 0
            && self.item_index != 0
            && self.item_amount != 0
            && self.last_item_tick_ms.wrapping_add(self.frequency_ms) < now_ms
    }

    /// AI: нулевой item timestamp получает started; иначе отдельный clock.
    pub fn set_last_item_tick(&mut self, now_ms: u32) {
        self.last_item_tick_ms = now_ms;
    }

    pub fn client_state_time(&self, now: &mut dyn FnMut() -> u32) -> u32 {
        guarded_client_state_time(self.started_ms, self.keep_time_ms, now)
    }

    /// Serialize записывает вычисленный остаток в живой keep, не меняя started.
    pub fn commit_serialized_time(&mut self, remaining: u32) {
        self.keep_time_ms = remaining;
    }

    /// Factory snapshot: порядок запросов свойств сохранён из исходного вызова.
    pub fn from_properties(state_id: u32, mut query_property: impl FnMut(u32) -> u32) -> Self {
        let p = &mut query_property;
        Self {
            state_id,
            state_type: p(SKILL_USAGE_CONST) as u16,
            keep_time_ms: p(SKILL_USAGE_STATE_PERSIST_TIME),
            started_ms: 0,
            last_item_tick_ms: 0,
            disappear_after_dead: u8::from(p(80_001) != 0),
            percentage: p(80_002) != 0,
            maximum_hp: p(118) as i16,
            maximum_mp: p(119) as i16,
            minimum_attack: p(116) as i16,
            maximum_attack: p(117) as i16,
            element_modify: p(115) as i16,
            defense: p(109) as i16,
            element_resistance: p(112) as i16,
            blast_attack: p(125) as i16,
            blast_element_attack: p(126) as i16,
            strength: p(101) as i32,
            dexterity: p(102) as i32,
            constitution: p(103) as i32,
            intelligence: p(104) as i32,
            cch: p(108) as i16,
            full_miss: p(127) as i16,
            attack_avoid: p(128) as i16,
            element_avoid: p(129) as i16,
            hit: p(20_001) as i16,
            dodge: p(110) as i16,
            item_index: p(50_001),
            item_amount: p(50_002),
            frequency_ms: p(6_001),
            serialized_offset: None,
        }
    }

    /// Unserialize: внешний ID уже прочитан factory; один clock до блока.
    pub fn decode_at(payload: &[u8], offset: usize, now: &mut dyn FnMut() -> u32) -> Option<Self> {
        if read_u32(payload, offset) != Some(UNDEAD_STATE_ID) {
            return None;
        }
        let base = offset.checked_add(4)?;
        let block = payload.get(base..base.checked_add(UNDEAD_STATE_PARAMETER_BYTES)?)?;
        let now_ms = now();
        Some(Self {
            state_id: read_u32(block, 4)?,
            state_type: read_u16(block, 0)?,
            keep_time_ms: read_u32(block, 8)?,
            started_ms: now_ms,
            last_item_tick_ms: now_ms,
            disappear_after_dead: block[12],
            percentage: block[13] != 0,
            maximum_hp: read_i16(block, 14)?,
            maximum_mp: read_i16(block, 16)?,
            minimum_attack: read_i16(block, 18)?,
            maximum_attack: read_i16(block, 20)?,
            element_modify: read_i16(block, 22)?,
            defense: read_i16(block, 24)?,
            element_resistance: read_i16(block, 26)?,
            blast_attack: read_i16(block, 28)?,
            blast_element_attack: read_i16(block, 30)?,
            strength: read_i32(block, 32)?,
            dexterity: read_i32(block, 36)?,
            constitution: read_i32(block, 40)?,
            intelligence: read_i32(block, 44)?,
            cch: read_i16(block, 48)?,
            full_miss: read_i16(block, 50)?,
            attack_avoid: read_i16(block, 52)?,
            element_avoid: read_i16(block, 54)?,
            hit: read_i16(block, 56)?,
            dodge: read_i16(block, 58)?,
            item_index: read_u32(block, 60)?,
            item_amount: read_u32(block, 64)?,
            frequency_ms: read_u32(block, 68)?,
            serialized_offset: Some(offset),
        })
    }

    pub fn encoded_for_install(&self) -> [u8; Self::SERIALIZED_BYTES] {
        let mut record = [0; Self::SERIALIZED_BYTES];
        self.update_serialized_record(&mut record, 0, self.keep_time_ms);
        record
    }

    pub fn update_serialized_record(&self, payload: &mut [u8], offset: usize, remaining: u32) {
        if offset
            .checked_add(Self::SERIALIZED_BYTES)
            .is_none_or(|end| end > payload.len())
        {
            return;
        }
        let base = offset + 4;
        write_u32(payload, offset, UNDEAD_STATE_ID);
        write_u16(payload, base, self.state_type);
        write_u32(payload, base + 4, self.state_id);
        write_u32(payload, base + 8, remaining);
        payload[base + 12] = self.disappear_after_dead;
        // Native Serialize копирует исходный BOOL-байт, а не нормализует его.
        if (payload[base + 13] != 0) != self.percentage {
            payload[base + 13] = u8::from(self.percentage);
        }
        for (position, value) in [
            (14, self.maximum_hp),
            (16, self.maximum_mp),
            (18, self.minimum_attack),
            (20, self.maximum_attack),
            (22, self.element_modify),
            (24, self.defense),
            (26, self.element_resistance),
            (28, self.blast_attack),
            (30, self.blast_element_attack),
            (48, self.cch),
            (50, self.full_miss),
            (52, self.attack_avoid),
            (54, self.element_avoid),
            (56, self.hit),
            (58, self.dodge),
        ] {
            write_i16(payload, base + position, value);
        }
        for (position, value) in [
            (32, self.strength),
            (36, self.dexterity),
            (40, self.constitution),
            (44, self.intelligence),
        ] {
            write_i32(payload, base + position, value);
        }
        write_u32(payload, base + 60, self.item_index);
        write_u32(payload, base + 64, self.item_amount);
        write_u32(payload, base + 68, self.frequency_ms);
    }

    pub fn serialized_span(&self) -> Option<(usize, usize)> {
        self.serialized_offset
            .map(|offset| (offset, Self::SERIALIZED_BYTES))
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

const SKILL_USAGE_CONST: u32 = 20_010;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;

fn read_u32(payload: &[u8], offset: usize) -> Option<u32> {
    let bytes = payload.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

fn read_i32(payload: &[u8], offset: usize) -> Option<i32> {
    let bytes = payload.get(offset..offset.checked_add(4)?)?;
    Some(i32::from_le_bytes(bytes.try_into().ok()?))
}

fn read_u16(payload: &[u8], offset: usize) -> Option<u16> {
    let bytes = payload.get(offset..offset.checked_add(2)?)?;
    Some(u16::from_le_bytes(bytes.try_into().ok()?))
}

fn read_i16(payload: &[u8], offset: usize) -> Option<i16> {
    let bytes = payload.get(offset..offset.checked_add(2)?)?;
    Some(i16::from_le_bytes(bytes.try_into().ok()?))
}

fn write_u32(payload: &mut [u8], offset: usize, value: u32) {
    payload[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_i32(payload: &mut [u8], offset: usize, value: i32) {
    payload[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn write_u16(payload: &mut [u8], offset: usize, value: u16) {
    payload[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_i16(payload: &mut [u8], offset: usize, value: i16) {
    payload[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}
