//! Владелец навыка `CBossFiendSummon` (`0x1F9`) для объектного пути монстра.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/bossfiendsummon.cpp`. Задержки, блокировка движения,
//! пакеты `0xBFE01` и создание существ проходят общий узкий механизм семейства
//! призыва. Отличие владельца — один исходный `random(3)` на всё завершённое
//! применение и неизменная разновидность `30003/30004/30005` для всех
//! созданных им существ. Координатные перегрузки `Begin` ниже остаются RAW до
//! появления вызывающего пути.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendsummon.cpp

// ============================================================================
// FUNCTION: CBossFiendSummon::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendsummon.cpp:77
// RVA: 0x0012C390
// ADDRESS: 0052c390
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossFiendSummon::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendsummon.cpp:93
// RVA: 0x0012C460
// ADDRESS: 0052c460
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

pub(crate) const BOSS_FIEND_SUMMON_SKILL_ID: u32 = 0x1f9;

pub(crate) const fn summoned_creature_usage(random_value: i32) -> u32 {
    match random_value {
        1 => 30_004,
        2 => 30_005,
        _ => 30_003,
    }
}
