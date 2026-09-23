//! Данные, запись и расчёт ослабления `CPoisonFogState` в Zone.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/poisonfogstate.cpp` и `poisonfogstate.h`.
//! Конструктор VA `0x00607C40`, vtable `0x006622D4`, writer `0x00607F50`,
//! reader `0x006084C0`, property callback `0x00608060`.

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};

use crate::combat::{truncate_original, truncate_original_i64_low};

use super::timed_client_state_time;

pub const POISON_FOG_STATE_ID: u32 = 0xc9;
pub const POISON_FOG_STATE_BYTES: usize = 36;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoisonFogState {
    skill_level: i32,
    started_at_ms: u32,
    keep_time_ms: u32,
    defense_loss: u32,
    defense_loss_coefficient: u32,
    dodge_loss: u32,
    element_resistance_loss: u32,
    element_resistance_loss_coefficient: u32,
    weapon_damage_level: u32,
}

impl PoisonFogState {
    #[allow(
        clippy::too_many_arguments,
        reason = "поля соответствуют состоянию EXE"
    )]
    pub const fn new(
        skill_level: i32,
        keep_time_ms: u32,
        defense_loss: u32,
        defense_loss_coefficient: u32,
        dodge_loss: u32,
        element_resistance_loss: u32,
        element_resistance_loss_coefficient: u32,
        weapon_damage_level: u32,
    ) -> Self {
        Self {
            skill_level,
            started_at_ms: 0,
            keep_time_ms,
            defense_loss,
            defense_loss_coefficient,
            dodge_loss,
            element_resistance_loss,
            element_resistance_loss_coefficient,
            weapon_damage_level,
        }
    }

    pub const fn skill_id(&self) -> u32 {
        POISON_FOG_STATE_ID
    }

    pub const fn skill_level(&self) -> i32 {
        self.skill_level
    }

    pub fn begin_at(&mut self, started_at_ms: u32) {
        self.started_at_ms = started_at_ms;
    }

    pub const fn expired(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub fn client_time(&self, now: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now) as i32
    }

    // В player-ветви x87-произведение сохраняется как float до ограничения
    // текущим свойством; __ftol2 затем отдаёт младшее WORD даже выше INT_MAX.
    fn player_loss(&self, target_level: u8, coefficient: u32, maximum: u32, current: u32) -> u32 {
        let scaled = if coefficient == 0 {
            0.0
        } else {
            let difference = f64::from(self.weapon_damage_level) - f64::from(target_level);
            let ratio = (difference / f64::from(coefficient)).clamp(0.0, 1.0);
            (f64::from(maximum) * ratio) as f32
        };
        let capped = if f64::from(current) < f64::from(scaled) {
            f64::from(current) as f32
        } else {
            scaled
        };
        truncate_original_i64_low(f64::from(capped)) as u32 & 0xffff
    }

    /// Возвращает новые значения двух свойств; Game записывает их живому игроку.
    pub fn player_properties(
        &self,
        target_level: u8,
        defense: u32,
        element_resistance: u32,
    ) -> (u32, u32) {
        let defense_loss = self.player_loss(
            target_level,
            self.defense_loss_coefficient,
            self.defense_loss,
            defense,
        );
        let resistance_loss = self.player_loss(
            target_level,
            self.element_resistance_loss_coefficient,
            self.element_resistance_loss,
            element_resistance,
        );
        (
            defense.wrapping_sub(defense_loss).min(i32::MAX as u32),
            element_resistance
                .wrapping_sub(resistance_loss)
                .min(i32::MAX as u32),
        )
    }

    // В monster-ветви разница уровней записывается во float заранее;
    // произведение защиты остаётся в x87, сопротивление записывается во float.
    pub fn monster_losses(&self, target_level: u8) -> (u32, u32) {
        let difference = (f64::from(self.weapon_damage_level) - f64::from(target_level)) as f32;
        let ratio = |coefficient: u32| {
            if coefficient == 0 {
                0.0
            } else {
                (f64::from(difference) / f64::from(coefficient)).clamp(0.0, 1.0)
            }
        };
        let defense =
            truncate_original(f64::from(self.defense_loss) * ratio(self.defense_loss_coefficient))
                as u32;
        let resistance = (f64::from(self.element_resistance_loss)
            * ratio(self.element_resistance_loss_coefficient)) as f32;
        (defense, truncate_original(f64::from(resistance)) as u32)
    }

    pub fn decode(
        payload: &[u8],
        offset: usize,
        now: &mut dyn FnMut() -> u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != POISON_FOG_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        let skill_level = reader.read_i32()?;
        let started_at_ms = now();
        Ok(Self {
            skill_level,
            started_at_ms,
            keep_time_ms: reader.read_u32()?,
            defense_loss: reader.read_u32()?,
            defense_loss_coefficient: reader.read_u32()?,
            dodge_loss: reader.read_u32()?,
            element_resistance_loss: reader.read_u32()?,
            element_resistance_loss_coefficient: reader.read_u32()?,
            weapon_damage_level: reader.read_u32()?,
        })
    }

    pub fn encoded(&self, now: impl FnMut() -> u32) -> [u8; POISON_FOG_STATE_BYTES] {
        self.encode_record(|| self.client_time(now) as u32)
    }

    pub fn encoded_for_install(&self) -> [u8; POISON_FOG_STATE_BYTES] {
        self.encode_record(|| self.keep_time_ms)
    }

    fn encode_record(&self, remaining: impl FnOnce() -> u32) -> [u8; POISON_FOG_STATE_BYTES] {
        let mut record = Vec::with_capacity(POISON_FOG_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut record);
        writer.write_u32(POISON_FOG_STATE_ID);
        writer.write_i32(self.skill_level);
        writer.write_u32(remaining());
        writer.write_u32(self.defense_loss);
        writer.write_u32(self.defense_loss_coefficient);
        writer.write_u32(self.dodge_loss);
        writer.write_u32(self.element_resistance_loss);
        writer.write_u32(self.element_resistance_loss_coefficient);
        writer.write_u32(self.weapon_damage_level);
        record
            .try_into()
            .expect("размер записи ядовитого тумана фиксирован")
    }
}
