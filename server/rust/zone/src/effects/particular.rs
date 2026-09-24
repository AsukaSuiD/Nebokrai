//! Данные и 8-байтная запись CParticularState (0x186A5) в Zone.
//! Источник: gameserver.exe + GameServer.pdb,
//! appserver/other states/particularstate.cpp/.h.
//! Конструктор VA 0x004F9440 принимает любой DWORD additional (vtable
//! 0x00653684); goods/duplicate-gates — ответственность вызывающего.
//! Serialize VA 0x005E23D0 и Unserialize VA 0x00601350 сохраняют
//! ID + additional без часов, включая additional=0 при загрузке.
//! AI VA 0x004F9900 после границы интервала проверяет товар у игрока;
//! checkstamp всегда нулевой и не продвигается. Живые Begin/End и visual
//! остаются у переходного Game.

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const PARTICULAR_STATE_ID: u32 = 0x186a5;
pub const PARTICULAR_STATE_BYTES: usize = 8;
pub const PARTICULAR_STATE_CHECK_INTERVAL_MS: u32 = 2_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ParticularState {
    additional_data: u32,
}

impl ParticularState {
    pub const fn new(additional_data: u32) -> Self {
        Self { additional_data }
    }

    pub const fn additional_data(&self) -> u32 {
        self.additional_data
    }

    pub fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let _state_id = reader.read_u32()?;
        Ok(Self::new(reader.read_u32()?))
    }

    pub fn encoded(&self) -> [u8; PARTICULAR_STATE_BYTES] {
        let mut record = [0; PARTICULAR_STATE_BYTES];
        record[..4].copy_from_slice(&PARTICULAR_STATE_ID.to_le_bytes());
        record[4..].copy_from_slice(&self.additional_data.to_le_bytes());
        record
    }

    pub const fn state_id(&self) -> i32 {
        PARTICULAR_STATE_ID as i32
    }

    /// Базовый GetRemainedTime этого владельца не переопределён.
    pub const fn client_state_time(&self) -> i32 {
        0
    }

    /// Checkstamp всегда нулевой: условие срабатывает при now >= 2000.
    pub const fn due(&self, now_ms: u32) -> bool {
        PARTICULAR_STATE_CHECK_INTERVAL_MS <= now_ms
    }
}
