//! Параметры немедленного навыка `CEnlargeMaxHp`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает навык `601`:
//! он проверяет только наличие свойств, всегда выбирает владельца, заменяет
//! только прежнее состояние `601`, публикует `OnChangeStates` и завершает
//! базовый навык одним снятием запрета движения.

pub(crate) const ENLARGE_MAX_HP_SKILL_ID: u32 = 601;
pub(crate) const SKILL_USAGE_MAX_HP_GAIN: u32 = 118;

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сырой C++ ниже является комментарием, а не Rust-реализацией.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargemaxhp.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargemaxhp.h

// ============================================================================
// FUNCTION: CEnlargeMaxHp::CEnlargeMaxHp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargemaxhp.cpp:18
// RVA: 0x00116B40
// ADDRESS: 00516b40
// PROTOTYPE: undefined __thiscall CEnlargeMaxHp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnlargeMaxHp::~CEnlargeMaxHp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargemaxhp.cpp:25
// RVA: 0x00116BB0
// ADDRESS: 00516bb0
// PROTOTYPE: void __thiscall ~CEnlargeMaxHp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnlargeMaxHp::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargemaxhp.cpp:99
// RVA: 0x00116BF0
// ADDRESS: 00516bf0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
