//! Змеиный снаряд `CSnakeBolt` (`0x1a5`) для достигнутого пути монстра.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает общий пошаговый
//! полёт с `CEnergyBolt/CZombieClaw`, но первый шаг начинается с позиции `0`,
//! а область `3×3` применяется только на уровне 3. Живой тип клетки читается
//! перед каждым шагом; столкновение и `BLOCK_UNFLY` сохраняют исходный двойной
//! пакет завершения. Ветвь игрока с MP и `CSoulCollectState` остаётся ниже.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\snakebolt.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\snakebolt.h

// ============================================================================
// FUNCTION: CSnakeBoltEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// Действия 0/1/3 монстра выполняет общий пошаговый владелец; клиентские
// ответы об ошибках игрока остаются исходным материалом.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\snakebolt.cpp:782
// RVA: 0x001335B0
// ADDRESS: 005335b0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSnakeBolt::~CSnakeBolt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\snakebolt.cpp:50
// RVA: 0x00133AF0
// ADDRESS: 00533af0
// PROTOTYPE: void __thiscall ~CSnakeBolt(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSnakeBolt::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Объектная ветвь монстра достигнута в `execute_owned_snake_bolt`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\snakebolt.cpp:204
// RVA: 0x00133BB0
// ADDRESS: 00533bb0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSnakeBolt::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\snakebolt.cpp:236
// RVA: 0x00133D50
// ADDRESS: 00533d50
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSnakeBolt::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\snakebolt.cpp:267
// RVA: 0x00133F00
// ADDRESS: 00533f00
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSnakeBolt::CSnakeBolt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\snakebolt.cpp:33
// RVA: 0x001340D0
// ADDRESS: 005340d0
// PROTOTYPE: undefined __thiscall CSnakeBolt(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSnakeBolt::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// Дальность, повторное использование и доступность цели монстра достигнуты;
// расход MP и сообщения игрока остаются исходным материалом.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\snakebolt.cpp:119
// RVA: 0x00134180
// ADDRESS: 00534180
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSnakeBolt::CalculateAttackPower
// STATUS: PARTIALLY_IMPLEMENTED
// Формула монстра достигнута общим пошаговым владельцем; множитель собранных
// душ относится к ещё не подключённой ветви игрока.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\snakebolt.cpp:710
// RVA: 0x00134390
// ADDRESS: 00534390
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSnakeBolt::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// Объектная атака монстра достигнута общим пошаговым владельцем.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\snakebolt.cpp:664
// RVA: 0x00134570
// ADDRESS: 00534570
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSnakeBolt::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// Атака области монстра достигнута с областью уровня 3 размером `3×3`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\snakebolt.cpp:578
// RVA: 0x001346F0
// ADDRESS: 005346f0
// PROTOTYPE: int __thiscall Attack(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSnakeBolt::AI
// STATUS: PARTIALLY_IMPLEMENTED
// Полный пошаговый путь монстра достигнут через `execute_owned_snake_bolt`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\snakebolt.cpp:319
// RVA: 0x00134A20
// ADDRESS: 00534a20
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

pub(crate) const SNAKE_BOLT_SKILL_ID: u32 = 0x1a5;

#[allow(clippy::too_many_arguments, reason = "обёртка сохраняет конкретного владельца навыка")]
pub(crate) fn execute_owned_snake_bolt<Runtime: GameMainLoopRuntime>(
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
        PathProjectileSpec::new(SNAKE_BOLT_SKILL_ID, 3, true, 0),
        skill_level,
        properties,
        now_ms,
        runtime,
        deaths,
    )
}
