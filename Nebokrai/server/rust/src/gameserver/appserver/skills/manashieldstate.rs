//! Каноническое состояние `CManaShieldState`.
//!
//! Щит `321` хранит срок, остаток прочности, две защиты и WORD-факторы
//! преобразования урона. `absorb_damage` вызывается в исходной позиции
//! `CFightDefense::PreDefense`, до обычной защиты и финального damage factor.

use super::manashield::MANA_SHIELD_SKILL_ID;
use crate::gameserver::appserver::states::attackpower::{AttackPower, AttackPowerType};

pub(crate) const MANA_SHIELD_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
pub(crate) const MANA_SHIELD_STATE_END_MESSAGE: i32 = 0x000b_fe04;

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

fn round_original(value: f32) -> i32 {
    value.round_ties_even() as i32
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

    pub(crate) const fn expired(self, now_ms: u32, player_mana: u32, player_dead: bool) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
            || self.life < 1
            || player_dead
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
    pub(crate) fn absorb_damage(
        &mut self,
        skill_id: u32,
        damage_factor: f32,
        player_mana: u32,
        power: &mut AttackPower,
    ) {
        if ((530..=545).contains(&skill_id) && skill_id != 544)
            || self.life <= 0
            || player_mana == 0
            || power.hp_damage <= 0
        {
            return;
        }
        let factor = if damage_factor == 0.0 {
            1.0
        } else {
            damage_factor
        };
        power.hp_damage = round_original(power.hp_damage as f32 * factor);
        match power.kind {
            AttackPowerType::Physical => {
                power.hp_damage = power.hp_damage.wrapping_sub(self.physical_defense / 2);
            }
            AttackPowerType::Element => {
                power.hp_damage = power.hp_damage.wrapping_sub(self.element_defense / 2);
            }
            AttackPowerType::Soul => {}
        }
        power.hp_damage = power.hp_damage.max(0);
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

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сырой C++ ниже является комментарием, а не Rust-реализацией.

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
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\manashieldstate.cpp:17
// RVA: 0x001F30D0
// ADDRESS: 005f30d0
// PROTOTYPE: undefined __thiscall CManaShieldState(long param_1, long param_2, long param_3, long param_4, ushort param_5, ushort param_6)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

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
//

// ============================================================================
// FUNCTION: CManaShieldState::~CManaShieldState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\manashieldstate.cpp:45
// RVA: 0x001F31F0
// ADDRESS: 005f31f0
// PROTOTYPE: void __thiscall ~CManaShieldState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CManaShieldState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\manashieldstate.cpp:77
// RVA: 0x001F3200
// ADDRESS: 005f3200
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CManaShieldState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\manashieldstate.cpp:91
// RVA: 0x001F32C0
// ADDRESS: 005f32c0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CManaShieldState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\manashieldstate.cpp:165
// RVA: 0x001F3380
// ADDRESS: 005f3380
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CManaShieldState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\manashieldstate.cpp:66
// RVA: 0x001F3400
// ADDRESS: 005f3400
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CManaShieldState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\manashieldstate.cpp:180
// RVA: 0x001F3520
// ADDRESS: 005f3520
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CManaShieldStateVisualEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\manashieldstate.cpp:195
// RVA: 0x001F3590
// ADDRESS: 005f3590
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





// COMPONENT_VARIANT_END: GameServer
