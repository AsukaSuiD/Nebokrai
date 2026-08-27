//! Немедленное исполнение `CTaiJi`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `taiji.cpp` и `taiji.h`. `AI` выбирает владельца навыка раньше заданной
//! цели, создаёт новое состояние до поиска прежнего `0x12d`, заменяет только
//! это состояние, публикует текущее состояние владельца и завершает навык с часами
//! восстановления. Навык не ставит запрет движения и не создаёт отдельный
//! визуальный пакет, однако базовое завершение один раз снимает запрет.

pub(crate) const TAIJI_SKILL_ID: u32 = 301;
pub(crate) const SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN: u32 = 112;

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сырой C++ ниже является комментарием, а не Rust-реализацией.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\taiji.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\taiji.h

// ============================================================================
// FUNCTION: CTaiJi::CTaiJi
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\taiji.cpp:17
// RVA: 0x001AF6C0
// ADDRESS: 005af6c0
// PROTOTYPE: undefined __thiscall CTaiJi(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaiJi::~CTaiJi
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\taiji.cpp:24
// RVA: 0x001AF730
// ADDRESS: 005af730
// PROTOTYPE: void __thiscall ~CTaiJi(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CTaiJi::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\taiji.cpp:99
// RVA: 0x001AF770
// ADDRESS: 005af770
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//






















// COMPONENT_VARIANT_END: GameServer
