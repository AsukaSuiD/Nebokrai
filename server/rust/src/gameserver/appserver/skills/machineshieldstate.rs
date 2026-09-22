//! CMachineShieldState: gameserver.exe + GameServer.pdb, appserver/skills/machineshieldstate.cpp.
//! Общий shieldstate владеет первичным Begin/ареной, restart, AI и End;
//! payload исполняется в PreDefense. Формула не подменяет MP игрока:
//! рассчитанный `lMPDamage` применяется позднее обычным владельцем атаки.
//! DB-запись хранит остаток срока, прочность и два WORD-фактора;
//! после загрузки отсчёт начинается от текущих часов. Клиентский срок совпадает
//! с Blind и условно читает часы второй раз.
//! Все преобразования absorbed damage используют x87-усечение к нулю;
//! `mp_factor` сохраняется в `f32` перед умножением, а `hp_factor` — нет.
//! Первичное масштабирование через `damage_factor` использует `__ftol2` и
//! младшие 32 бита до проверок состояния; ноль единицей не подменяется.

use super::fightdefense::truncate_original;
use super::machineshield::MACHINE_SHIELD_SKILL_ID;
use super::thunder::truncate_original_i64_low;
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::states::attackpower::AttackPower;
use crate::gameserver::appserver::states::state::timed_client_state_time;

pub(crate) const MACHINE_SHIELD_STATE_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MachineShieldState {
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
    pub(crate) const fn new(
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

    pub(crate) const fn skill_id(self) -> u32 {
        MACHINE_SHIELD_SKILL_ID
    }

    pub(crate) const fn life(self) -> i32 {
        self.life
    }

    pub(crate) fn begin_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub(crate) const fn lifetime_expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms || self.life < 1
    }

    pub(crate) const fn expired(self, now_ms: u32, player_mana: u32, dead: bool) -> bool {
        self.lifetime_expired(now_ms)
            || dead
            || player_mana == 0
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != MACHINE_SHIELD_SKILL_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(now_ms, reader.read_u32()?, reader.read_i32()?, reader.read_u16()?, reader.read_u16()?))
    }

    pub(crate) fn encoded(
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

    pub(crate) fn encoded_for_install(self) -> [u8; MACHINE_SHIELD_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }


    pub(crate) fn absorb_damage(
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
