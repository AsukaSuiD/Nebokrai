//! Владелец состояния преображения `CHBYState` GameServer.
//!
//! Контракт подтверждён точной парой `gameserver.exe + GameServer.pdb` и
//! исходным owner-ом `appserver/other states/chbystate.cpp`. Реализация хранит
//! byte-exact 120-байтовый `tagCHBYState`, время, пять временных навыков и
//! сохранённые hotkey 12..23. Системный wrapping tick передаётся caller-ом;
//! Rust-владение заменяет raw `CState*`. Сохранённый ниже псевдокод служит
//! локальным provenance для реализованного owner-а и не входит в runtime.
//! Доступ к little-endian полям делегирован общему legacy codec поверх `bytes`.
//! `Serialize` записывает остаток обратно в keeptime без перезапуска clock;
//! клиентский снимок этого не делает. `AI` (0x005daaa0) завершает состояние
//! только после абсолютного wrapping deadline, а не на его границе.

use crate::gameserver::appserver::skills::skillfactory::CSkillFactory;
use crate::gameserver::appserver::legacycodec::{LegacyReader, LegacyWriter};

pub(crate) const CHANGE_BODY_STATE_ID: u32 = 0x37;
pub(crate) const CHANGE_BODY_SKILL_TYPE: u32 = 55;
const CHANGE_BODY_PARAMETER_BYTES: usize = 120;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ChangeBodyState {
    pub(crate) has_changed_region: bool,
    pub(crate) visual_effect: u16,
    pub(crate) level: u32,
    pub(crate) keep_time_ms: u32,
    pub(crate) mode: u32,
    pub(crate) change_region: bool,
    pub(crate) restore_online: bool,
    pub(crate) continue_after_death: bool,
    pub(crate) online: bool,
    pub(crate) maximum_hp: u32,
    pub(crate) maximum_mp: u32,
    pub(crate) minimum_attack: u32,
    pub(crate) maximum_attack: u32,
    pub(crate) defense: u32,
    pub(crate) element_resistance: u32,
    pub(crate) cch: u16,
    pub(crate) blast_attack: u16,
    pub(crate) blast_element_attack: u16,
    pub(crate) skills: [(u16, u16); 5],
    pub(crate) old_hotkeys: [u32; 12],
    pub(crate) started_ms: u32,
    serialized_offset: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ChangeBodyMutation {
    pub(crate) removed: Option<ChangeBodyState>,
    pub(crate) added: Option<ChangeBodyState>,
    pub(crate) legacy_return: u32,
}

impl ChangeBodyState {
    pub(crate) fn from_factory(
        level: u32,
        factory: &CSkillFactory,
        started_ms: u32,
    ) -> Option<Self> {
        let properties =
            factory.query_skill_base_properties(CHANGE_BODY_SKILL_TYPE, level as i32)?;
        let get = |usage| Some(properties.query_property(usage));
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
            skills: [
                (get(60_011)? as u16, get(60_012)? as u16),
                (get(60_021)? as u16, get(60_022)? as u16),
                (get(60_031)? as u16, get(60_032)? as u16),
                (get(60_041)? as u16, get(60_042)? as u16),
                (get(60_051)? as u16, get(60_052)? as u16),
            ],
            old_hotkeys: [0; 12],
            started_ms,
            serialized_offset: None,
        })
    }

    pub(crate) fn decode_all(payload: &[u8], started_ms: u32) -> Vec<Self> {
        if payload.len() < 4 {
            return Vec::new();
        }
        let mut states = Vec::new();
        for offset in 4..=payload
            .len()
            .saturating_sub(4 + CHANGE_BODY_PARAMETER_BYTES)
        {
            if read_u32(payload, offset) != Some(CHANGE_BODY_STATE_ID) {
                continue;
            }
            let base = offset + 4;
            let Some(level) = read_u32(payload, base + 4) else {
                continue;
            };
            if level == 0
                || payload[base] > 1
                || payload[base + 16] > 1
                || payload[base + 17] > 1
                || payload[base + 18] > 1
                || payload[base + 19] > 1
            {
                continue;
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
            states.push(Self {
                has_changed_region: payload[base] != 0,
                visual_effect: read_u16(payload, base + 2).unwrap_or_default(),
                level,
                keep_time_ms: read_u32(payload, base + 8).unwrap_or_default(),
                mode: read_u32(payload, base + 12).unwrap_or_default(),
                change_region: payload[base + 16] != 0,
                restore_online: payload[base + 17] != 0,
                continue_after_death: payload[base + 18] != 0,
                online: payload[base + 19] != 0,
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
                old_hotkeys,
                started_ms,
                serialized_offset: Some(offset),
            });
        }
        states
    }

    pub(crate) fn remove_serialized(&self, payload: &mut Vec<u8>) {
        let Some(offset) = self.serialized_offset else {
            return;
        };
        if offset + 4 + CHANGE_BODY_PARAMETER_BYTES > payload.len() {
            return;
        }
        payload.drain(offset..offset + 4 + CHANGE_BODY_PARAMETER_BYTES);
        if let Some(count) = read_u32(payload, 0) {
            write_u32(payload, 0, count.saturating_sub(1));
        }
    }

    pub(crate) fn serialized_span(&self) -> Option<(usize, usize)> {
        self.serialized_offset
            .map(|offset| (offset, 4 + CHANGE_BODY_PARAMETER_BYTES))
    }

    pub(crate) fn write_serialized(&mut self, payload: &mut [u8], offset: usize) {
        let base = offset + 4;
        write_u32(payload, offset, CHANGE_BODY_STATE_ID);
        payload[base] = u8::from(self.has_changed_region);
        write_u16(payload, base + 2, self.visual_effect);
        write_u32(payload, base + 4, self.level);
        write_u32(payload, base + 8, self.keep_time_ms);
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
        self.serialized_offset = Some(offset);
    }

    pub(crate) fn activate_loaded(&mut self, now_ms: u32) {
        self.started_ms = now_ms;
        self.online = true;
    }

    pub(crate) fn remaining_time_ms(&self, now_ms: u32) -> u32 {
        let deadline = self.started_ms.wrapping_add(self.keep_time_ms);
        if self.keep_time_ms != 0 && deadline <= now_ms {
            1
        } else if deadline <= now_ms {
            0
        } else {
            deadline.wrapping_sub(now_ms)
        }
    }

    pub(crate) fn expired(&self, now_ms: u32) -> bool {
        self.keep_time_ms != 0 && self.started_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn commit_saved_time(&mut self, now_ms: u32) {
        self.keep_time_ms = self.remaining_time_ms(now_ms);
    }

    pub(crate) fn on_change_region(&mut self) -> bool {
        if !self.has_changed_region {
            self.has_changed_region = true;
            return false;
        }
        !self.change_region
    }

    pub(crate) fn on_player_lost(&mut self) -> bool {
        self.has_changed_region = false;
        self.online = true;
        !self.restore_online
    }

    pub(crate) fn update_serialized_runtime(&self, payload: &mut [u8], now_ms: u32) {
        let Some(offset) = self.serialized_offset else {
            return;
        };
        let base = offset + 4;
        if base + CHANGE_BODY_PARAMETER_BYTES > payload.len() {
            return;
        }
        payload[base] = u8::from(self.has_changed_region);
        write_u32(payload, base + 8, self.remaining_time_ms(now_ms));
        payload[base + 19] = u8::from(self.online);
    }

    pub(crate) fn shift_serialized_offset_for_insert(&mut self, inserted_offset: usize, amount: usize) {
        if let Some(offset) = &mut self.serialized_offset {
            if *offset >= inserted_offset {
                *offset += amount;
            }
        }
    }

    pub(crate) fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) {
        if self
            .serialized_offset
            .is_some_and(|offset| removed_offset < offset)
        {
            self.serialized_offset = self.serialized_offset.map(|offset| offset - amount);
        }
    }
}

