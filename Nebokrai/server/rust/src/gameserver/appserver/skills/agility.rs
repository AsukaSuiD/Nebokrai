//! Общее исполнение семейства `CAgility/CAgility2/CNatural/CRapture`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `appserver/skills/agility.cpp` и `agility2.cpp`. Общая часть сохраняет
//! отдельные часы восстановления, повторную проверку и расход MP, дополнительный
//! `UpdateCurrentState`, задержку и сетевой формат навыка на себя. Нулевая цена MP
//! намеренно не ставит запрет движения, хотя завершение всё равно снимает его.
//! Три постоянных состояния взаимно заменяются, а временная `CAgility2`
//! заменяет только себя. Различающиеся свойства и жизненный цикл принадлежат
//! `CanonicalStateStorage` и вызывающему `CGame`, а не общему
//! `SkillExecutionKernel`.

use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::skills::kernel::SkillExecutionKernel;

pub(crate) const AGILITY_SKILL_ID: u32 = 218;
pub(crate) const AGILITY_2_SKILL_ID: u32 = 129;
pub(crate) const AGILITY_EFFECT_MESSAGE: i32 = 0x000b_fe01;
pub(crate) const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
pub(crate) const SKILL_USAGE_TARGET_FULL_MISS_GAIN: u32 = 127;
pub(crate) const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
pub(crate) const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
pub(crate) const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub(crate) const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AgilityFamilyExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
}

impl AgilityFamilyExecutionState {
    pub(crate) const fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
        }
    }

    pub(crate) const fn kernel(self) -> SkillExecutionKernel<PlayerSkillDispatch> {
        self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }
}

// Статус сохранённых метаданных: UNKNOWN; полный декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сырой C++ ниже является комментарием, а не Rust-реализацией.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agility.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agility.h

// ============================================================================
// FUNCTION: CAgility::End
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agility.cpp:153
// RVA: 0x00146090
// ADDRESS: 00546090
// PROTOTYPE: void __thiscall End(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAgility::CAgility
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agility.cpp:18
// RVA: 0x0016CEA0
// ADDRESS: 0056cea0
// PROTOTYPE: undefined __thiscall CAgility(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAgility::~CAgility
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agility.cpp:26
// RVA: 0x0016CF10
// ADDRESS: 0056cf10
// PROTOTYPE: void __thiscall ~CAgility(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAgility::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agility.cpp:121
// RVA: 0x0016CF30
// ADDRESS: 0056cf30
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAgility::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agility.cpp:137
// RVA: 0x0016D000
// ADDRESS: 0056d000
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAgility::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agility.cpp:104
// RVA: 0x0016D0F0
// ADDRESS: 0056d0f0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAgilityEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agility.cpp:292
// RVA: 0x0016D1B0
// ADDRESS: 0056d1b0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAgility::CheckCastCondition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agility.cpp:38
// RVA: 0x0016D4D0
// ADDRESS: 0056d4d0
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAgility::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agility.cpp:167
// RVA: 0x0016D6B0
// ADDRESS: 0056d6b0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CContainerListener::OnTraversingContainer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\agility.cpp:33
// RVA: 0x001AFCE0
// ADDRESS: 005afce0
// PROTOTYPE: int __thiscall OnTraversingContainer(CContainer * param_1, CBaseObject * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//










// COMPONENT_VARIANT_END: GameServer
