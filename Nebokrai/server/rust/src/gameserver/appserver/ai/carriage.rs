//! Владелец переходов повозки `CCarriage`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/ai/carriage.cpp`. Контроллер хранит действие, секундную проверку
//! хозяина, выход хозяина, повторную привязку и таймер исчезновения. `CGame`
//! оставляет пространственное перемещение, привязку игрока, журналирование,
//! пакеты и фактическое удаление.
//!
//! Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
//! Декомпилятор: Ghidra 12.1.2
//! Сохранены ещё не сопоставленные поиск хозяина и технические конструкторы.

pub(crate) const CARRIAGE_FOLLOWING: i32 = 0;
pub(crate) const CARRIAGE_STAYING: i32 = 1;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CarriageLifecycleState {
    action: i32,
    invalid_master_ms: u32,
    seek_master_ms: u32,
    master_logout: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CarriageMasterFacts {
    pub(crate) now_ms: u32,
    pub(crate) disappear_time_ms: u32,
    pub(crate) master_present: bool,
    pub(crate) master_owns_other: bool,
    pub(crate) master_close: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CarriageMasterOutcome {
    pub(crate) checked: bool,
    pub(crate) rebound: bool,
    pub(crate) vanish_reason: Option<i32>,
}

impl CarriageLifecycleState {
    pub(crate) const fn action(self) -> i32 {
        self.action
    }

    pub(crate) const fn set_action(&mut self, action: i32) {
        self.action = action;
    }

    pub(crate) fn tick_master(&mut self, facts: CarriageMasterFacts) -> CarriageMasterOutcome {
        if facts.now_ms.wrapping_sub(self.seek_master_ms) < 1_000 {
            return CarriageMasterOutcome::default();
        }
        self.seek_master_ms = facts.now_ms;
        let mut outcome = CarriageMasterOutcome {
            checked: true,
            ..Default::default()
        };
        if !facts.master_present {
            if !self.master_logout {
                self.master_logout = true;
                if self.invalid_master_ms == 0 {
                    self.action = CARRIAGE_STAYING;
                    self.invalid_master_ms = facts.now_ms;
                }
            }
        } else {
            if !self.master_logout && facts.master_owns_other {
                outcome.vanish_reason = Some(5);
            } else if self.master_logout {
                self.action = CARRIAGE_FOLLOWING;
                self.master_logout = false;
                outcome.rebound = true;
            }
            if facts.master_close {
                self.invalid_master_ms = 0;
            } else if self.invalid_master_ms == 0 {
                self.invalid_master_ms = facts.now_ms;
            }
        }
        if self.invalid_master_ms != 0
            && facts.now_ms.wrapping_sub(self.invalid_master_ms) >= facts.disappear_time_ms
        {
            outcome.vanish_reason.get_or_insert(4);
        }
        outcome
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\carriage.cpp

// ============================================================================
// FUNCTION: CPet::GetPetMaster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\carriage.cpp:351
// RVA: 0x000E97F0
// ADDRESS: 004e97f0
// PROTOTYPE: CMoveShape * __thiscall GetPetMaster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCarriage::CCarriage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\carriage.cpp:24
// RVA: 0x001066D0
// ADDRESS: 005066d0
// PROTOTYPE: undefined __thiscall CCarriage(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCarriage::~CCarriage
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\carriage.cpp:28
// RVA: 0x00106700
// ADDRESS: 00506700
// PROTOTYPE: void __thiscall ~CCarriage(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
