//! Базовая атака боевой феи GameServer (`SKILL_BATTLEFAIRY_BASE_ATTACK`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/battlefairybasemagic.cpp`. Отдельный FIFO боевой феи
//! проходит общий `SkillExecutionKernel`, но не блокирует движение игрока.
//! Начало, выстрел и обязательное завершение используют визуальный тип `700`;
//! ошибки используют отдельный префикс `4`. После выстрела урон откладывается
//! до owned `CBattleFairyBaseMagicPhalanx` в ИИ региона. Сырой C++ ниже
//! сохраняет ещё не достигнутые перегрузки и детали визуального класса.

use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::kernel::SkillExecutionKernel;

pub(crate) const BATTLE_FAIRY_BASE_MAGIC_SKILL_ID: u32 = 0x224;
pub(crate) const BATTLE_FAIRY_VISUAL_OBJECT_TYPE: i32 = 700;
pub(crate) const DENIED_STATE_A: u32 = 0x192;
pub(crate) const DENIED_STATE_B: u32 = 0x67;
pub(crate) const DENIED_STATE_C: u32 = 0xd2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BattleFairyBaseMagicExecutionState {
    kernel: SkillExecutionKernel<BattleFairySkillDispatch>,
    target: ShapeIdentity,
}

impl BattleFairyBaseMagicExecutionState {
    pub(crate) const fn begin(
        dispatch: BattleFairySkillDispatch,
        target: ShapeIdentity,
        started_at_ms: u32,
    ) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            target,
        }
    }

    pub(crate) const fn kernel(self) -> SkillExecutionKernel<BattleFairySkillDispatch> {
        self.kernel
    }

    pub(crate) fn kernel_mut(
        &mut self,
    ) -> &mut SkillExecutionKernel<BattleFairySkillDispatch> {
        &mut self.kernel
    }

    pub(crate) const fn target(self) -> ShapeIdentity {
        self.target
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\battlefairybasemagic.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\battlefairybasemagic.h

// ============================================================================
// FUNCTION: BFBaseAttack::BFBaseAttack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\battlefairybasemagic.cpp:20
// RVA: 0x00116D80
// ADDRESS: 00516d80
// PROTOTYPE: undefined __thiscall BFBaseAttack(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: BFBaseAttack::~BFBaseAttack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\battlefairybasemagic.cpp:29
// RVA: 0x00116DF0
// ADDRESS: 00516df0
// PROTOTYPE: void __thiscall ~BFBaseAttack(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: BFBaseAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\battlefairybasemagic.cpp:154
// RVA: 0x00116E10
// ADDRESS: 00516e10
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: BFBaseAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\battlefairybasemagic.cpp:170
// RVA: 0x00116EE0
// ADDRESS: 00516ee0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: BFBaseAttack::End
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\battlefairybasemagic.cpp:186
// RVA: 0x00116FB0
// ADDRESS: 00516fb0
// PROTOTYPE: void __thiscall End(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: BFBaseAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\battlefairybasemagic.cpp:137
// RVA: 0x00117020
// ADDRESS: 00517020
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: BFBaseAttackEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\battlefairybasemagic.cpp:487
// RVA: 0x001170E0
// ADDRESS: 005170e0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: BFBaseAttack::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\battlefairybasemagic.cpp:205
// RVA: 0x00117610
// ADDRESS: 00517610
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: BFBaseAttack::CheckCastCondition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\battlefairybasemagic.cpp:36
// RVA: 0x001179F0
// ADDRESS: 005179f0
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: BFBaseAttack::Summon
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\battlefairybasemagic.cpp:384
// RVA: 0x00117E40
// ADDRESS: 00517e40
// PROTOTYPE: int __thiscall Summon(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//










// COMPONENT_VARIANT_END: GameServer
