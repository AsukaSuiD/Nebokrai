//! Базовая стрельба GameServer (`SKILL_BASE_ARCHERY`, ID `2`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/archery.cpp`. Навык исполняется из обычной очереди
//! `CPlayerAI`: проверяет дальность, непролётные клетки и оружие категории
//! лука либо арбалета, блокирует движение на задержку и передаёт попадание
//! региональному `CArcheryPhalanx`. Формулы и порядок RNG применяются только
//! при достижении цели снарядом.

use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::kernel::SkillExecutionKernel;

pub(crate) const ARCHERY_SKILL_ID: u32 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArcheryExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    target: ShapeIdentity,
}

impl ArcheryExecutionState {
    pub(crate) const fn begin(
        dispatch: PlayerSkillDispatch,
        target: ShapeIdentity,
        started_at_ms: u32,
    ) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            target,
        }
    }

    pub(crate) const fn kernel(self) -> SkillExecutionKernel<PlayerSkillDispatch> {
        self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
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
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.h

// ============================================================================
// FUNCTION: CArchery::CArchery
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.cpp:22
// RVA: 0x001B1BB0
// ADDRESS: 005b1bb0
// PROTOTYPE: undefined __thiscall CArchery(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArchery::~CArchery
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.cpp:31
// RVA: 0x001B1C20
// ADDRESS: 005b1c20
// PROTOTYPE: void __thiscall ~CArchery(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArchery::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.cpp:181
// RVA: 0x001B1C40
// ADDRESS: 005b1c40
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArchery::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.cpp:197
// RVA: 0x001B1D10
// ADDRESS: 005b1d10
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArchery::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.cpp:164
// RVA: 0x001B1E00
// ADDRESS: 005b1e00
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArcheryEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.cpp:524
// RVA: 0x001B1EC0
// ADDRESS: 005b1ec0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArchery::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.cpp:226
// RVA: 0x001B2370
// ADDRESS: 005b2370
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArchery::CheckCastCondition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.cpp:38
// RVA: 0x001B2770
// ADDRESS: 005b2770
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_005b27a3
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.cpp:41
// RVA: 0x001B27A3
// ADDRESS: 005b27a3
// PROTOTYPE: undefined FUN_005b27a3()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CArchery::Summon
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\archery.cpp:431
// RVA: 0x001B2B10
// ADDRESS: 005b2b10
// PROTOTYPE: int __thiscall Summon(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
















// COMPONENT_VARIANT_END: GameServer
