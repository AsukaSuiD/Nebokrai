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
    begin_primary_poison_state as begin_primary_spider_poison_state,
};
pub(crate) type SpiderPoisonState =
    crate::gameserver::appserver::states::poison::PoisonState<SPIDER_POISON_SKILL_ID>;
