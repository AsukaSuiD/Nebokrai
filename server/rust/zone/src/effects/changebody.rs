//! Данные и 124-байтная запись `CHBYState` (ID `0x37`) в Zone.
//! Источник: gameserver.exe + GameServer.pdb,
//! appserver/other states/chbystate.cpp/.h.
//! Конструктор VA `0x005DAC90` записывает ID без чтения часов и оставляет
//! накопленный список навыков пустым. Vtable `0x0065E41C`: AI
//! `0x005DAAA0` (при нулевом сроке часов не читает, иначе один clock и
//! строгое unsigned-сравнение), End `0x005DA240`, OnUpdateProperties
//! `0x005DA0F0`, GetRemainedTime `0x005DA030`, Serialize `0x005DA080`,
//! Unserialize `0x005DA0C0` (clock до 120-байтного блока, затем
//! `online = true`). Живые Begin/restart/End, visual, навыки и hotkeys
//! остаются у переходного Game.

use super::time::change_body_client_state_time;

pub const CHANGE_BODY_STATE_ID: u32 = 0x37;
pub const CHANGE_BODY_SKILL_TYPE: u32 = 55;
pub const CHANGE_BODY_PARAMETER_BYTES: usize = 120;
pub const CHANGE_BODY_STATE_BYTES: usize = 4 + CHANGE_BODY_PARAMETER_BYTES;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChangeBodyState {
    pub has_changed_region: bool,
    pub visual_effect: u16,
    pub level: u32,
    pub keep_time_ms: u32,
    pub mode: u32,
    pub change_region: bool,
    pub restore_online: bool,
    pub continue_after_death: bool,
    pub online: bool,
    pub maximum_hp: u32,
    pub maximum_mp: u32,
    pub minimum_attack: u32,
    pub maximum_attack: u32,
    pub defense: u32,
    pub element_resistance: u32,
    pub cch: u16,
    pub blast_attack: u16,
    pub blast_element_attack: u16,
    pub skills: [(u16, u16); 5],
    pub begun_skill_ids: Vec<u16>,
    pub old_hotkeys: [u32; 12],
    pub started_ms: u32,
    serialized_offset: Option<usize>,
}

impl ChangeBodyState {
    /// Factory snapshot; порядок запросов свойств сохранён. Нулевой уровень
    /// не создаёт состояния; отсутствие записи навыка решает вызывающий.
    pub fn from_properties(
        level: u32,
        mut query_property: impl FnMut(u32) -> Option<u32>,
    ) -> Option<Self> {
        if level == 0 {
            return None;
        }
        let get = &mut query_property;
        let skills = [
            (get(60_011)? as u16, get(60_012)? as u16),
            (get(60_021)? as u16, get(60_022)? as u16),
            (get(60_031)? as u16, get(60_032)? as u16),
            (get(60_041)? as u16, get(60_042)? as u16),
            (get(60_051)? as u16, get(60_052)? as u16),
        ];
        Some(Self {
            has_changed_region: true,
            visual_effect: get(60_001)? as u16,
            level,
            keep_time_ms: get(10_002)?,
            mode: get(60_000)?,
            change_region: get(60_002)? != 0,
            restore_online: get(60_003)? != 0,
            continue_after_death: get(60_004)? != 0,
            online: false,
            maximum_hp: get(118)?,
            maximum_mp: get(119)?,
            minimum_attack: get(116)?,
            maximum_attack: get(117)?,
            defense: get(109)?,
            element_resistance: get(112)?,
            cch: get(108)? as u16,
            blast_attack: get(125)? as u16,
            blast_element_attack: get(126)? as u16,
            skills,
            begun_skill_ids: Vec::new(),
            old_hotkeys: [0; 12],
            started_ms: 0,
            serialized_offset: None,
        })
    }

