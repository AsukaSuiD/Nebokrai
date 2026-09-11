//! CSpiderPoisonState (0x191), GameServer.exe/GameServer.pdb,
//! исходный owner skills/spiderpoisonstate.cpp. Ctor 0x005E90C0 задаёт start/count0;
//! Object Begin 0x005E9430 и AI 0x005E96B0 используют общий states/poison.rs.
//! CalculateAttackPower 0x005E9610 передаёт signed HP-loss без clamp,
//! в отличие от PoisonArrow. Сохраняются defaults skill-id/level1,
//! type400-only team/guild/union/PK, actual Sufferer и отдельные чтения часов.
//! Vtable 0x0065F9F4: property=true, S-only SetRegion; End без state.ended,
//! чистый Save56 и Load Master40→clock→поля реализованы общим механизмом.
//! Два недостигнутых overload Begin остаются ниже.

use super::spiderpoison::SPIDER_POISON_SKILL_ID;
pub(crate) use crate::gameserver::appserver::states::poison::{
    POISON_STATE_BYTES as SPIDER_POISON_STATE_BYTES,
    begin_primary_poison_state as begin_primary_spider_poison_state,
};
pub(crate) type SpiderPoisonState =
    crate::gameserver::appserver::states::poison::PoisonState<SPIDER_POISON_SKILL_ID>;


// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoisonstate.cpp

// ============================================================================
// FUNCTION: CSpiderPoisonState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoisonstate.cpp:69
// RVA: 0x001E9270
// ADDRESS: 005e9270
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoisonState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoisonstate.cpp:83
// RVA: 0x001E9310
// ADDRESS: 005e9310
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



// COMPONENT_VARIANT_END: GameServer
