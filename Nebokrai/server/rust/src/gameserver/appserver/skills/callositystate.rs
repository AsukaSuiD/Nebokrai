//! Каноническое состояние пары `CCallosityState/CCallosityState2`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `appserver/skills/callositystate.cpp` и `callositystate2.cpp`. Состояния
//! взаимно исключают друг друга и хранятся одним типизированным владельцем.
//! Подтверждённая
//! странность сохранена: `time_to_keep` не обслуживается отдельным `AI`, а
//! унаследованные `GetClientStateTime/GetAdditionalData` возвращают нули.
//! Коэффициент `CCH` применяется только при общем `UpdateProperty`; каждый
//! такой проход повторно публикует начальный визуальный эффект, как
//! `OnUpdateProperties`.

use super::callosity::{CALLOSITY_2_SKILL_ID, CALLOSITY_SKILL_ID};

pub(crate) const CALLOSITY_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CallosityState {
    skill_id: u32,
    blast_factor: u16,
    time_to_keep: i32,
}

impl CallosityState {
    pub(crate) const fn new(skill_id: u32, blast_factor: u16, time_to_keep: i32) -> Self {
        debug_assert!(skill_id == CALLOSITY_SKILL_ID || skill_id == CALLOSITY_2_SKILL_ID);
        Self {
            skill_id,
            blast_factor,
            time_to_keep,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        self.skill_id
    }

    pub(crate) const fn blast_factor(self) -> u16 {
        self.blast_factor
    }

    pub(crate) const fn time_to_keep(self) -> i32 {
        self.time_to_keep
    }

    pub(crate) const fn client_state_time(self) -> i32 {
        0
    }

    pub(crate) const fn additional_data(self) -> u32 {
        0
    }
}

// Статус сохранённых метаданных: UNKNOWN; полный декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сырой C++ ниже является комментарием, а не Rust-реализацией.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\callositystate.cpp

// ============================================================================
// FUNCTION: CCallosityState::OnUpdateProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\callositystate.cpp:46
// RVA: 0x001F10B0
// ADDRESS: 005f10b0
// PROTOTYPE: int __thiscall OnUpdateProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCallosityState::CCallosityState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\callositystate.cpp:15
// RVA: 0x001F4580
// ADDRESS: 005f4580
// PROTOTYPE: undefined __thiscall CCallosityState(ushort param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCallosityState::CCallosityState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\callositystate.cpp:25
// RVA: 0x001F4600
// ADDRESS: 005f4600
// PROTOTYPE: undefined __thiscall CCallosityState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCallosityState::~CCallosityState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\callositystate.cpp:35
// RVA: 0x001F4670
// ADDRESS: 005f4670
// PROTOTYPE: void __thiscall ~CCallosityState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCallosityState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\callositystate.cpp:98
// RVA: 0x001F4680
// ADDRESS: 005f4680
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCallosityState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\callositystate.cpp:117
// RVA: 0x001F4740
// ADDRESS: 005f4740
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCallosityState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\callositystate.cpp:82
// RVA: 0x001F4830
// ADDRESS: 005f4830
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCallosityStateVisualEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\callositystate.cpp:195
// RVA: 0x001F4920
// ADDRESS: 005f4920
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: GameServer
