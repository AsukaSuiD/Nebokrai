#pragma once

#include "rsregion.h"
#include "../../world/appworld/organizingsystem/attackcitysys.h"

/*
 * Исходный владелец: dbaccess/worlddb/rscitywar.cpp/.h. Owner сохраняет
 * конечный победивший faction/union для city-region; расписание остаётся в
 * CAttackCitySys и не дублируется в DB-обвязке.
 */
class CRsCityWar {
public:
    [[nodiscard]] static bool SaveResult(IWorldDbExecutor&,
                                         const AttackCityResult&,
                                         const RegionDbSnapshot& currentRegion);
};
