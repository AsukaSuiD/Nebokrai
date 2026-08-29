//! Каноническое состояние `CLifeShieldState`.
//!
//! Состояние хранит уровень навыка для обязательного последующего
//! `CCureState`, проверяет наличие и MP боевого духа, но рассчитанный
//! `lMPDamage` применяет к MP игрока обычный владелец атаки.

use super::curestate::{send_cure_state_visual, CureState};
use super::lifeshield::{
    LIFE_SHIELD_SKILL_ID, SKILL_USAGE_STATE_PERSIST_TIME,
};
use super::manashieldstate::{
    MANA_SHIELD_STATE_BEGIN_MESSAGE, MANA_SHIELD_STATE_END_MESSAGE,
};
use crate::gameserver::appserver::states::attackpower::AttackPower;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LifeShieldState {
    started_at_ms: u32,
    keep_time_ms: u32,
    life: i32,
    hp_factor: u16,
    mp_factor: u16,
    skill_level: i32,
}

fn round_original(value: f32) -> i32 {
    value.round_ties_even() as i32
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

    pub(crate) const fn expired(
        self,
        now_ms: u32,
        player_mana: u32,
        dead: bool,
        war_soul_mana: Option<i32>,
    ) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
            || self.life < 1
            || dead
            || player_mana == 0
            || match war_soul_mana {
                Some(mana) => mana < 1,
                None => true,
            }
    }

    pub(crate) const fn client_time(self, now_ms: u32) -> i32 {
        let deadline = self.started_at_ms.wrapping_add(self.keep_time_ms);
        if deadline <= now_ms {
            0
        } else {
            deadline.wrapping_sub(now_ms) as i32
        }
    }

    pub(crate) fn absorb_damage(
        &mut self,
        damage_factor: f32,
        war_soul_mana: Option<i32>,
        power: &mut AttackPower,
    ) {
        let Some(war_soul_mana) = war_soul_mana else {
            return;
        };
        if self.life <= 0 || war_soul_mana <= 0 || power.hp_damage <= 0 {
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
        } else if (war_soul_mana & 0xffff) < mp_damage {
            let available_mana = war_soul_mana & 0xffff;
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

pub(crate) fn send_life_shield_state_visual(
    game: &mut CGame,
    player_id: i32,
    state: LifeShieldState,
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

pub(crate) fn finish_life_shield_state(
    game: &mut CGame,
    player_id: i32,
    state: LifeShieldState,
    now_ms: u32,
) {
    if game
        .skill_base_properties(state.skill_id(), state.skill_level())
        .map(|properties| properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME))
        .is_some()
    {
        let Some(identity) = game.find_player(player_id).map(|player| player.shape().identity()) else {
            send_life_shield_state_visual(game, player_id, state, false, now_ms);
            return;
        };
        let cure = CureState::new(identity, identity);
        let previous = game
            .find_player_mut(player_id)
            .and_then(|player| player.replace_cure_state(cure));
        if let Some(previous) = previous {
            send_cure_state_visual(game, player_id, previous, false);
        }
        send_cure_state_visual(game, player_id, cure, true);
        let _ = game.publish_player_states(player_id);
    }
    send_life_shield_state_visual(game, player_id, state, false, now_ms);
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранены только не подключённые конструктор по умолчанию и сериализация.

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


// ============================================================================
// FUNCTION: CLifeShieldState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lifeshieldstate.cpp:211
// RVA: 0x001E2C60
// ADDRESS: 005e2c60
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: CLifeShieldState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lifeshieldstate.cpp:226
// RVA: 0x001E2E30
// ADDRESS: 005e2e30
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
