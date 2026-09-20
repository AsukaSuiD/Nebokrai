#include "rsgodsbattle.h"

WorldDbResult CRsGodsBattle::LoadRegions(IWorldDbExecutor& database)
{
    return database.Execute({"SELECT * FROM CSL_GODSBATTLE", {}});
}

WorldDbResult CRsGodsBattle::LoadNpcs(IWorldDbExecutor& database)
{
    return database.Execute({"SELECT NPC_NAME,Faciton FROM CSL_GODSBATTLE_NPC", {}});
}

bool CRsGodsBattle::ReplaceRegions(IWorldDbExecutor& database,
                                   const std::vector<GodsBattleRegionDbRow>& rows)
{
    if (!database.Execute({"DELETE FROM CSL_GODSBATTLE", {}}).success) return false;
    for (const auto& row : rows) {
        if (!database.Execute({
                "INSERT INTO CSL_GODSBATTLE(RegionID,AFactionXYD,BFactionXYD) VALUES(@P1,@P2,@P3)",
                {static_cast<std::int64_t>(row.regionId), static_cast<std::int64_t>(row.factionA),
                 static_cast<std::int64_t>(row.factionB)}}).success) return false;
    }
    return true;
}

bool CRsGodsBattle::ReplaceNpcs(IWorldDbExecutor& database,
                                const std::vector<GodsBattleNpcDbRow>& rows)
{
    if (!database.Execute({"DELETE FROM CSL_GODSBATTLE_NPC", {}}).success) return false;
    for (const auto& row : rows) {
        if (!database.Execute({
                "INSERT INTO CSL_GODSBATTLE_NPC(NPC_NAME,Faciton) VALUES(@P1,@P2)",
                {row.npcName, static_cast<std::int64_t>(row.faction)}}).success) return false;
    }
    return true;
}

WorldDbResult CRsGodsBattle::LoadTopPlayers(IWorldDbExecutor& database, const std::int32_t faction)
{
    return database.Execute({
        "SELECT TOP 10 Name,SZL,Levels FROM CSL_PLAYER_ABILITY "
        "WHERE GodsBattleFaction=@P1 ORDER BY SZL DESC",
        {static_cast<std::int64_t>(faction)}});
}
