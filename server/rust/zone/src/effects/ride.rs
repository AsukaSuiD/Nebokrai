//! Данные и wire-кодек `CRideState` (ID `100004`) в Zone.
//! Источник: gameserver.exe + GameServer.pdb,
//! appserver/other states/ridestate.h/.cpp.
//! Serialize VA `0x004F8F60` пишет четыре DWORD (ID, type, level,
//! roleLimit) и C-string имени без часов и мутации. Safe Unserialize
//! VA `0x004F93B0` читает три DWORD и требует NUL в пределах стекового
//! буфера: запись, которая переполнила бы стек C++, отклоняется, а не
//! обрезается. AI VA `0x004F9110` не обновляет timestamp проверки товара,
//! поэтому после первого gate проверка идёт каждый последующий проход.
//! Живые Begin/End, visual и разрешение участников остаются у переходного
//! Game.

use nebokrai_shared::protocol::{LegacyReader, LegacyWriter};

pub const RIDE_STATE_ID: u32 = 100_004;
const RIDE_STATE_FIXED_BYTES: usize = 16;
const RIDE_GOODS_NAME_CAPACITY: usize = 256;
pub const RIDE_GOODS_CHECK_INTERVAL_MS: u32 = 10_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RideState {
    mount_type: u32,
    level: u32,
    role_limit: u32,
    goods_name: Vec<u8>,
    check_goods_timestamp_ms: u32,
    serialized_offset: Option<usize>,
}

impl RideState {
    pub fn new(mount_type: u32, level: u32, role_limit: u32, goods_name: &[u8]) -> Self {
        Self {
            mount_type,
            level,
            role_limit,
            goods_name: goods_name.to_vec(),
            check_goods_timestamp_ms: 0,
            serialized_offset: None,
        }
    }

    pub const fn mount_type(&self) -> u32 {
        self.mount_type
    }

    pub const fn level(&self) -> u32 {
        self.level
    }

    pub const fn role_limit(&self) -> u32 {
        self.role_limit
    }

    pub fn goods_name(&self) -> &[u8] {
        &self.goods_name
    }

    /// Хвост объектного Begin `0x004F8D60`, после визуала и fight-lock.
    pub const fn reset_goods_check(&mut self) {
        self.check_goods_timestamp_ms = 0;
    }

    pub const fn additional_data(&self) -> u32 {
        self.mount_type.wrapping_shl(16) | self.level
    }

    /// Базовый GetRemainedTime этого владельца не переопределён.
    pub const fn client_state_time(&self) -> i32 {
        0
    }

    /// Exact `timestamp + 10000 <= timeGetTime`; timestamp намеренно не
    /// обновляется после успешного gate.
    pub const fn goods_check_due(&self, now_ms: u32) -> bool {
        self.check_goods_timestamp_ms
            .wrapping_add(RIDE_GOODS_CHECK_INTERVAL_MS)
            <= now_ms
    }

    pub fn decode_at(payload: &[u8], offset: usize) -> Option<Self> {
        if read_u32(payload, offset)? != RIDE_STATE_ID {
            return None;
        }
        let name_start = offset
            .checked_add(RIDE_STATE_FIXED_BYTES)
            .filter(|start| *start <= payload.len())?;
        let available = payload.len().saturating_sub(name_start);
        let name_length = payload[name_start..]
            .iter()
            .take(RIDE_GOODS_NAME_CAPACITY)
            .position(|byte| *byte == 0)?;
        if name_length >= available {
            return None;
        }
        Some(Self {
            mount_type: read_u32(payload, offset + 4)?,
            level: read_u32(payload, offset + 8)?,
            role_limit: read_u32(payload, offset + 12)?,
            goods_name: payload[name_start..name_start + name_length].to_vec(),
            check_goods_timestamp_ms: 0,
            serialized_offset: Some(offset),
        })
    }

    pub fn encoded_for_install(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(self.serialized_size());
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(RIDE_STATE_ID);
        writer.write_u32(self.mount_type);
        writer.write_u32(self.level);
        writer.write_u32(self.role_limit);
        writer.write_c_string(&self.goods_name);
        bytes
    }

    pub fn serialized_size(&self) -> usize {
        let name_length = self
            .goods_name
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(self.goods_name.len());
        RIDE_STATE_FIXED_BYTES + name_length + 1
    }

    pub fn serialized_span(&self) -> Option<(usize, usize)> {
        self.serialized_offset
            .map(|offset| (offset, self.serialized_size()))
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

fn read_u32(payload: &[u8], offset: usize) -> Option<u32> {
    LegacyReader::at(payload, offset).ok()?.read_u32().ok()
}
