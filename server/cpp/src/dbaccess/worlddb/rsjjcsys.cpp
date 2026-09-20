#include "rsjjcsys.h"

WorldDbResult CRsJjcSys::Load(IWorldDbExecutor& database, const std::int32_t playerId)
{
    return database.Execute({"SELECT * FROM csl_player_jjc WHERE id=@P1",
                             {static_cast<std::int64_t>(playerId)}});
}

bool CRsJjcSys::Save(IWorldDbExecutor& database, const JjcDbData& data)
{
    return database.Execute({
        "EXEC sp_JJccUpdatePlayer @id=@P1,@jjcLevel=@P2,@jjcScore=@P3,@weekJoin=@P4,"
        "@weekWin=@P5,@weekLose=@P6,@weekTie=@P7,@seasonJoin=@P8,@seasonWin=@P9,"
        "@seasonLose=@P10,@seasonTie=@P11",
        {static_cast<std::int64_t>(data.playerId), static_cast<std::int64_t>(data.level),
         static_cast<std::int64_t>(data.score), static_cast<std::int64_t>(data.weekJoin),
         static_cast<std::int64_t>(data.weekWin), static_cast<std::int64_t>(data.weekLose),
         static_cast<std::int64_t>(data.weekTie), static_cast<std::int64_t>(data.seasonJoin),
         static_cast<std::int64_t>(data.seasonWin), static_cast<std::int64_t>(data.seasonLose),
         static_cast<std::int64_t>(data.seasonTie)}}).success;
}

bool CRsJjcSys::RunConfirmedReset(IWorldDbExecutor& database, const std::string_view procedure)
{
    if (procedure.empty() || procedure.find_first_not_of(
            "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789_[]") != std::string_view::npos) {
        return false;
    }
    return database.Execute({"EXEC " + std::string(procedure), {}}).success;
}