fn read_u16(source: &[u8], offset: usize) -> Option<u16> {
    LegacyReader::at(source, offset).ok()?.read_u16().ok()
}

fn read_u32(source: &[u8], offset: usize) -> Option<u32> {
    LegacyReader::at(source, offset).ok()?.read_u32().ok()
}

fn write_u16(destination: &mut [u8], offset: usize, value: u16) {
    LegacyWriter::write_u16_at(destination, offset, value).expect("проверенное поле CHBYState");
}

fn write_u32(destination: &mut [u8], offset: usize, value: u32) {
    LegacyWriter::write_u32_at(destination, offset, value).expect("проверенное поле CHBYState");
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp

// ============================================================================
// FUNCTION: CHBYState::GetRemainedTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:61
// RVA: 0x001DA030
// ADDRESS: 005da030
// PROTOTYPE: ulong __thiscall GetRemainedTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:454
// RVA: 0x001DA080
// ADDRESS: 005da080
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:467
// RVA: 0x001DA0C0
// ADDRESS: 005da0c0
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::OnUpdateProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:70
// RVA: 0x001DA0F0
// ADDRESS: 005da0f0
// PROTOTYPE: int __thiscall OnUpdateProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::End
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:370
// RVA: 0x001DA240
// ADDRESS: 005da240
// PROTOTYPE: void __thiscall End(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCHBYStateVisualEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:482
// RVA: 0x001DA390
// ADDRESS: 005da390
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::~CHBYState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:57
// RVA: 0x001DAA30
// ADDRESS: 005daa30
// PROTOTYPE: void __thiscall ~CHBYState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:412
// RVA: 0x001DAAA0
// ADDRESS: 005daaa0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::OnChangeRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:425
// RVA: 0x001DAB80
// ADDRESS: 005dab80
// PROTOTYPE: void __thiscall OnChangeRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::CHBYState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:37
// RVA: 0x001DAC90
// ADDRESS: 005dac90
// PROTOTYPE: undefined __thiscall CHBYState(tagCHBYState * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::CHBYState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:47
// RVA: 0x001DAD20
// ADDRESS: 005dad20
// PROTOTYPE: undefined __thiscall CHBYState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:110
// RVA: 0x001DADB0
// ADDRESS: 005dadb0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2, int param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:175
// RVA: 0x001DB000
// ADDRESS: 005db000
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:239
// RVA: 0x001DB280
// ADDRESS: 005db280
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CHBYState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\chbystate.cpp:305
// RVA: 0x001DB560
// ADDRESS: 005db560
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
