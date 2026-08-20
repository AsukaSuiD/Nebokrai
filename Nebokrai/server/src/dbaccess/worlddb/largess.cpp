#include "largess.h"

WorldDbResult CLargess::LoadPending(IWorldDbExecutor& database, const std::int32_t worldId)
{
    return database.Execute({
        "SELECT * FROM Largess WHERE WorldID=@P1 AND IsProcessed=0 ORDER BY SendID",
        {static_cast<std::int64_t>(worldId)}});
}

bool CLargess::Insert(IWorldDbExecutor& database, const LargessDbRecord& row)
{
    return database.Execute({
        "INSERT INTO Largess(SendID,Cdkey,PlayerId,GoodsIndex,GoodsName,GoodsLevel,SendNum,WorldID,SendTime,ObtainedNum) "
        "VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10)",
        {static_cast<std::int64_t>(row.sendId), row.cdKey, static_cast<std::int64_t>(row.playerId),
         static_cast<std::uint64_t>(row.goodsIndex), row.goodsName,
         static_cast<std::int64_t>(row.goodsLevel), static_cast<std::int64_t>(row.sendNumber),
         static_cast<std::int64_t>(row.worldId), row.sendTime,
         static_cast<std::int64_t>(row.obtainedNumber)}}).success;
}

bool CLargess::UpdateObtained(IWorldDbExecutor& database,
                              const std::int32_t sendId,
                              const std::int32_t playerId,
                              const std::int32_t obtained)
{
    return database.Execute({
        "UPDATE Largess SET ObtainedNum=@P1 WHERE SendID=@P2 AND PlayerId=@P3",
        {static_cast<std::int64_t>(obtained), static_cast<std::int64_t>(sendId),
         static_cast<std::int64_t>(playerId)}}).success;
}

bool CLargess::WriteResult(IWorldDbExecutor& database,
                           const LargessDbRecord& row,
                           std::string result,
                           const bool success)
{
    return database.Execute({
        "INSERT INTO LoadDetails(SendID,PlayerId,ObtainedNum,LoadTime,Failedreason,SaveTime) "
        "VALUES(@P1,@P2,@P3,GETDATE(),@P4,GETDATE())",
        {static_cast<std::int64_t>(row.sendId), static_cast<std::int64_t>(row.playerId),
         static_cast<std::int64_t>(row.obtainedNumber), success ? std::string{} : std::move(result)}}).success;
}
