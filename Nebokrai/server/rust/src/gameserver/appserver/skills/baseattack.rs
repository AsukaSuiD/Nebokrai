//! Базовая атака GameServer (`SKILL_BASE_ATTACK == 1`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный owner
//! `appserver/skills/baseattack.cpp`. Подтверждённый контракт: первый AI tick
//! проверяет дальность, поворачивает игрока и публикует action 0; после
//! `SKILL_USAGE_DELAY_TIME` action 1 до расчёта атаки; dead target завершает
//! skill action 2, distant target — action `0x0b`. Состояние хранится рядом с
//! canonical `CPlayerAI`, а не в shadow-map CGame. Конкретный PvP damage и
//! сетевые hurt/death effects исполняет CGame caller; прочие skill ID сюда не
//! маршрутизируются.

use crate::gameserver::appserver::player::PlayerSkillDispatch;

pub(crate) const BASE_ATTACK_SKILL_ID: u32 = 1;
pub(crate) const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5003;
pub(crate) const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
pub(crate) const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub(crate) const SKILL_USAGE_USER_HIT_MODIFIER: u32 = 20_001;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BaseAttackExecutionState {
    pub(crate) dispatch: PlayerSkillDispatch,
    pub(crate) started_at_ms: u32,
}

pub(crate) const fn time_reached(now_ms: u32, started_at_ms: u32, delay_ms: u32) -> bool {
    now_ms.wrapping_sub(started_at_ms) >= delay_ms
}

pub(crate) fn real_distance(source_x: i32, source_y: i32, target_x: i32, target_y: i32) -> i32 {
    let x = target_x.wrapping_sub(source_x) as f32;
    let y = target_y.wrapping_sub(source_y) as f32;
    (x.mul_add(x, y * y).sqrt()).round_ties_even() as i32
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp

// ============================================================================
// FUNCTION: CBaseAttack::Restart
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:110
// RVA: 0x00113E00
// ADDRESS: 00513e00
// PROTOTYPE: void __thiscall Restart(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::AfterUseSkill
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:233
// RVA: 0x0013CF30
// ADDRESS: 0053cf30
// PROTOTYPE: void __thiscall AfterUseSkill(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::OnChangeRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:243
// RVA: 0x0016A370
// ADDRESS: 0056a370
// PROTOTYPE: void __thiscall OnChangeRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::CBaseAttack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:18
// RVA: 0x001B2DB0
// ADDRESS: 005b2db0
// PROTOTYPE: undefined __thiscall CBaseAttack(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::~CBaseAttack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:26
// RVA: 0x001B2E20
// ADDRESS: 005b2e20
// PROTOTYPE: void __thiscall ~CBaseAttack(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::CheckCastCondition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:33
// RVA: 0x001B2E40
// ADDRESS: 005b2e40
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:65
// RVA: 0x001B2E70
// ADDRESS: 005b2e70
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:81
// RVA: 0x001B2F40
// ADDRESS: 005b2f40
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::End
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:97
// RVA: 0x001B3010
// ADDRESS: 005b3010
// PROTOTYPE: void __thiscall End(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:48
// RVA: 0x001B3040
// ADDRESS: 005b3040
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttackEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:346
// RVA: 0x001B3100
// ADDRESS: 005b3100
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::CalculateAttackPower
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:279
// RVA: 0x001B3600
// ADDRESS: 005b3600
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::Attack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:248
// RVA: 0x001B3860
// ADDRESS: 005b3860
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseAttack::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\baseattack.cpp:123
// RVA: 0x001B39B0
// ADDRESS: 005b39b0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
