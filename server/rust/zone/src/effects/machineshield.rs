//! Данные и поглощение урона машинным щитом Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/machineshieldstate.cpp/.h;
//! Serialize 0x005F1EE0, Unserialize 0x005F2000, AI 0x005F34B0,
//! CFightDefense::PreDefense 0x005B0CD8–0x005B0E5F.

use super::shieldabsorption::absorb_shield_damage;
use super::time::timed_client_state_time;
use crate::combat::AttackPower;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};

pub const MACHINE_SHIELD_SKILL_ID: u32 = 222;

pub const MACHINE_SHIELD_STATE_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct MachineShieldState {
    started_at_ms: u32,
    keep_time_ms: u32,
    life: i32,
    hp_factor: u16,
    mp_factor: u16,
}

impl Default for MachineShieldState {
    fn default() -> Self {
        Self::new(0, 0, 0, 1, 1)
    }
}

impl MachineShieldState {
    pub const fn new(
        started_at_ms: u32,
        keep_time_ms: u32,
        life: i32,
        hp_factor: u16,
        mp_factor: u16,
    ) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
            life,
            hp_factor,
            mp_factor,
        }
    }

    pub const fn skill_id(self) -> u32 {
        MACHINE_SHIELD_SKILL_ID
    }

    pub const fn life(self) -> i32 {
        self.life
    }

    pub fn begin_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub const fn lifetime_expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms || self.life < 1
    }

    pub const fn expired(self, now_ms: u32, player_mana: u32, dead: bool) -> bool {
        self.lifetime_expired(now_ms) || dead || player_mana == 0
    }

    pub fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    pub fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != MACHINE_SHIELD_SKILL_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(
            now_ms,
            reader.read_u32()?,
            reader.read_i32()?,
            reader.read_u16()?,
            reader.read_u16()?,
        ))
    }

    pub fn encoded(
        self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> [u8; MACHINE_SHIELD_STATE_BYTES] {
        self.encoded_with_remaining(self.client_time(now_milliseconds) as u32)
    }

    fn encoded_with_remaining(self, remaining_time_ms: u32) -> [u8; MACHINE_SHIELD_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(MACHINE_SHIELD_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(MACHINE_SHIELD_SKILL_ID);
        writer.write_u32(remaining_time_ms);
        writer.write_i32(self.life);
        writer.write_u16(self.hp_factor);
        writer.write_u16(self.mp_factor);
        bytes
            .try_into()
            .expect("размер состояния машинного щита фиксирован")
    }

    pub fn encoded_for_install(self) -> [u8; MACHINE_SHIELD_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }
    /// PreDefense меняет урон и прочность до обычной защиты; MP игрока меняется позднее.
    pub fn absorb_damage(&mut self, damage_factor: f32, player_mana: u32, power: &mut AttackPower) {
        absorb_shield_damage(
            &mut self.life,
            self.hp_factor,
            self.mp_factor,
            damage_factor,
            player_mana,
            power,
            None,
        );
    }
}
