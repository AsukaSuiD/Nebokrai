//! Немедленное исполнение `CTaiJi`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `taiji.cpp` и `taiji.h`. `AI` выбирает владельца навыка раньше заданной
//! цели, создаёт новое состояние до поиска прежнего `0x12d`, заменяет только
//! это состояние, публикует текущее состояние владельца и завершает навык с часами
//! восстановления. Навык не ставит запрет движения и не создаёт отдельный
//! визуальный пакет, однако базовое завершение один раз снимает запрет.
//! Monster-ветвь использует тот же `TaiJiState` из реального monster skill
//! caller-а и завершает derived AI тем же completion action.

pub(crate) const TAIJI_SKILL_ID: u32 = 301;
pub(crate) const SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN: u32 = 112;

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Смешанная player/monster-функция `AI` подключена обоими canonical owner-ами.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\taiji.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\taiji.h

// ============================================================================
// FUNCTION: CTaiJi::AI
// STATUS: IMPLEMENTED
// IMPLEMENTED: `execute_player_immediate_state` и
// `execute_monster_immediate_state`.
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
