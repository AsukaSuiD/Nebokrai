//! Параметры немедленного навыка `CEnlargeMaxMp`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает навык `602`:
//! он проверяет только наличие свойств, всегда выбирает владельца, заменяет
//! только прежнее состояние `602`, публикует `OnChangeStates` и завершает
//! базовый навык одним снятием запрета движения.

pub(crate) const ENLARGE_MAX_MP_SKILL_ID: u32 = 602;
pub(crate) const SKILL_USAGE_MAX_MP_GAIN: u32 = 115;

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сырой C++ ниже является комментарием, а не Rust-реализацией.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargemaxmp.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargemaxmp.h

// ============================================================================
// FUNCTION: CEnlargeMaxMp::CEnlargeMaxMp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargemaxmp.cpp:18
// RVA: 0x001168B0
// ADDRESS: 005168b0
// PROTOTYPE: undefined __thiscall CEnlargeMaxMp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnlargeMaxMp::~CEnlargeMaxMp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargemaxmp.cpp:25
// RVA: 0x00116920
// ADDRESS: 00516920
// PROTOTYPE: void __thiscall ~CEnlargeMaxMp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnlargeMaxMp::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\enlargemaxmp.cpp:99
// RVA: 0x001169B0
// ADDRESS: 005169b0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



























// COMPONENT_VARIANT_END: GameServer
