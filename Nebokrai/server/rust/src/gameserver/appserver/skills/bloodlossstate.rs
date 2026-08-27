//! Каноническое периодическое состояние потери крови `CBloodLossState`.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/bloodlossstate.cpp`. Состояние хранит снимок владельца,
//! выполняет два отдельных чтения часов, затем ровно два обращения к
//! `legacy MSVCRT RNG`: равномерный урон на включительном диапазоне и проверку
//! критического удара.
//! Применение рассчитанной атаки к независимым владельцам выполняет дочерний
//! исполняющий модуль `CGame`. Сохранение в DB и координатные перегрузки `Begin`
//! остаются ниже как RAW без параллельного изменяемого представления.

use super::bloodloss::BLOOD_LOSS_SKILL_ID;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;
const LEGACY_UNKNOWN_SKILL_ID: u32 = i32::MAX as u32;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum BloodLossStateTick {
    Pending,
    Attack(AttackInformation),
    Ended,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BloodLossState {
    master: MasterInfo,
    started_at_ms: u32,
    keep_time_ms: u32,
    frequency_ms: u32,
    damage_factor_bits: u32,
    damage_modifier_bits: u32,
    minimum_attack: u16,
    maximum_attack: u16,
    attack_count: u32,
}

impl BloodLossState {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют состоянию EXE")]
    pub(crate) const fn new(
        master: MasterInfo,
        started_at_ms: u32,
        keep_time_ms: u32,
        frequency_ms: u32,
        damage_factor: f32,
        damage_modifier: f32,
        minimum_attack: u16,
        maximum_attack: u16,
    ) -> Self {
        Self {
            master,
            started_at_ms,
            keep_time_ms,
            frequency_ms,
            damage_factor_bits: damage_factor.to_bits(),
            damage_modifier_bits: damage_modifier.to_bits(),
            minimum_attack,
            maximum_attack,
            attack_count: 0,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        BLOOD_LOSS_SKILL_ID
    }

    pub(crate) const fn master(self) -> MasterInfo {
        self.master
    }

    pub(crate) const fn client_time(self, now_ms: u32) -> i32 {
        let elapsed = now_ms.wrapping_sub(self.started_at_ms);
        if elapsed >= self.keep_time_ms {
            0
        } else {
            self.keep_time_ms.wrapping_sub(elapsed) as i32
        }
    }

    pub(crate) fn tick(
        &mut self,
        lifetime_now_ms: u32,
        frequency_now_ms: u32,
        target_dead: bool,
        critical_chance: u16,
        critical_rate: f32,
        random: &mut dyn FnMut(i32) -> i32,
    ) -> BloodLossStateTick {
        if lifetime_now_ms.wrapping_sub(self.started_at_ms) > self.keep_time_ms || target_dead {
            return BloodLossStateTick::Ended;
        }
        let delay = self.frequency_ms.wrapping_mul(self.attack_count);
        if frequency_now_ms.wrapping_sub(self.started_at_ms) <= delay {
            return BloodLossStateTick::Pending;
        }
        self.attack_count = self.attack_count.wrapping_add(1);
        BloodLossStateTick::Attack(self.attack(critical_chance, critical_rate, random))
    }

    fn attack(
        self,
        critical_chance: u16,
        critical_rate: f32,
        random: &mut dyn FnMut(i32) -> i32,
    ) -> AttackInformation {
        let minimum = i32::from(self.minimum_attack);
        let maximum = i32::from(self.maximum_attack);
        let span = minimum.abs_diff(maximum).wrapping_add(1) as i32;
        let rolled = minimum.wrapping_add(random(span));
        let mut damage =
            (rolled as f32 + f32::from_bits(self.damage_modifier_bits)) as i32;
        if damage < 0 {
            damage = 0;
        }
        let critical = random(100) < i32::from(critical_chance);
        if critical {
            damage = (damage as f32 * critical_rate).round_ties_even() as i32;
        }
        AttackInformation {
            skill_id: LEGACY_UNKNOWN_SKILL_ID,
            skill_level: 0,
            attacker_type: self.master.master_type,
            attacker_id: self.master.master_id,
            attacker_team_id: self.master.master_team_id,
            attacker_faction_id: self.master.master_guild_id,
            attacker_union_id: self.master.master_union_id,
            hit_modifier: 0,
            damage_factor: f32::from_bits(self.damage_factor_bits),
            damage_modifier: 0,
            critical,
            blast_attack: false,
            full_miss: 0,
            damages: vec![AttackPower {
                kind: AttackPowerType::Physical,
                hp_damage: damage,
                mp_damage: 0,
            }],
        }
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической around-доставки")]
pub(crate) fn send_blood_loss_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: BloodLossState,
    begin: bool,
    now_ms: u32,
) {
    let mut message = CMessage::new(if begin {
        STATE_BEGIN_MESSAGE
    } else {
        STATE_END_MESSAGE
    });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(now_ms));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

// Остаются недостигнутыми восстановление состояния из DB,
// его сохранение и координатные перегрузки Begin.
// ============================================================================
// FUNCTION: CBloodLossState::CBloodLossState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bloodlossstate.cpp:34
// RVA: 0x001E38E0
// ADDRESS: 005e38e0
// PROTOTYPE: undefined __thiscall CBloodLossState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBloodLossState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bloodlossstate.cpp:75
// RVA: 0x001E39E0
// ADDRESS: 005e39e0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBloodLossState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bloodlossstate.cpp:89
// RVA: 0x001E3A80
// ADDRESS: 005e3a80
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBloodLossState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bloodlossstate.cpp:214
// RVA: 0x001E3B40
// ADDRESS: 005e3b40
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBloodLossState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bloodlossstate.cpp:238
// RVA: 0x001E3C70
// ADDRESS: 005e3c70
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
