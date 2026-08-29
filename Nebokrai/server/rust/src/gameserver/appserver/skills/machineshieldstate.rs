//! Каноническое состояние `CMachineShieldState`.
//!
//! Состояние хранится в общей упорядоченной ветви щитов и исполняется в
//! исходной точке `CFightDefense::PreDefense`. Формула не подменяет MP игрока:
//! рассчитанный `lMPDamage` применяется позднее обычным владельцем атаки.
//! DB-запись хранит остаток срока, прочность и два WORD-фактора;
//! после загрузки отсчёт начинается от текущих часов.

use super::machineshield::MACHINE_SHIELD_SKILL_ID;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use super::manashieldstate::{
    MANA_SHIELD_STATE_BEGIN_MESSAGE, MANA_SHIELD_STATE_END_MESSAGE,
};
use crate::gameserver::appserver::states::attackpower::AttackPower;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const MACHINE_SHIELD_STATE_BYTES: usize = 16;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MachineShieldState {
    started_at_ms: u32,
    keep_time_ms: u32,
    life: i32,
    hp_factor: u16,
    mp_factor: u16,
}

fn round_original(value: f32) -> i32 {
    value.round_ties_even() as i32
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

    pub(crate) const fn expired(self, now_ms: u32, player_mana: u32, dead: bool) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
            || self.life < 1
            || dead
            || player_mana == 0
    }

    pub(crate) const fn client_time(self, now_ms: u32) -> i32 {
        let deadline = self.started_at_ms.wrapping_add(self.keep_time_ms);
        if deadline <= now_ms {
            0
        } else {
            deadline.wrapping_sub(now_ms) as i32
        }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != MACHINE_SHIELD_SKILL_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(now_ms, reader.read_u32()?, reader.read_i32()?, reader.read_u16()?, reader.read_u16()?))
    }

    pub(crate) fn encoded(self, now_ms: u32) -> [u8; MACHINE_SHIELD_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(MACHINE_SHIELD_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(MACHINE_SHIELD_SKILL_ID);
        writer.write_u32(self.client_time(now_ms) as u32);
        writer.write_i32(self.life);
        writer.write_u16(self.hp_factor);
        writer.write_u16(self.mp_factor);
        bytes.try_into().expect("размер состояния машинного щита фиксирован")
    }

    pub(crate) fn encoded_for_install(self) -> [u8; MACHINE_SHIELD_STATE_BYTES] {
        self.encoded(self.started_at_ms)
    }

    pub(crate) fn activate_loaded(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub(crate) fn absorb_damage(
        &mut self,
        damage_factor: f32,
        player_mana: u32,
        power: &mut AttackPower,
    ) {
        if self.life <= 0 || player_mana == 0 || power.hp_damage <= 0 {
            return;
        }
        let factor = if damage_factor == 0.0 {
            1.0
        } else {
            damage_factor
        };
        power.hp_damage = round_original(power.hp_damage as f32 * factor);
        let hp_factor = self.hp_factor as f32 * 0.01;
        let mp_factor = self.mp_factor as f32 * 0.01;
        if hp_factor <= 0.0 || mp_factor <= 0.0 {
            power.hp_damage = round_original(power.hp_damage as f32 / factor);
            return;
        }
        let hp_shield = round_original(hp_factor * power.hp_damage as f32);
        let mp_damage = round_original(mp_factor * power.hp_damage as f32);
        if self.life < hp_shield {
            let old_life = self.life;
            self.life = 0;
            power.mp_damage = mp_damage;
            power.hp_damage = round_original((hp_shield - old_life) as f32 / hp_factor);
        } else if ((player_mana & 0xffff) as i32) < mp_damage {
            let available_mana = (player_mana & 0xffff) as i32;
            self.life = 0;
            power.mp_damage = mp_damage;
            power.hp_damage = round_original((mp_damage - available_mana) as f32 / mp_factor);
        } else {
            self.life = self.life.wrapping_sub(hp_shield);
            power.hp_damage = 0;
            power.mp_damage = mp_damage;
        }
        power.hp_damage = round_original(power.hp_damage as f32 / factor);
    }
}

pub(crate) fn send_machine_shield_state_visual(
    game: &mut CGame,
    player_id: i32,
    state: MachineShieldState,
    begin: bool,
    now_ms: u32,
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
        message.add_long(state.client_time(now_ms));
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
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\machineshieldstate.cpp

// ============================================================================
// FUNCTION: CMachineShieldState::CMachineShieldState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\machineshieldstate.cpp:27
// RVA: 0x001F1CD0
// ADDRESS: 005f1cd0
// PROTOTYPE: undefined __thiscall CMachineShieldState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.

// COMPONENT_VARIANT_END: GameServer
