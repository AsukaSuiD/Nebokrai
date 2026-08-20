#pragma once
#include "rssetup.h"

/* Исходный владелец: dbaccess/worlddb/rsregion.cpp/.h. Шесть scalar полей
 * CSL_Region подтверждены Rust/EXE; player/region runtime здесь не хранится. */
struct RegionDbSnapshot{std::int32_t regionId{},factionId{},unionId{},taxRate{},todayTax{},totalTax{};};
class CRsRegion{public:static WorldDbResult Load(IWorldDbExecutor&);static WorldDbResult Save(IWorldDbExecutor&,const RegionDbSnapshot&);};
