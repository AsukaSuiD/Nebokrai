//! CLifeShieldState: gameserver.exe/GameServer.pdb, appserver/skills/lifeshieldstate.cpp.
//! Общий shieldstate выполняет object Begin: S guard, часы при U, loop1
//! и немедленный пакет перед append. Payload и visual принадлежат общей арене.
//! DB-запись хранит уровень, остаток срока, прочность и два WORD-фактора.
//! Load читает часы после уровня; Save пишет ID/уровень до чтения остатка.
//! Положительный клиентский остаток требует двух живых чтений часов.
//! Щит проверяет MP боевого духа, но lMPDamage применяет к MP игрока владелец
//! атаки. Absorbed damage усекается к нулю; mp_factor округляется до f32
//! перед умножением, hp_factor — нет. Начальное damage_factor масштабирование
//! использует FISTP dword до поиска war-soul goods, ноль не заменяется единицей.
//! End сначала создаёт Cure для actual S с собственным UpdateProperty,
//! затем обновляет visual и удаляет щит со вторым отдельным UpdateProperty.
//! Для неплеерных holders unchecked MP/war-soul layout AI (0x005E2D90)
//! остаётся неподтверждённым; общий lifetime-префикс не выдумывает их ресурсы.

use super::curestate::{CURE_STATE_SKILL_ID, CureState, begin_primary_cure_state};
use super::fightdefense::truncate_original;
use super::lifeshield::{
    LIFE_SHIELD_SKILL_ID, SKILL_USAGE_STATE_PERSIST_TIME,
};
use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::states::attackpower::AttackPower;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_applied_state_sufferer, resolve_state_move_shape,
    timed_client_state_time,
};
use crate::gameserver::gameserver::game::{CGame, game_tick_milliseconds};

pub(crate) const LIFE_SHIELD_STATE_BYTES: usize = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LifeShieldState {
    started_at_ms: u32,
    keep_time_ms: u32,
    life: i32,
    hp_factor: u16,
    mp_factor: u16,
    skill_level: i32,
}

impl Default for LifeShieldState {
    fn default() -> Self { Self::new(0, 0, 1, 1, 0) }
}

impl LifeShieldState {
    pub(crate) const fn new(
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

    pub(crate) fn begin_at(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub(crate) const fn skill_id(self) -> u32 {
        LIFE_SHIELD_SKILL_ID
    }

    pub(crate) const fn skill_level(self) -> i32 {
        self.skill_level
    }

    pub(crate) const fn life(self) -> i32 {
        self.life
    }

    pub(crate) const fn lifetime_expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms || self.life < 1
    }

    pub(crate) const fn expired(
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

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    pub(crate) fn decode(
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

    pub(crate) fn encoded(
        &self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> [u8; LIFE_SHIELD_STATE_BYTES] {
        self.encoded_with_remaining(|| self.client_time(now_milliseconds) as u32)
    }

    fn encoded_with_remaining(
        &self, remaining_time: impl FnOnce() -> u32,
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

    pub(crate) fn encoded_for_install(&self) -> [u8; LIFE_SHIELD_STATE_BYTES] {
        self.encoded_with_remaining(|| self.keep_time_ms)
    }


    pub(crate) fn absorb_damage(
        &mut self,
        damage_factor: f32,
        war_soul_mana: Option<i32>,
        power: &mut AttackPower,
    ) {
        power.hp_damage = truncate_original(
            f64::from(power.hp_damage) * f64::from(damage_factor),
        );
        let Some(war_soul_mana) = war_soul_mana else {
            return;
        };
        if self.life > 0 && war_soul_mana > 0 && power.hp_damage > 0 {
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
            } else if (war_soul_mana & 0xffff) < mp_damage {
                let available_mana = war_soul_mana & 0xffff;
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

/// Пролог LifeShield End; visual и Remove самого щита остаются общему End.
pub(crate) fn add_life_shield_cure(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, state: LifeShieldState, key: StateKey,
) {
    let Some(properties) = game.skill_base_properties(state.skill_id(), state.skill_level()).cloned()
    else { return; };
    let Some(target) = resolve_applied_state_sufferer(game, region_id, holder, key) else { return; };
    let Some(shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let previous = shape.find_state_position(|state| state.state_id() == CURE_STATE_SKILL_ID);
    if let Some((position, _)) = previous {
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    }
    // Срок читается из сохранённой таблицы после полного End прежнего Cure.
    // Новый Begin использует actual S как U и S; сам LifeShield ещё в арене.
    let keep = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let _ = begin_primary_cure_state(
        game, target.0, target.1, Some(target), Some(target), CureState::new(keep),
        &mut game_tick_milliseconds,
    );
    let _ = game.update_move_shape_properties(target.0, target.1);
}
