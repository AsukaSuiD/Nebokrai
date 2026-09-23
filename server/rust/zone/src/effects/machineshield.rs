//! Данные и поглощение урона машинным щитом Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/machineshieldstate.cpp/.h;
//! Serialize 0x005F1EE0, Unserialize 0x005F2000, AI 0x005F34B0,
//! CFightDefense::PreDefense 0x005B0CD8–0x005B0E5F.

use super::time::timed_client_state_time;
use crate::combat::{AttackPower, truncate_original, truncate_original_i64_low};
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
        self.lifetime_expired(now_ms)
            || dead
            || player_mana == 0
    }

    pub fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    pub fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != MACHINE_SHIELD_SKILL_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
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
        bytes.try_into().expect("размер состояния машинного щита фиксирован")
    }

    pub fn encoded_for_install(self) -> [u8; MACHINE_SHIELD_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }
    /// PreDefense меняет урон и прочность до обычной защиты; MP игрока меняется позднее.
    pub fn absorb_damage(
        &mut self,
        damage_factor: f32,
        player_mana: u32,
        power: &mut AttackPower,
    ) {
        power.hp_damage = truncate_original_i64_low(
            f64::from(power.hp_damage) * f64::from(damage_factor),
        );
        if self.life > 0 && player_mana > 0 && power.hp_damage > 0 {
            let hp_factor = f64::from(self.hp_factor) * f64::from(0.01_f32);
            let mp_factor = self.mp_factor as f32 * 0.01_f32;
            let hp_shield = truncate_original(hp_factor * f64::from(power.hp_damage));
            let mp_damage =
                truncate_original(f64::from(mp_factor) * f64::from(power.hp_damage));
            // После прочности оригинал сравнивает расход с младшим WORD MP.
            if self.life < hp_shield {
                let old_life = self.life;
                self.life = 0;
                power.mp_damage = mp_damage;
                power.hp_damage =
                    truncate_original(f64::from(hp_shield.wrapping_sub(old_life)) / hp_factor);
            } else if ((player_mana & 0xffff) as i32) < mp_damage {
                let available_mana = (player_mana & 0xffff) as i32;
                self.life = 0;
                power.mp_damage = mp_damage;
                power.hp_damage = truncate_original(
                    f64::from(mp_damage.wrapping_sub(available_mana)) / f64::from(mp_factor),
                );
            } else {
                self.life = self.life.wrapping_sub(hp_shield);
                power.hp_damage = 0;
                power.mp_damage = mp_damage;
            }
        }
        power.hp_damage = truncate_original(
            f64::from(power.hp_damage) / f64::from(damage_factor),
        );
    }
}
