//! `CZombieClaw` для достигнутого monster-owner-а.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает общий с
//! `CEnergyBolt` пошаговый путь и отличающуюся таблицу scope: только третий
//! уровень атакует 3×3 клетки. Конкретный owner задаёт ID и scope-политику,
//! а общий узкий runtime сохраняет packet layout, такты полёта и порядок
//! X→Y. Player MP, `CSoulCollectState` и координатные overload-ы остаются RAW.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\zombieclaw.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\zombieclaw.h

// ============================================================================
// FUNCTION: CZombieClawEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\zombieclaw.cpp:775
// RVA: 0x00136D80
// ADDRESS: 00536d80
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CZombieClaw::~CZombieClaw
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\zombieclaw.cpp:50
// RVA: 0x001372C0
// ADDRESS: 005372c0
// PROTOTYPE: void __thiscall ~CZombieClaw(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CZombieClaw::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\zombieclaw.cpp:200
// RVA: 0x00137380
// ADDRESS: 00537380
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CZombieClaw::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\zombieclaw.cpp:232
// RVA: 0x00137520
// ADDRESS: 00537520
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CZombieClaw::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\zombieclaw.cpp:263
// RVA: 0x001376D0
// ADDRESS: 005376d0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CZombieClaw::CZombieClaw
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\zombieclaw.cpp:33
// RVA: 0x001378A0
// ADDRESS: 005378a0
// PROTOTYPE: undefined __thiscall CZombieClaw(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CZombieClaw::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\zombieclaw.cpp:119
// RVA: 0x00137950
// ADDRESS: 00537950
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CZombieClaw::CalculateAttackPower
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\zombieclaw.cpp:704
// RVA: 0x00137B50
// ADDRESS: 00537b50
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CZombieClaw::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\zombieclaw.cpp:658
// RVA: 0x00137D10
// ADDRESS: 00537d10
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CZombieClaw::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\zombieclaw.cpp:570
// RVA: 0x00137E90
// ADDRESS: 00537e90
// PROTOTYPE: int __thiscall Attack(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CZombieClaw::AI
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\zombieclaw.cpp:315
// RVA: 0x001381C0
// ADDRESS: 005381c0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



















// COMPONENT_VARIANT_END: GameServer

use super::energybolt::{PathProjectileSpec, execute_owned_path_projectile};
use super::monsterattack::MonsterAttackDeath;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) const ZOMBIE_CLAW_SKILL_ID: u32 = 0x1a2;

#[allow(clippy::too_many_arguments, reason = "обёртка сохраняет конкретного владельца навыка")]
pub(crate) fn execute_owned_zombie_claw<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    deaths: &mut Vec<MonsterAttackDeath>,
) -> bool {
    execute_owned_path_projectile(
        game,
        region,
        monster_id,
        target_identity,
        PathProjectileSpec::new(ZOMBIE_CLAW_SKILL_ID, 3, true, 1),
        skill_level,
        properties,
        now_ms,
        runtime,
        deaths,
    )
}
