//! Базовая магическая атака GameServer (`SKILL_BASE_MAGIC == 3`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/basemagic.cpp`. `SkillExecutionKernel` сохраняет применение
//! между тактами: первый такт проверяет цель, поворачивает игрока, отправляет
//! начало эффекта и запрещает движение; по истечении задержки движение
//! разрешается до повторной проверки цели и отправки пакета выстрела. Сам урон
//! намеренно не выполняется здесь: выстрел создаёт принадлежащий региону
//! `CBaseMagicPhalanx`, который атакует в ИИ региона после отдельной задержки
//! полёта. Это сохраняет исходные моменты действий, двух владельцев жизненного
//! цикла и порядок пакетов.
//! Сырой C++ ниже остаётся локальной документацией ещё не достигнутых
//! перегрузок и владельца визуального эффекта.

use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::kernel::SkillExecutionKernel;

pub(crate) const BASE_MAGIC_SKILL_ID: u32 = 3;
pub(crate) const BASE_MAGIC_EFFECT_MESSAGE: i32 = 0x000b_fe01;
pub(crate) const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
pub(crate) const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub(crate) const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
pub(crate) const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
pub(crate) const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
pub(crate) const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;
pub(crate) const SKILL_USAGE_ELEMENT_MODIFIER: u32 = 20_015;
pub(crate) const SKILL_USAGE_SUMMONED_LIFETIME: u32 = 30_001;
pub(crate) const SKILL_USAGE_SUMMONED_SPEED: u32 = 30_002;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct BaseMagicExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    target: ShapeIdentity,
    condition_checked: bool,
}

impl BaseMagicExecutionState {
    pub(crate) const fn begin(
        dispatch: PlayerSkillDispatch,
        target: ShapeIdentity,
        started_at_ms: u32,
    ) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            target,
            condition_checked: false,
        }
    }

    pub(crate) const fn kernel(self) -> SkillExecutionKernel<PlayerSkillDispatch> {
        self.kernel
    }

    pub(crate) const fn target(self) -> ShapeIdentity {
        self.target
    }

    pub(crate) const fn condition_checked(self) -> bool {
        self.condition_checked
    }

    pub(crate) fn mark_condition_checked(&mut self) {
        self.condition_checked = true;
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagic.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagic.h

// ============================================================================
// FUNCTION: CBaseMagic::CBaseMagic
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagic.cpp:20
// RVA: 0x001B3B70
// ADDRESS: 005b3b70
// PROTOTYPE: undefined __thiscall CBaseMagic(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseMagic::~CBaseMagic
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagic.cpp:29
// RVA: 0x001B3BE0
// ADDRESS: 005b3be0
// PROTOTYPE: void __thiscall ~CBaseMagic(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseMagic::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagic.cpp:156
// RVA: 0x001B3C00
// ADDRESS: 005b3c00
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseMagic::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagic.cpp:172
// RVA: 0x001B3CD0
// ADDRESS: 005b3cd0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseMagic::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagic.cpp:139
// RVA: 0x001B3DC0
// ADDRESS: 005b3dc0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseMagicEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagic.cpp:503
// RVA: 0x001B3E80
// ADDRESS: 005b3e80
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseMagic::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagic.cpp:201
// RVA: 0x001B4330
// ADDRESS: 005b4330
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseMagic::CheckCastCondition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagic.cpp:36
// RVA: 0x001B4700
// ADDRESS: 005b4700
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBaseMagic::Summon
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\basemagic.cpp:394
// RVA: 0x001B49A0
// ADDRESS: 005b49a0
// PROTOTYPE: int __thiscall Summon(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




















// COMPONENT_VARIANT_END: GameServer
