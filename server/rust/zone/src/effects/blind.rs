//! Данные и 8-байтная запись семейства CBlindState в Zone.
//! Источник: gameserver.exe + GameServer.pdb,
//! appserver/skills/{blindstate,knightcutstate,knockoutstate,boalockstate,...}.cpp/.h.
//! Конструкторы записывают свой ID и срок без чтения часов: KnightCut
//! VA 0x005FCCF0/0x005FCD60 (vtable 0x00661254, ID 0x67), KnockOut
//! VA 0x005F4F30/0x005F4FA0 (vtable 0x00660894, ID 0x192), BoaLock
//! VA 0x005FB560/0x005FB5D0 (vtable 0x006610DC, ID 0xD2), Blind
//! VA 0x00607380 (vtable 0x00662214, ID 0x76), Rush VA 0x006077E0
//! (vtable 0x00662274, ID 0x73), Rush2 VA 0x005F12E0/0x005F1350
//! (vtable 0x0066041C, ID 0x7C), Seal VA 0x005FF800/0x005FF870
//! (vtable 0x006615F4, ID 0x138), Strike VA 0x00606830
//! (vtable 0x00662154, ID 0xDD).
//! Все восемь vtable разделяют Serialize VA 0x005F51E0 (ID, затем остаток
//! через getter), Unserialize VA 0x005EAAC0 (часы после внешнего ID и до
//! сохранённого остатка), GetRemainedTime VA 0x005F2CD0 и AI
//! VA 0x005D5BA0 (завершение только при `now > start + keep`).

use super::time::timed_client_state_time;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};

pub const BLIND_STATE_ID: u32 = 0x76;
pub const BLIND_STATE_BYTES: usize = 8;
pub const KNIGHT_CUT_STATE_ID: u32 = 0x67;
pub const KNIGHT_CUT_STATE_BYTES: usize = BLIND_STATE_BYTES;
pub const KNOCK_OUT_STATE_ID: u32 = 0x192;
pub const KNOCK_OUT_STATE_BYTES: usize = BLIND_STATE_BYTES;
pub const BOA_LOCK_STATE_ID: u32 = 0xd2;
pub const BOA_LOCK_STATE_BYTES: usize = BLIND_STATE_BYTES;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BlindState<const ID: u32 = BLIND_STATE_ID, const BLOCKS_FIGHTING: bool = true> {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl<const ID: u32, const BLOCKS_FIGHTING: bool> BlindState<ID, BLOCKS_FIGHTING> {
    /// Duration-конструктор оставляет нулевое начало отсчёта.
    pub const fn new(keep_time_ms: u32) -> Self {
        Self {
            started_at_ms: 0,
            keep_time_ms,
        }
    }

    pub fn begin_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub fn decode(
        payload: &[u8],
        offset: usize,
        now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        // Общий Unserialize читает часы после внешнего ID и до остатка срока.
        let started_at_ms = now();
        let keep_time_ms = reader.read_u32()?;
        Ok(Self {
            started_at_ms,
            keep_time_ms,
        })
    }

    pub const fn skill_id(&self) -> u32 {
        ID
    }

    /// Строгий wrapping-deadline действует и при нулевом сроке.
    pub const fn expired(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub fn client_state_time(&self, now: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now)
    }

    pub fn client_time(&self, now: impl FnMut() -> u32) -> i32 {
        self.client_state_time(now) as i32
    }

    pub fn encoded(&self, now: impl FnMut() -> u32) -> [u8; BLIND_STATE_BYTES] {
        self.encode_record(|| self.client_state_time(now))
    }

    pub fn encoded_for_install(&self) -> [u8; BLIND_STATE_BYTES] {
        self.encode_record(|| self.keep_time_ms)
    }

    fn encode_record(&self, remaining: impl FnOnce() -> u32) -> [u8; BLIND_STATE_BYTES] {
        let mut record = [0; BLIND_STATE_BYTES];
        record[..4].copy_from_slice(&ID.to_le_bytes());
        // Serialize записывает ID до вызова GetRemainedTime.
        record[4..].copy_from_slice(&remaining().to_le_bytes());
        record
    }
}

pub type KnightCutState = BlindState<KNIGHT_CUT_STATE_ID>;
pub type KnockOutState = BlindState<KNOCK_OUT_STATE_ID>;
/// Единственное отличие BoaLock в данных семейства: запрет боя не ставится
/// (его vtable имеет собственный End VA 0x005FB800 и пустой OnAction).
pub type BoaLockState = BlindState<BOA_LOCK_STATE_ID, false>;
