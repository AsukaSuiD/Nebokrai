//! Широкая атака владыки `CLordWiderangingAttack` (`0x1f6`) для пути монстра.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/lordwiderangingattack.cpp`. Точечная проверка EXE
//! подтвердила отдельные глобальные данные `0x006A1608..0x006A162C`: полную маску
//! `5×5` и две дуги по три клетки. Исполнение, формула и wire-контракт полностью
//! совпадают с `CMachineryStomp`, кроме ID и таблицы свойств, поэтому достигнутый
//! путь монстра использует один узкий семейный механизм. Варианты игрока остаются
//! RAW ниже.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordwiderangingattack.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordwiderangingattack.h

// ============================================================================
// FUNCTION: CLordWiderangingAttack::CLordWiderangingAttack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordwiderangingattack.cpp:30
// RVA: 0x0012EDE0
// ADDRESS: 0052ede0
// PROTOTYPE: undefined __thiscall CLordWiderangingAttack(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordWiderangingAttack::~CLordWiderangingAttack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordwiderangingattack.cpp:38
// RVA: 0x0012EE50
// ADDRESS: 0052ee50
// PROTOTYPE: void __thiscall ~CLordWiderangingAttack(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordWiderangingAttack::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Объектный вход монстра использует `prepare_owned_lord_wideranging_attack`;
// координатные варианты и варианты игрока остаются ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordwiderangingattack.cpp:141
// RVA: 0x0012EE70
// ADDRESS: 0052ee70
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordWiderangingAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordwiderangingattack.cpp:161
// RVA: 0x0012EF50
// ADDRESS: 0052ef50
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordWiderangingAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordwiderangingattack.cpp:120
// RVA: 0x0012F050
// ADDRESS: 0052f050
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordWiderangingAttackEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// Действия 0/1 монстра формирует общий семейный механизм; клиентские ошибки игрока
// остаются исходным материалом.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordwiderangingattack.cpp:635
// RVA: 0x0012F120
// ADDRESS: 0052f120
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordWiderangingAttack::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// Перезарядка, дальность, `BLOCK_UNFLY` и блокировка движения для входа монстра
// выполняются общим семейным механизмом.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordwiderangingattack.cpp:45
// RVA: 0x0012F6B0
// ADDRESS: 0052f6b0
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordWiderangingAttack::GetOutsideCells
// STATUS: IMPLEMENTED
// Точная знаковая граница и обе дуги совпадают с `CMachineryStomp` и выполняются
// общим семейным механизмом.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordwiderangingattack.cpp:358
// RVA: 0x0012F8A0
// ADDRESS: 0052f8a0
// PROTOTYPE: void __thiscall GetOutsideCells(long param_1, long param_2, long param_3, long param_4, vector<CSkill::tagCell,std::allocator<CSkill::tagCell>_> * param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordWiderangingAttack::CalculateAttackPower
// STATUS: PARTIALLY_IMPLEMENTED
// Формула монстра и оба вызова RNG выполняются общим семейным механизмом;
// свойства игрока остаются в теле ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordwiderangingattack.cpp:567
// RVA: 0x0012FB50
// ADDRESS: 0052fb50
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordWiderangingAttack::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// Цель-монстр проходит общий упорядоченный владелец исполнения; снимок разрешений
// игрока остаётся в теле ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordwiderangingattack.cpp:543
// RVA: 0x0012FDC0
// ADDRESS: 0052fdc0
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLordWiderangingAttack::AI
// STATUS: PARTIALLY_IMPLEMENTED
// Стадии выполнения монстра, порядок клеток и завершение выполняет рабочий путь;
// вход игрока остаётся RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\lordwiderangingattack.cpp:194
// RVA: 0x0012FED0
// ADDRESS: 0052fed0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//














// COMPONENT_VARIANT_END: GameServer

use super::machinerystomp::{WideArcAttackDispatch, prepare_owned_wide_arc_attack};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) const LORD_WIDERANGING_ATTACK_SKILL_ID: u32 = 0x1f6;

#[allow(clippy::too_many_arguments, reason = "обёртка сохраняет конкретного владельца навыка")]
pub(crate) fn prepare_owned_lord_wideranging_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    dispatch: &mut Option<WideArcAttackDispatch>,
) -> bool {
    prepare_owned_wide_arc_attack(
        game,
        region,
        monster_id,
        target_identity,
        LORD_WIDERANGING_ATTACK_SKILL_ID,
        skill_level,
        properties,
        now_ms,
        runtime,
        dispatch,
    )
}
