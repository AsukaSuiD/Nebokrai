#include "rsregion.h"

WorldDbResult CRsRegion::Load(IWorldDbExecutor& database)
{
    return database.Execute({"SELECT * FROM CSL_Region ORDER BY RegionID", {}});
}

WorldDbResult CRsRegion::Save(IWorldDbExecutor& database, const RegionDbSnapshot& s)
{
    return database.Execute({
        "IF EXISTS(SELECT 1 FROM CSL_Region WHERE RegionID=@P1) "
        "UPDATE TOP (1) CSL_Region SET OwnedFactionID=@P2,OwnedUnionID=@P3,CurTaxRate=@P4,"
        "TodayTotalTax=@P5,TotalTax=@P6 WHERE RegionID=@P1 "
        "ELSE INSERT INTO CSL_Region(RegionID,OwnedFactionID,OwnedUnionID,CurTaxRate,TodayTotalTax,TotalTax) "
        "VALUES(@P1,@P2,@P3,@P4,@P5,@P6)",
        {static_cast<std::int64_t>(s.regionId), static_cast<std::int64_t>(s.factionId),
         static_cast<std::int64_t>(s.unionId), static_cast<std::int64_t>(s.taxRate),
         static_cast<std::int64_t>(s.todayTax), static_cast<std::int64_t>(s.totalTax)}});
}
