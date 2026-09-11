//! Каноническое состояние `CLifeShieldState`.
//!
//! Состояние хранит уровень навыка для обязательного последующего
//! `CCureState`, проверяет наличие и MP боевого духа, но рассчитанный
//! `lMPDamage` применяет к MP игрока обычный владелец атаки. DB-запись
//! сохраняет уровень, остаток срока, прочность и два WORD-фактора. Vtable
//! exact EXE направляет `GetRemainedTime` на общее тело `CBlindState` по
//! `0x005F2CD0` с условным вторым чтением clock.
//! Все преобразования absorbed damage используют x87-усечение к нулю;
//! `mp_factor` сохраняется в `f32` перед умножением, а `hp_factor` — нет.
//! Первичное масштабирование через `damage_factor` выполняется `FISTP dword`
//! до поиска war-soul goods; ноль единицей не подменяется.
//! AddCure (`0x005E2FD0`) завершает прежний Cure, начинает новый и вызывает
//! UpdateProperty до эффекта завершения самого LifeShield (`0x005E3110`).
//! AddCure использует generic CState::GetSufferer, а не CPlayer cast:
//! опубликованный holder переиспользует тот же Cure Begin/storage/visual.
//! Сам щит остаётся в арене до завершения AddCure и своего visual End.
//! Граница unchecked player MP/war-soul части AI зафиксирована в shieldstate.rs;
//! чистый lifetime-префикс не подставляет вымышленные ресурсы другой форме.

use super::curestate::{
    end_cure_state_key,
    send_cure_state_visual_for_holder, CureState,
};
use super::fightdefense::truncate_original;
use super::lifeshield::{
    LIFE_SHIELD_SKILL_ID, SKILL_USAGE_STATE_PERSIST_TIME,
};
use super::manashieldstate::{
    MANA_SHIELD_STATE_BEGIN_MESSAGE, MANA_SHIELD_STATE_END_MESSAGE,
};
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::states::attackpower::AttackPower;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    resolve_state_move_shape, resolve_state_move_shape_mut, timed_client_state_time,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

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

impl LifeShieldState {
    pub(crate) const fn new(
        started_at_ms: u32,
        keep_time_ms: u32,
        life: i32,
        hp_factor: u16,
        mp_factor: u16,
        skill_level: i32,
    ) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
            life,
            hp_factor,
            mp_factor,
            skill_level,
        }
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
        now_ms: u32,
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
        Ok(Self::new(
            now_ms,
            reader.read_u32()?,
            reader.read_i32()?,
            reader.read_u16()?,
            reader.read_u16()?,
            skill_level,
        ))
    }

    pub(crate) fn encoded(
        self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> [u8; LIFE_SHIELD_STATE_BYTES] {
        self.encoded_with_remaining(self.client_time(now_milliseconds) as u32)
    }

    fn encoded_with_remaining(self, remaining_time_ms: u32) -> [u8; LIFE_SHIELD_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(LIFE_SHIELD_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(LIFE_SHIELD_SKILL_ID);
        writer.write_i32(self.skill_level);
        writer.write_u32(remaining_time_ms);
        writer.write_i32(self.life);
        writer.write_u16(self.hp_factor);
        writer.write_u16(self.mp_factor);
        bytes
            .try_into()
            .expect("размер состояния щита жизни фиксирован")
    }

    pub(crate) fn encoded_for_install(self) -> [u8; LIFE_SHIELD_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }

    pub(crate) fn activate_loaded(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
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

pub(crate) fn send_life_shield_state_visual(
    game: &mut CGame,
    player_id: i32,
    state: LifeShieldState,
    begin: bool,
    now_milliseconds: impl FnMut() -> u32,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let identity = player.shape().identity();
    let mut message = CMessage::new(if begin {
        MANA_SHIELD_STATE_BEGIN_MESSAGE
    } else {
        MANA_SHIELD_STATE_END_MESSAGE
    });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(now_milliseconds));
        message.add_long(state.life());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) fn finish_life_shield_state_for_holder(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    state: LifeShieldState,
) {
    if game.skill_base_properties(state.skill_id(), state.skill_level())
        .map(|properties| properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME)).is_some()
        && resolve_state_move_shape(game, region_id, holder).is_some()
    {
        if let Some(key) = resolve_state_move_shape(game, region_id, holder)
            .and_then(|shape| shape.cure_state_key())
        {
            let _ = end_cure_state_key(game, region_id, holder, key);
        }
        let cure = CureState::new(holder, holder).begin_now();
        send_cure_state_visual_for_holder(game, region_id, holder, cure, true);
        let _ = resolve_state_move_shape_mut(game, region_id, holder)
            .map(|shape| shape.push_cure_state(cure));
        if holder.object_type == 400 {
            let _ = game.update_player_properties(holder.id);
        }
    }
    let mut message = CMessage::new(MANA_SHIELD_STATE_END_MESSAGE);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state.skill_id() as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранён только не подключённый конструктор по умолчанию.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lifeshieldstate.cpp

// ============================================================================
// FUNCTION: CLifeShieldState::CLifeShieldState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lifeshieldstate.cpp:32
// RVA: 0x001E2A50
// ADDRESS: 005e2a50
// PROTOTYPE: undefined __thiscall CLifeShieldState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
