#include "rsvillagewar.h"

bool CRsVillageWar::SaveResult(IWorldDbExecutor& database,
                               const VillageWarResult& result,
                               const RegionDbSnapshot& currentRegion)
{
    if (result.regionId != currentRegion.regionId) return false;
    RegionDbSnapshot updated = currentRegion;
    updated.factionId = result.factionId;
    updated.unionId = 0;
    return CRsRegion::Save(database, updated).success;
}
