//! CPoisonArrowState (0x21E), GameServer.exe/GameServer.pdb,
//! исходный owner skills/poisonarrowstate.cpp. Ctor 0x005E3140 задаёт start/count0;
//! Object Begin 0x005E3460 и AI 0x005E3730 используют общий states/poison.rs.
//! CalculateAttackPower 0x005E3690 обнуляет отрицательный signed HP-loss:
//! это явное отличие от SpiderPoison. Сохраняются defaults skill-id/level1,
//! type400-only team/guild/union/PK, actual Sufferer и отдельные чтения часов.
//! Vtable 0x0065F24C: property=true, S-only SetRegion; End без state.ended,
//! чистый Save56 и Load Master40→clock→поля реализованы общим механизмом.
//! Два недостигнутых overload Begin остаются ниже.

use super::poisonarrow::POISON_ARROW_SKILL_ID;
pub(crate) use crate::gameserver::appserver::states::poison::{
    POISON_STATE_BYTES as POISON_ARROW_STATE_BYTES,
    begin_primary_poison_state as begin_primary_poison_arrow_state,
};
pub(crate) type PoisonArrowState =
    crate::gameserver::appserver::states::poison::PoisonState<POISON_ARROW_SKILL_ID>;

// ============================================================================
// FUNCTION: CPoisonArrowState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\poisonarrowstate.cpp:68
// RVA: 0x001E32F0
// ADDRESS: 005e32f0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPoisonArrowState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\poisonarrowstate.cpp:82
// RVA: 0x001E3390
// ADDRESS: 005e3390
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