    /// Unserialize: внешний ID уже прочитан factory; один clock до блока.
    pub fn decode_at(payload: &[u8], offset: usize, now: &mut dyn FnMut() -> u32) -> Option<Self> {
        offset
            .checked_add(CHANGE_BODY_STATE_BYTES)
            .filter(|end| *end <= payload.len())?;
        if read_u32(payload, offset) != Some(CHANGE_BODY_STATE_ID) {
            return None;
        }
        let started_ms = now();
        let base = offset + 4;
        let level = read_u32(payload, base + 4)?;
        if level == 0
            || payload[base] > 1
            || payload[base + 16] > 1
            || payload[base + 17] > 1
            || payload[base + 18] > 1
            || payload[base + 19] > 1
        {
            return None;
        }
        let mut skills = [(0, 0); 5];
        for (index, skill) in skills.iter_mut().enumerate() {
            *skill = (
                read_u16(payload, base + 50 + index * 4).unwrap_or_default(),
                read_u16(payload, base + 52 + index * 4).unwrap_or_default(),
            );
        }
        let mut old_hotkeys = [0; 12];
        for (index, hotkey) in old_hotkeys.iter_mut().enumerate() {
            *hotkey = read_u32(payload, base + 72 + index * 4).unwrap_or_default();
        }
        Some(Self {
            has_changed_region: payload[base] != 0,
            visual_effect: read_u16(payload, base + 2).unwrap_or_default(),
            level,
            keep_time_ms: read_u32(payload, base + 8).unwrap_or_default(),
            mode: read_u32(payload, base + 12).unwrap_or_default(),
            change_region: payload[base + 16] != 0,
            restore_online: payload[base + 17] != 0,
            continue_after_death: payload[base + 18] != 0,
            online: true,
            maximum_hp: read_u32(payload, base + 20).unwrap_or_default(),
            maximum_mp: read_u32(payload, base + 24).unwrap_or_default(),
            minimum_attack: read_u32(payload, base + 28).unwrap_or_default(),
            maximum_attack: read_u32(payload, base + 32).unwrap_or_default(),
            defense: read_u32(payload, base + 36).unwrap_or_default(),
            element_resistance: read_u32(payload, base + 40).unwrap_or_default(),
            cch: read_u16(payload, base + 44).unwrap_or_default(),
            blast_attack: read_u16(payload, base + 46).unwrap_or_default(),
            blast_element_attack: read_u16(payload, base + 48).unwrap_or_default(),
            skills,
            begun_skill_ids: Vec::new(),
            old_hotkeys,
            started_ms,
            serialized_offset: Some(offset),
        })
    }

    pub fn serialized_span(&self) -> Option<(usize, usize)> {
        self.serialized_offset
            .map(|offset| (offset, CHANGE_BODY_STATE_BYTES))
    }

    pub fn encoded_for_install(&self) -> [u8; CHANGE_BODY_STATE_BYTES] {
        let mut record = [0; CHANGE_BODY_STATE_BYTES];
        self.update_serialized_record(&mut record, 0, self.keep_time_ms);
        record
    }

    /// Writer по общему span обновляет все поля, включая mode/old_hotkeys,
    /// сохраняя padding загруженных записей по смещениям +1 и +70..71.
    pub fn update_serialized_record(&self, payload: &mut [u8], offset: usize, remaining: u32) {
        if offset
            .checked_add(CHANGE_BODY_STATE_BYTES)
            .is_none_or(|end| end > payload.len())
        {
            return;
        }
        let base = offset + 4;
        write_u32(payload, offset, CHANGE_BODY_STATE_ID);
        payload[base] = u8::from(self.has_changed_region);
        write_u16(payload, base + 2, self.visual_effect);
        write_u32(payload, base + 4, self.level);
        write_u32(payload, base + 8, remaining);
        write_u32(payload, base + 12, self.mode);
        payload[base + 16] = u8::from(self.change_region);
        payload[base + 17] = u8::from(self.restore_online);
        payload[base + 18] = u8::from(self.continue_after_death);
        payload[base + 19] = u8::from(self.online);
        write_u32(payload, base + 20, self.maximum_hp);
        write_u32(payload, base + 24, self.maximum_mp);
        write_u32(payload, base + 28, self.minimum_attack);
        write_u32(payload, base + 32, self.maximum_attack);
        write_u32(payload, base + 36, self.defense);
        write_u32(payload, base + 40, self.element_resistance);
        write_u16(payload, base + 44, self.cch);
        write_u16(payload, base + 46, self.blast_attack);
        write_u16(payload, base + 48, self.blast_element_attack);
        for (index, (skill_id, level)) in self.skills.iter().copied().enumerate() {
            write_u16(payload, base + 50 + index * 4, skill_id);
            write_u16(payload, base + 52 + index * 4, level);
        }
        for (index, hotkey) in self.old_hotkeys.iter().copied().enumerate() {
            write_u32(payload, base + 72 + index * 4, hotkey);
        }
    }

    pub fn remaining_time_ms(&self, now_ms: u32) -> u32 {
        self.client_state_time(|| now_ms)
    }

    /// Getter 0x005DA030: после истечения ненулевого срока возвращает 1.
    pub fn client_state_time(&self, now: impl FnMut() -> u32) -> u32 {
        change_body_client_state_time(self.started_ms, self.keep_time_ms, now)
    }

    /// AI: нулевой keep не читает часы и не истекает.
    pub fn expired(&self, now_ms: u32) -> bool {
        self.keep_time_ms != 0 && self.started_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    /// Serialize записывает остаток в живой keep без перезапуска clock.
    pub fn commit_serialized_time(&mut self, remaining: u32) {
        self.keep_time_ms = remaining;
    }

    pub fn on_change_region(&mut self) -> bool {
        if !self.has_changed_region {
            self.has_changed_region = true;
            return false;
        }
        !self.change_region
    }

    pub fn on_player_lost(&mut self) -> bool {
        self.has_changed_region = false;
        self.online = true;
        !self.restore_online
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
