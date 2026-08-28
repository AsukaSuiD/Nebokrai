//! Каноническое периодическое состояние `CSpiderPoisonState` (`0x191`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/spiderpoisonstate.cpp`. Состояние хранит снимок
//! `tagMasterInfo`, использует два чтения часов и строгую границу `>` для
//! срока и периодического удара. Формула яда и wire-сообщения принадлежат
//! этому модулю; `CGame` координирует независимых владельцев цели и смерти.
//! Координатные перегрузки `Begin` остаются RAW ниже.

use super::spiderpoison::SPIDER_POISON_SKILL_ID;
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
pub(crate) enum SpiderPoisonStateTick {
    Pending,
    Attack(AttackInformation),
    Ended,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpiderPoisonState {
    master: MasterInfo,
    started_at_ms: u32,
    keep_time_ms: u32,
    frequency_ms: u32,
    hp_loss: u32,
    attack_count: u32,
}

impl SpiderPoisonState {
    pub(crate) const fn new(
        master: MasterInfo,
        started_at_ms: u32,
        keep_time_ms: u32,
        frequency_ms: u32,
        hp_loss: u32,
    ) -> Self {
        Self { master, started_at_ms, keep_time_ms, frequency_ms, hp_loss, attack_count: 0 }
    }

    pub(crate) const fn skill_id(self) -> u32 { SPIDER_POISON_SKILL_ID }
    pub(crate) const fn master(self) -> MasterInfo { self.master }

    pub(crate) const fn client_time(self, now_ms: u32) -> i32 {
        let elapsed = now_ms.wrapping_sub(self.started_at_ms);
        if elapsed >= self.keep_time_ms { 0 } else { self.keep_time_ms.wrapping_sub(elapsed) as i32 }
    }

    pub(crate) fn tick(
        &mut self,
        lifetime_now_ms: u32,
        frequency_now_ms: u32,
        target_dead: bool,
    ) -> SpiderPoisonStateTick {
        if lifetime_now_ms.wrapping_sub(self.started_at_ms) > self.keep_time_ms || target_dead {
            return SpiderPoisonStateTick::Ended;
        }
        let delay = self.frequency_ms.wrapping_mul(self.attack_count);
        if frequency_now_ms.wrapping_sub(self.started_at_ms) <= delay {
            return SpiderPoisonStateTick::Pending;
        }
        self.attack_count = self.attack_count.wrapping_add(1);
        SpiderPoisonStateTick::Attack(AttackInformation {
            skill_id: LEGACY_UNKNOWN_SKILL_ID,
            skill_level: 0,
            attacker_type: self.master.master_type,
            attacker_id: self.master.master_id,
            attacker_team_id: self.master.master_team_id,
            attacker_faction_id: self.master.master_guild_id,
            attacker_union_id: self.master.master_union_id,
            hit_modifier: 0,
            damage_factor: 1.0,
            damage_modifier: 0,
            critical: false,
            blast_attack: false,
            full_miss: 0,
            damages: vec![AttackPower {
                kind: AttackPowerType::Poison,
                hp_damage: self.hp_loss as i32,
                mp_damage: 0,
            }],
        })
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической круговой доставки")]
pub(crate) fn send_spider_poison_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: SpiderPoisonState,
    begin: bool,
    now_ms: u32,
) {
    let mut message = CMessage::new(if begin { STATE_BEGIN_MESSAGE } else { STATE_END_MESSAGE });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(now_ms));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoisonstate.cpp

// ============================================================================
// FUNCTION: CSpiderPoisonState::CSpiderPoisonState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoisonstate.cpp:17
// RVA: 0x001E90C0
// ADDRESS: 005e90c0
// PROTOTYPE: undefined __thiscall CSpiderPoisonState(tagMasterInfo * param_1, ulong param_2, ulong param_3, ulong param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoisonState::CSpiderPoisonState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoisonstate.cpp:30
// RVA: 0x001E9170
// ADDRESS: 005e9170
// PROTOTYPE: undefined __thiscall CSpiderPoisonState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoisonState::~CSpiderPoisonState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoisonstate.cpp:43
// RVA: 0x001E9200
// ADDRESS: 005e9200
// PROTOTYPE: void __thiscall ~CSpiderPoisonState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoisonState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoisonstate.cpp:69
// RVA: 0x001E9270
// ADDRESS: 005e9270
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoisonState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoisonstate.cpp:83
// RVA: 0x001E9310
// ADDRESS: 005e9310
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoisonState::Begin
// STATUS: IMPLEMENTED
// IMPLEMENTED: `SpiderPoisonState::new` и централизованная замена состояния.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoisonstate.cpp:55
// RVA: 0x001E9430
// ADDRESS: 005e9430
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoisonStateVisualEffect::UpdateVisualEffect
// STATUS: IMPLEMENTED
// IMPLEMENTED: `send_spider_poison_state_visual`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoisonstate.cpp:228
// RVA: 0x001E94D0
// ADDRESS: 005e94d0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoisonState::CalculateAttackPower
// STATUS: IMPLEMENTED
// IMPLEMENTED: `SpiderPoisonState::tick`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoisonstate.cpp:163
// RVA: 0x001E9610
// ADDRESS: 005e9610
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, tagAttackInformation * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoisonState::AI
// STATUS: IMPLEMENTED
// IMPLEMENTED: `SpiderPoisonState::tick` и периодический runtime-владелец.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoisonstate.cpp:113
// RVA: 0x001E96B0
// ADDRESS: 005e96b0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: GameServer
