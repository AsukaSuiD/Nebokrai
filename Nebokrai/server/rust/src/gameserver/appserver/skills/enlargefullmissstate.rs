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
// Сохранены только не подключённые конструктор по умолчанию и сериализация.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargefullmissstate.h

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
