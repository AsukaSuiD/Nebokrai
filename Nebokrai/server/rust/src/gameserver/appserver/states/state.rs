//! Достигнутая базовая client-проекция `CState` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходные owners
//! `appserver/states/state.h/.cpp`. Vtable-аудит exact EXE подтверждает, что
//! базовые `GetClientStateTime` и `GetAdditionalData` сведены линкером в одну
//! функцию RVA `0x00201200`, возвращающую ноль. Конкретные state-классы могут
//! переопределять каждый getter; поэтому здесь закреплены только базовые
//! значения, а динамический remaining-time и additional-data остаются у
//! конкретных владельцев. Остальной корпус сохранён ниже как `UNKNOWN` (исследовательский декомпилят хранится локально).

/// Exact базовый `CState::GetClientStateTime` для классов без override-а.
pub(crate) const fn default_client_state_time() -> i32 {
    0
}

/// Exact базовый `CState::GetAdditionalData` для классов без override-а.
pub(crate) const fn default_additional_data() -> u32 {
    0
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.h

// ============================================================================
// FUNCTION: CState::IsEnded
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.h:61
// RVA: 0x000D8380
// ADDRESS: 004d8380
// PROTOTYPE: int __thiscall IsEnded(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:21
// RVA: 0x001DBC90
// ADDRESS: 005dbc90
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::CState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:29
// RVA: 0x001DBCA0
// ADDRESS: 005dbca0
// PROTOTYPE: undefined __thiscall CState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::End
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:149
// RVA: 0x001DBCE0
// ADDRESS: 005dbce0
// PROTOTYPE: void __thiscall End(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::Serialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:247
// RVA: 0x001DBD00
// ADDRESS: 005dbd00
// PROTOTYPE: void __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::~CState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:48
// RVA: 0x001DBD40
// ADDRESS: 005dbd40
// PROTOTYPE: void __thiscall ~CState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:175
// RVA: 0x001DBD70
// ADDRESS: 005dbd70
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:208
// RVA: 0x001DBDD0
// ADDRESS: 005dbdd0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:227
// RVA: 0x001DBE20
// ADDRESS: 005dbe20
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:256
// RVA: 0x001DBE70
// ADDRESS: 005dbe70
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::GetUser
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:55
// RVA: 0x001DBEF0
// ADDRESS: 005dbef0
// PROTOTYPE: CMoveShape * __thiscall GetUser(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CState::GetSufferer
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\states\state.cpp:80
// RVA: 0x001DBFD0
// ADDRESS: 005dbfd0
// PROTOTYPE: CMoveShape * __thiscall GetSufferer(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
