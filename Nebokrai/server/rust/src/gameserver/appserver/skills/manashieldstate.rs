//! Каноническое состояние `CManaShieldState`.
//!
//! Щит `321` хранит срок, остаток прочности, две защиты и WORD-факторы
//! преобразования урона. `absorb_damage` вызывается в исходной позиции
//! `CFightDefense::PreDefense`, до обычной защиты и итогового коэффициента
//! `damage_factor`. DB-запись хранит остаток срока, прочность, обе
//! защиты и WORD-факторы; после загрузки отсчёт начинается от текущих часов.
//! Vtable exact EXE подтверждает общий с `CBlindState` клиентский срок по
//! `0x005F2CD0`, включая условное второе чтение clock.
//! Все преобразования absorbed damage используют x87-усечение к нулю;
//! `mp_factor` сохраняется в `f32` перед умножением, а `hp_factor` — нет.
//! Первичное масштабирование через `damage_factor` использует `__ftol2` и
//! младшие 32 бита до проверок состояния; ноль единицей не подменяется.

use super::fightdefense::truncate_original;
use super::manashield::MANA_SHIELD_SKILL_ID;
use super::thunder::truncate_original_i64_low;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::states::attackpower::{AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const MANA_SHIELD_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
pub(crate) const MANA_SHIELD_STATE_END_MESSAGE: i32 = 0x000b_fe04;
pub(crate) const MANA_SHIELD_STATE_BYTES: usize = 24;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ManaShieldState {
    started_at_ms: u32,
    keep_time_ms: u32,
    life: i32,
    physical_defense: i32,
    element_defense: i32,
    hp_factor: u16,
    mp_factor: u16,
}

impl ManaShieldState {
    pub(crate) const fn new(
        started_at_ms: u32,
        keep_time_ms: u32,
        life: i32,
        physical_defense: i32,
        element_defense: i32,
        hp_factor: u16,
        mp_factor: u16,
    ) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
            life,
            physical_defense,
            element_defense,
            hp_factor,
            mp_factor,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        MANA_SHIELD_SKILL_ID
    }

    pub(crate) const fn life(self) -> i32 {
        self.life
    }

    pub(crate) const fn lifetime_expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms || self.life < 1
    }

    pub(crate) const fn expired(self, now_ms: u32, player_mana: u32, player_dead: bool) -> bool {
        self.lifetime_expired(now_ms)
            || player_dead
            || player_mana == 0
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != MANA_SHIELD_SKILL_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(now_ms, reader.read_u32()?, reader.read_i32()?, reader.read_i32()?, reader.read_i32()?, reader.read_u16()?, reader.read_u16()?))
    }

    pub(crate) fn encoded(
        self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> [u8; MANA_SHIELD_STATE_BYTES] {
        self.encoded_with_remaining(self.client_time(now_milliseconds) as u32)
    }

    fn encoded_with_remaining(self, remaining_time_ms: u32) -> [u8; MANA_SHIELD_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(MANA_SHIELD_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(MANA_SHIELD_SKILL_ID);
        writer.write_u32(remaining_time_ms);
        writer.write_i32(self.life);
        writer.write_i32(self.physical_defense);
        writer.write_i32(self.element_defense);
        writer.write_u16(self.hp_factor);
        writer.write_u16(self.mp_factor);
        bytes.try_into().expect("размер состояния мана-щита фиксирован")
    }

    pub(crate) fn encoded_for_install(self) -> [u8; MANA_SHIELD_STATE_BYTES] {
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
            match power.kind {
                AttackPowerType::Physical => {
                    power.hp_damage = power.hp_damage.wrapping_sub(self.physical_defense / 2);
                }
                AttackPowerType::Element => {
                    power.hp_damage = power.hp_damage.wrapping_sub(self.element_defense / 2);
                }
                AttackPowerType::Soul => {}
                AttackPowerType::Poison => {}
            }
            power.hp_damage = power.hp_damage.max(0);
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

pub(crate) fn send_mana_shield_state_visual(
    game: &mut CGame,
    player_id: i32,
    state: ManaShieldState,
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

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Ниже сохранён только ещё не подключённый конструктор по умолчанию.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\manashieldstate.cpp

// ============================================================================
// FUNCTION: CManaShieldState::CManaShieldState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\manashieldstate.cpp:31
// RVA: 0x001F3170
// ADDRESS: 005f3170
// PROTOTYPE: undefined __thiscall CManaShieldState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//

// COMPONENT_VARIANT_END: GameServer
