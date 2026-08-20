#include "rscitywar.h"

bool CRsCityWar::SaveResult(IWorldDbExecutor& database,
                            const AttackCityResult& result,
                            const RegionDbSnapshot& currentRegion)
{
    if (result.regionId != currentRegion.regionId) return false;
    RegionDbSnapshot updated = currentRegion;
    updated.factionId = result.factionId;
    updated.unionId = result.unionId;
    return CRsRegion::Save(database, updated).success;
}
