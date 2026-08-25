//! Владелец обычных extended-state `CExState/CExStateNew` GameServer.
//!
//! Контракт подтверждён `gameserver.exe + GameServer.pdb`, исходными owner-ами
//! `other states/exstate.cpp`, `exstatenew.cpp` и caller-ом
//! `CMoveShape::Add/Del/GetExState*`. Rust enum заменяет два raw `CState*`, но
//! сохраняет state ID `0x32/0x33`, byte-layout `40/52`, wrapping DWORD clock,
//! replacement по type/level, property overlay и periodic item consumption.
//! Сырой псевдокод ниже остаётся локальным provenance реализованного owner-а.

use crate::gameserver::appserver::skills::skillfactory::CSkillFactory;

pub(crate) const EX_STATE_ID: u32 = 0x32;
pub(crate) const EX_STATE_NEW_ID: u32 = 0x33;
const EX_STATE_BYTES: usize = 40;
const EX_STATE_NEW_BYTES: usize = 52;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ExtendedStateKind {
    Original,
    New,
}

impl ExtendedStateKind {
    pub(crate) const fn state_id(self) -> u32 {
        match self {
            Self::Original => EX_STATE_ID,
            Self::New => EX_STATE_NEW_ID,
        }
    }

    const fn parameter_bytes(self) -> usize {
        match self {
            Self::Original => EX_STATE_BYTES,
            Self::New => EX_STATE_NEW_BYTES,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExtendedState {
    pub(crate) kind: ExtendedStateKind,
    pub(crate) state_type: u16,
    pub(crate) level: u32,
    pub(crate) keep_time_ms: u32,
    pub(crate) maximum_hp: u16,
    pub(crate) maximum_mp: u16,
    pub(crate) minimum_attack: u16,
    pub(crate) maximum_attack: u16,
    pub(crate) element_modify: u16,
    pub(crate) defense: u16,
    pub(crate) element_resistance: u16,
    pub(crate) cch: u16,
    pub(crate) full_miss: u16,
    pub(crate) attack_avoid: u16,
    pub(crate) element_avoid: u16,
    pub(crate) hit: u16,
    pub(crate) dodge: u16,
    pub(crate) item_index: u32,
    pub(crate) item_amount: u32,
    pub(crate) frequency_ms: u32,
    pub(crate) started_ms: u32,
    pub(crate) last_item_tick_ms: u32,
    serialized_offset: Option<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ExtendedStateMutation {
    pub(crate) removed: Vec<ExtendedState>,
    pub(crate) added: Option<ExtendedState>,
    pub(crate) legacy_return: u32,
}

impl ExtendedState {
    pub(crate) fn from_factory(
        kind: ExtendedStateKind,
        level: u32,
        factory: &CSkillFactory,
        now_ms: u32,
    ) -> Option<Self> {
        if level == 0 {
            return None;
        }
        let properties = factory.query_skill_base_properties(kind.state_id(), level as i32)?;
        let p = |usage| properties.query_property(usage);
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
            started_ms: now_ms,
            last_item_tick_ms: now_ms,
            serialized_offset: None,
        })
    }

    pub(crate) fn decode_all(payload: &[u8], now_ms: u32) -> Vec<Self> {
        if payload.len() < 4 {
            return Vec::new();
        }
        let mut states = Vec::new();
        for offset in 4..payload.len().saturating_sub(3) {
            let Some(state_id) = read_u32(payload, offset) else {
                continue;
            };
            let kind = match state_id {
                EX_STATE_ID => ExtendedStateKind::Original,
                EX_STATE_NEW_ID => ExtendedStateKind::New,
                _ => continue,
            };
            let base = offset + 4;
            if base + kind.parameter_bytes() > payload.len() {
                continue;
            }
            let Some(level) = read_u32(payload, base + 4) else {
                continue;
            };
            if level == 0 {
                continue;
            }
            states.push(Self {
                kind,
                state_type: read_u16(payload, base).unwrap_or_default(),
                level,
                keep_time_ms: read_u32(payload, base + 8).unwrap_or_default(),
                maximum_hp: read_u16(payload, base + 12).unwrap_or_default(),
                maximum_mp: read_u16(payload, base + 14).unwrap_or_default(),
                minimum_attack: read_u16(payload, base + 16).unwrap_or_default(),
                maximum_attack: read_u16(payload, base + 18).unwrap_or_default(),
                element_modify: read_u16(payload, base + 20).unwrap_or_default(),
                defense: read_u16(payload, base + 22).unwrap_or_default(),
                element_resistance: read_u16(payload, base + 24).unwrap_or_default(),
                cch: read_u16(payload, base + 26).unwrap_or_default(),
                full_miss: read_u16(payload, base + 28).unwrap_or_default(),
                attack_avoid: read_u16(payload, base + 30).unwrap_or_default(),
                element_avoid: read_u16(payload, base + 32).unwrap_or_default(),
                hit: read_u16(payload, base + 34).unwrap_or_default(),
                dodge: read_u16(payload, base + 36).unwrap_or_default(),
                item_index: (kind == ExtendedStateKind::New)
                    .then(|| read_u32(payload, base + 40).unwrap_or_default())
                    .unwrap_or(0),
                item_amount: (kind == ExtendedStateKind::New)
                    .then(|| read_u32(payload, base + 44).unwrap_or_default())
                    .unwrap_or(0),
                frequency_ms: (kind == ExtendedStateKind::New)
                    .then(|| read_u32(payload, base + 48).unwrap_or_default())
                    .unwrap_or(0),
                started_ms: now_ms,
                last_item_tick_ms: now_ms,
                serialized_offset: Some(offset),
            });
        }
        states
    }

    pub(crate) const fn state_id(&self) -> u32 {
        self.kind.state_id()
    }

    pub(crate) fn serialized_span(&self) -> Option<(usize, usize)> {
        self.serialized_offset
            .map(|offset| (offset, 4 + self.kind.parameter_bytes()))
    }

    pub(crate) fn expired(&self, now_ms: u32) -> bool {
        self.keep_time_ms != 0 && self.keep_time_ms < now_ms.wrapping_sub(self.started_ms)
    }

    pub(crate) fn activate_loaded(&mut self, now_ms: u32) {
        self.started_ms = now_ms;
        self.last_item_tick_ms = now_ms;
    }

    pub(crate) fn remaining_time_ms(&self, now_ms: u32) -> u32 {
        if self.keep_time_ms == 0 {
            return 0;
        }
        self.keep_time_ms
            .saturating_sub(now_ms.wrapping_sub(self.started_ms))
    }

    pub(crate) fn item_due(&self, now_ms: u32) -> bool {
        self.kind == ExtendedStateKind::New
            && self.frequency_ms != 0
            && self.item_index != 0
            && self.item_amount != 0
            && self.frequency_ms < now_ms.wrapping_sub(self.last_item_tick_ms)
    }

    pub(crate) fn restart_item_clock(&mut self, now_ms: u32) {
        self.last_item_tick_ms = now_ms;
    }

    pub(crate) fn write_serialized(&mut self, payload: &mut [u8], offset: usize) {
        let base = offset + 4;
        write_u32(payload, offset, self.state_id());
        write_u16(payload, base, self.state_type);
        write_u32(payload, base + 4, self.level);
        write_u32(payload, base + 8, self.keep_time_ms);
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
            write_u16(payload, base + position, value);
        }
        if self.kind == ExtendedStateKind::New {
            write_u32(payload, base + 40, self.item_index);
            write_u32(payload, base + 44, self.item_amount);
            write_u32(payload, base + 48, self.frequency_ms);
        }
        self.serialized_offset = Some(offset);
    }

    pub(crate) fn update_serialized_runtime(&self, payload: &mut [u8], now_ms: u32) {
        let Some(offset) = self.serialized_offset else {
            return;
        };
        if offset + 4 + self.kind.parameter_bytes() <= payload.len() {
            write_u32(payload, offset + 12, self.remaining_time_ms(now_ms));
        }
    }

    pub(crate) fn remove_serialized(&self, payload: &mut Vec<u8>) {
        let Some(offset) = self.serialized_offset else {
            return;
        };
        let end = offset + 4 + self.kind.parameter_bytes();
        if end > payload.len() {
            return;
        }
        payload.drain(offset..end);
        if let Some(count) = read_u32(payload, 0) {
            write_u32(payload, 0, count.saturating_sub(1));
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
    Some(u16::from_le_bytes(
        source.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn read_u32(source: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        source.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn write_u16(destination: &mut [u8], offset: usize, value: u16) {
    destination[offset..offset + 2].copy_from_slice(&value.to_le_bytes());
}

fn write_u32(destination: &mut [u8], offset: usize, value: u32) {
    destination[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp

// ============================================================================
// FUNCTION: CExState::CExState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:15
// RVA: 0x001D9230
// ADDRESS: 005d9230
// PROTOTYPE: undefined __thiscall CExState(tagExState * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExState::CExState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:24
// RVA: 0x001D92B0
// ADDRESS: 005d92b0
// PROTOTYPE: undefined __thiscall CExState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExState::~CExState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:33
// RVA: 0x001D9340
// ADDRESS: 005d9340
// PROTOTYPE: void __thiscall ~CExState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:113
// RVA: 0x001D9350
// ADDRESS: 005d9350
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:131
// RVA: 0x001D9410
// ADDRESS: 005d9410
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExState::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:168
// RVA: 0x001D94E0
// ADDRESS: 005d94e0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:202
// RVA: 0x001D9510
// ADDRESS: 005d9510
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:215
// RVA: 0x001D9550
// ADDRESS: 005d9550
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExState::OnUpdateProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:46
// RVA: 0x001D95A0
// ADDRESS: 005d95a0
// PROTOTYPE: int __thiscall OnUpdateProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:98
// RVA: 0x001D9780
// ADDRESS: 005d9780
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CExStateVisualEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\other states\exstate.cpp:227
// RVA: 0x001D9830
// ADDRESS: 005d9830
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
