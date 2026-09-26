//! Данные и поглощение урона щитом жизни Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/lifeshieldstate.cpp/.h;
//! Serialize 0x005E2C60, Unserialize 0x005E2E30, AI 0x005E2D90,
//! CFightDefense::PreDefense 0x005B0E64–0x005B1030.
//! Произведение MP-фактора — MATCH по якорю 0x5B0F55–0x5B0F83 (ветвь ID 0x220):
//! умножение идёт неокруглённым x87-продуктом `mp*0.01`; f32-копия
//! (`fst [esp+0x1C]`) создаётся только для деления mana-ветви. Прежняя
//! реконструкция округляла фактор до f32 до умножения (установленное
//! расхождение, здесь исправлено).

use super::time::timed_client_state_time;
use crate::combat::{AttackPower, truncate_original};
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};

pub const LIFE_SHIELD_SKILL_ID: u32 = 544;
pub const LIFE_SHIELD_STATE_BYTES: usize = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LifeShieldState {
    started_at_ms: u32,
    keep_time_ms: u32,
    life: i32,
    hp_factor: u16,
    mp_factor: u16,
    skill_level: i32,
}

impl Default for LifeShieldState {
    fn default() -> Self {
        Self::new(0, 0, 1, 1, 0)
    }
}

impl LifeShieldState {
    pub const fn new(
        keep_time_ms: u32,
        life: i32,
        hp_factor: u16,
        mp_factor: u16,
        skill_level: i32,
    ) -> Self {
        Self {
            started_at_ms: 0,
            keep_time_ms,
            life,
            hp_factor,
            mp_factor,
            skill_level,
        }
    }

    pub fn begin_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub const fn skill_id(self) -> u32 {
        LIFE_SHIELD_SKILL_ID
    }

    pub const fn skill_level(self) -> i32 {
        self.skill_level
    }

    pub const fn life(self) -> i32 {
        self.life
    }

    pub const fn lifetime_expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms || self.life < 1
    }

    pub const fn expired(
        self,
        now_ms: u32,
        player_mana: u32,
        dead: bool,
        war_soul_mana: Option<i32>,
    ) -> bool {
        self.lifetime_expired(now_ms)
            || dead
            || player_mana == 0
            || match war_soul_mana {
                Some(mana) => mana < 1,
                None => true,
            }
    }

    pub fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    pub fn decode(
        payload: &[u8],
        offset: usize,
        now_milliseconds: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != LIFE_SHIELD_SKILL_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        let skill_level = reader.read_i32()?;
        let now_ms = now_milliseconds();
        let mut state = Self::new(
            reader.read_u32()?,
            reader.read_i32()?,
            reader.read_u16()?,
            reader.read_u16()?,
            skill_level,
        );
        state.begin_at(now_ms);
        Ok(state)
    }

    pub fn encoded(&self, now_milliseconds: impl FnMut() -> u32) -> [u8; LIFE_SHIELD_STATE_BYTES] {
        self.encoded_with_remaining(|| self.client_time(now_milliseconds) as u32)
    }

    fn encoded_with_remaining(
        &self,
        remaining_time: impl FnOnce() -> u32,
    ) -> [u8; LIFE_SHIELD_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(LIFE_SHIELD_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(LIFE_SHIELD_SKILL_ID);
        writer.write_i32(self.skill_level);
        writer.write_u32(remaining_time());
        writer.write_i32(self.life);
        writer.write_u16(self.hp_factor);
        writer.write_u16(self.mp_factor);
        bytes
            .try_into()
            .expect("размер состояния щита жизни фиксирован")
    }

    pub fn encoded_for_install(&self) -> [u8; LIFE_SHIELD_STATE_BYTES] {
        self.encoded_with_remaining(|| self.keep_time_ms)
    }
    /// При отсутствии боевого духа исходная ветвь не отменяет начальное масштабирование.
    pub fn absorb_damage(
        &mut self,
        damage_factor: f32,
        war_soul_mana: Option<i32>,
        power: &mut AttackPower,
    ) {
        power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(damage_factor));
        let Some(war_soul_mana) = war_soul_mana else {
            return;
        };
        if self.life > 0 && war_soul_mana > 0 && power.hp_damage > 0 {
            let hp_factor = f64::from(self.hp_factor) * f64::from(0.01_f32);
            // Оригинал умножает неокруглённый x87-продукт `mp*0.01`; f32-копия
            // фактора (`fst [esp+0x1C]`) служит только делителем mana-ветви.
            let mp_factor = f64::from(self.mp_factor) * f64::from(0.01_f32);
            let mp_factor_f32 = mp_factor as f32;
            let hp_shield = truncate_original(hp_factor * f64::from(power.hp_damage));
            let mp_damage = truncate_original(mp_factor * f64::from(power.hp_damage));
            if self.life < hp_shield {
                let old_life = self.life;
                self.life = 0;
                power.mp_damage = mp_damage;
                power.hp_damage =
                    truncate_original(f64::from(hp_shield.wrapping_sub(old_life)) / hp_factor);
            } else if (war_soul_mana & 0xffff) < mp_damage {
                let available_mana = war_soul_mana & 0xffff;
                self.life = 0;
                power.mp_damage = mp_damage;
                power.hp_damage = truncate_original(
                    f64::from(mp_damage.wrapping_sub(available_mana)) / f64::from(mp_factor_f32),
                );
            } else {
                self.life = self.life.wrapping_sub(hp_shield);
                power.hp_damage = 0;
                power.mp_damage = mp_damage;
            }
        }
        power.hp_damage = truncate_original(f64::from(power.hp_damage) / f64::from(damage_factor));
    }
}
