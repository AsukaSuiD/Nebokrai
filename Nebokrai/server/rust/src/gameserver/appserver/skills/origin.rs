//! Параметры достигнутого player-пути `COrigin`.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает навык `304`:
//! он выбирает владельца раньше sufferer-а, создаёт состояние из usage `115`,
//! заменяет только прежнее состояние `304`, публикует `OnChangeStates` и
//! завершает базовый навык с восстановлением движения. Monster-ветвь теперь
//! использует тот же `OriginState` из реального monster skill caller-а.

pub(crate) const ORIGIN_SKILL_ID: u32 = 304;
pub(crate) const SKILL_USAGE_ELEMENT_MODIFY_GAIN: u32 = 115;

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Смешанная player/monster-функция `AI` подключена обоими canonical owner-ами.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\origin.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\origin.h

// ============================================================================
// FUNCTION: COrigin::AI
// STATUS: IMPLEMENTED
// IMPLEMENTED: `execute_player_immediate_state` и
// `execute_monster_immediate_state`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\origin.cpp:99
// RVA: 0x001AFA70
// ADDRESS: 005afa70
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//











// COMPONENT_VARIANT_END: GameServer
