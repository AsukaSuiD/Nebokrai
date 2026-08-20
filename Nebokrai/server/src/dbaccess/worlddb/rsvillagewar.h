#pragma once

#include "rsregion.h"
#include "../../world/appworld/organizingsystem/villagewarsys.h"

/* Исходный владелец: dbaccess/worlddb/rsvillagewar.cpp/.h. DB-owner хранит
 * результат village war; календарь и смена фаз принадлежат CVillageWarSys. */
class CRsVillageWar {
public:
    [[nodiscard]] static bool SaveResult(IWorldDbExecutor&,
                                         const VillageWarResult&,
                                         const RegionDbSnapshot& currentRegion);
};
