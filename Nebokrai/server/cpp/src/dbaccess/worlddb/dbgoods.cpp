#include "dbgoods.h"

WorldDbResult CDBGoods::Load(IWorldDbExecutor& database, const std::int32_t playerId)
{
    return database.Execute({
        "SELECT g.*,e.type,e.modifierValue1,e.modifierValue2 "
        "FROM player_goods g LEFT JOIN extend_properties e ON e.id=g.id "
        "WHERE g.playerID=@P1 ORDER BY g.place,g.position,g.id",
        {static_cast<std::int64_t>(playerId)}});
}

bool CDBGoods::Replace(IWorldDbExecutor& database,
                       const std::int32_t playerId,
                       const std::vector<GoodsDbRow>& goods,
                       const std::vector<GoodsAddonDbRow>& addons)
{
    if (!database.Execute({
            "DELETE e FROM extend_properties e INNER JOIN player_goods g ON g.id=e.id "
            "WHERE g.playerID=@P1",
            {static_cast<std::int64_t>(playerId)}}).success ||
        !database.Execute({"DELETE FROM player_goods WHERE playerID=@P1",
                           {static_cast<std::int64_t>(playerId)}}).success) {
        return false;
    }

    for (const GoodsDbRow& row : goods) {
        if (!database.Execute({
                "INSERT INTO player_goods(id,GoodsID,goodsIndex,playerID,name,price,amount,place,position) "
                "VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9)",
                {row.instanceId, static_cast<std::uint64_t>(row.goodsId),
                 static_cast<std::uint64_t>(row.goodsIndex), static_cast<std::int64_t>(row.playerId),
                 row.name, static_cast<std::uint64_t>(row.price), static_cast<std::uint64_t>(row.amount),
                 static_cast<std::int64_t>(row.place), static_cast<std::int64_t>(row.position)}}).success) {
            return false;
        }
    }

    for (const GoodsAddonDbRow& row : addons) {
        if (!database.Execute({
                "INSERT INTO extend_properties(type,modifierValue1,modifierValue2,id) "
                "VALUES(@P1,@P2,@P3,@P4)",
                {static_cast<std::int64_t>(row.type), static_cast<std::int64_t>(row.modifierValue1),
                 static_cast<std::int64_t>(row.modifierValue2), row.goodsInstanceId}}).success) {
            return false;
        }
    }
    return true;
}
