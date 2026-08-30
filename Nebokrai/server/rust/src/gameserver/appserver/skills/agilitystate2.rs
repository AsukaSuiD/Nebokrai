//! Каноническое временное состояние `CAgilityState2`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `agilitystate2.cpp`. Состояние `0x81` добавляет `full_miss` сложением с
//! переполнением, завершается только при строгом `started + keep < now` и при
//! вычислении положительного клиентского остатка второй раз читает часы.
//! Жизненный цикл принадлежит `CanonicalStateStorage`; DB-запись хранит
//! остаток срока и WORD-прибавку полного уклонения.

use super::agility2::AGILITY_2_SKILL_ID;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::gameserver::game::CGame;
use crate::gameserver::appserver::states::state::timed_client_state_time;

pub(crate) const AGILITY_STATE_2_BYTES: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AgilityState2 {
    full_miss: u16,
    started_at_ms: u32,
    keep_time_ms: i32,
}

impl AgilityState2 {
    pub(crate) const fn new(full_miss: u16, started_at_ms: u32, keep_time_ms: i32) -> Self {
        Self {
            full_miss,
            started_at_ms,
            keep_time_ms,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 { AGILITY_2_SKILL_ID }

    pub(crate) fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        properties.full_miss = properties.full_miss.wrapping_add(self.full_miss);
        properties
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms as u32) < now_ms
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(
            self.started_at_ms,
            self.keep_time_ms as u32,
            now_milliseconds,
        ) as i32
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != AGILITY_2_SKILL_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        let keep_time_ms = reader.read_i32()?;
        Ok(Self::new(reader.read_u16()?, now_ms, keep_time_ms))
    }

    pub(crate) fn encoded(self, now_ms: u32) -> [u8; AGILITY_STATE_2_BYTES] {
        let mut bytes = Vec::with_capacity(AGILITY_STATE_2_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(AGILITY_2_SKILL_ID);
        writer.write_i32(self.client_time(|| now_ms));
        writer.write_u16(self.full_miss);
        bytes.try_into().expect("размер временного состояния ловкости фиксирован")
    }

    pub(crate) fn encoded_for_install(self) -> [u8; AGILITY_STATE_2_BYTES] {
        self.encoded(self.started_at_ms)
    }

    pub(crate) fn activate_loaded(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }
}

pub(crate) fn expire_player_agility_state_2(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
) -> bool {
    let ended = game
        .find_player_mut(player_id)
        .and_then(|player| player.take_expired_agility_state_2(now_ms))
        .is_some();
    if ended {
        let _ = game.publish_player_states(player_id);
    }
    ended
}
