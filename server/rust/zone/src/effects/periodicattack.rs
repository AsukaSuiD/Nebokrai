//! Данные, часы и запись периодического урона Zone.
//!
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/poisonarrowstate.cpp` и `kerosenestate.cpp`
//! (соответствующие `.h` задают исходные классы).
//! Живой владелец, рассылка и применение атаки остаются у Game-адаптера.
//! Опорные адреса:
//! docs/reconstruction/gameserver-skills.md#effects-wire-опорные-адреса-состояний-zone

use crate::combat::MasterInfo;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};

use super::timed_client_state_time;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeriodicAttackCore {
    master: MasterInfo,
    started_at_ms: u32,
    keep_time_ms: u32,
    frequency_ms: u32,
    attack_count: u32,
}

impl PeriodicAttackCore {
    pub const fn new(master: MasterInfo, keep_time_ms: u32, frequency_ms: u32) -> Self {
        Self {
            master,
            started_at_ms: 0,
            keep_time_ms,
            frequency_ms,
            attack_count: 0,
        }
    }

    pub const fn master(&self) -> MasterInfo {
        self.master
    }

    pub fn client_state_time(&self, now: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now)
    }

    pub fn expired(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub fn start_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub fn reset_attack_count(&mut self) {
        self.attack_count = 0;
    }

    pub const fn attack_count(&self) -> u32 {
        self.attack_count
    }

    /// Возвращает источник лишь после увеличения счётчика удара.
    pub fn prepare_attack(&mut self, count: u32, now_ms: u32) -> Option<MasterInfo> {
        let deadline = self
            .started_at_ms
            .wrapping_add(self.frequency_ms.wrapping_mul(count));
        if deadline >= now_ms {
            return None;
        }
        self.attack_count = count.wrapping_add(1);
        Some(self.master)
    }

    pub fn decode<'a>(
        payload: &'a [u8],
        offset: usize,
        id: u32,
        now: &mut dyn FnMut() -> u32,
    ) -> Result<(Self, LegacyReader<'a>), LegacyReadBlock> {
        Self::decode_prefix(payload, offset, id, Some(now))
    }

    /// Kerosene читает часы после хвоста, поэтому вызывающий завершает загрузку.
    pub fn decode_deferred_start<'a>(
        payload: &'a [u8],
        offset: usize,
        id: u32,
    ) -> Result<(Self, LegacyReader<'a>), LegacyReadBlock> {
        Self::decode_prefix(payload, offset, id, None)
    }

    pub fn finish_loading_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    fn decode_prefix<'a>(
        payload: &'a [u8],
        offset: usize,
        id: u32,
        now: Option<&mut dyn FnMut() -> u32>,
    ) -> Result<(Self, LegacyReader<'a>), LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != id {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        let master = MasterInfo {
            master_type: reader.read_i32()?,
            master_id: reader.read_i32()?,
            master_guild_id: reader.read_i32()?,
            master_team_id: reader.read_i32()?,
            master_union_id: reader.read_i32()?,
            master_country_id: reader.read_i32()?,
            permitted_to_kill_player: reader.read_i32()?,
            permitted_to_kill_teammate: reader.read_i32()?,
            permitted_to_kill_guild_member: reader.read_i32()?,
            permitted_to_kill_criminal: reader.read_i32()?,
        };
        let started_at_ms = match now {
            Some(now) => now(),
            None => 0,
        };
        let core = Self {
            master,
            started_at_ms,
            keep_time_ms: reader.read_u32()?,
            frequency_ms: reader.read_u32()?,
            attack_count: 0,
        };
        Ok((core, reader))
    }

    fn encode(&self, writer: &mut LegacyWriter<'_>, remaining: u32) {
        for value in [
            self.master.master_type,
            self.master.master_id,
            self.master.master_guild_id,
            self.master.master_team_id,
            self.master.master_union_id,
            self.master.master_country_id,
            self.master.permitted_to_kill_player,
            self.master.permitted_to_kill_teammate,
            self.master.permitted_to_kill_guild_member,
            self.master.permitted_to_kill_criminal,
        ] {
            writer.write_i32(value);
        }
        writer.write_u32(remaining);
        writer.write_u32(self.frequency_ms);
    }
}

/// Формат конкретного состояния дополняет общий префикс собственным хвостом.
pub trait PeriodicAttackRecord {
    const STATE_ID: u32;
    const RECORD_BYTES: usize;
    fn core(&self) -> &PeriodicAttackCore;
    fn encode_attack(&self, writer: &mut LegacyWriter<'_>);
}

pub fn encode_periodic_state<T: PeriodicAttackRecord>(
    state: &T,
    now: impl FnMut() -> u32,
) -> Vec<u8> {
    encode_with_remaining(state, state.core().client_state_time(now))
}

pub fn encode_periodic_state_for_install<T: PeriodicAttackRecord>(state: &T) -> Vec<u8> {
    encode_with_remaining(state, state.core().keep_time_ms)
}

fn encode_with_remaining<T: PeriodicAttackRecord>(state: &T, remaining: u32) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(T::RECORD_BYTES);
    let mut writer = LegacyWriter::new(&mut bytes);
    writer.write_u32(T::STATE_ID);
    state.core().encode(&mut writer, remaining);
    state.encode_attack(&mut writer);
    bytes
}
