//! Каноническая достигнутая часть `CEnlargeFullMissState`.
//!
//! Для игрока состояние `603` прибавляет младшие 16 бит знакового параметра
//! к `full_miss` точным WORD-сложением с переполнением. Собственного
//! визуального сообщения и таймера нет.

use super::enlargefullmiss::ENLARGE_FULL_MISS_SKILL_ID;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EnlargeFullMissState {
    gain: i32,
}

impl EnlargeFullMissState {
    pub(crate) const fn new(gain: i32) -> Self { Self { gain } }
    pub(crate) const fn skill_id(self) -> u32 { ENLARGE_FULL_MISS_SKILL_ID }
    pub(crate) const fn apply(self, value: u16) -> u16 {
        value.wrapping_add(self.gain as u16)
    }
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сырой C++ ниже является комментарием, а не Rust-реализацией.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.h

// ============================================================================
// FUNCTION: CEnlargeFullMissState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.cpp:59
// RVA: 0x001DFD40
// ADDRESS: 005dfd40
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnlargeFullMissState::CEnlargeFullMissState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.cpp:15
// RVA: 0x001E2010
// ADDRESS: 005e2010
// PROTOTYPE: undefined __thiscall CEnlargeFullMissState(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnlargeFullMissState::CEnlargeFullMissState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.cpp:24
// RVA: 0x001E2080
// ADDRESS: 005e2080
// PROTOTYPE: undefined __thiscall CEnlargeFullMissState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnlargeFullMissState::~CEnlargeFullMissState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.cpp:33
// RVA: 0x001E20F0
// ADDRESS: 005e20f0
// PROTOTYPE: void __thiscall ~CEnlargeFullMissState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnlargeFullMissState::OnUpdateProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.cpp:37
// RVA: 0x001E2120
// ADDRESS: 005e2120
// PROTOTYPE: int __thiscall OnUpdateProperties(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnlargeFullMissState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.cpp:65
// RVA: 0x001E2250
// ADDRESS: 005e2250
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnlargeFullMissState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.cpp:82
// RVA: 0x001E23D0
// ADDRESS: 005e23d0
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnlargeFullMissState::End
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.cpp:71
// RVA: 0x001ECFC0
// ADDRESS: 005ecfc0
// PROTOTYPE: void __thiscall End(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnlargeFullMissState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.cpp:53
// RVA: 0x00201290
// ADDRESS: 00601290
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnlargeFullMissState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.cpp:92
// RVA: 0x00201350
// ADDRESS: 00601350
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
