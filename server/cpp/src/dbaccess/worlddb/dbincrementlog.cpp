#include "dbincrementlog.h"

WorldDbResult CDBIncrementLog::LoadRecent(IWorldDbExecutor& database, const std::int32_t days)
{
    return database.Execute({
        "SELECT id,type,money,description,log_time,player_id FROM increment_log "
        "WHERE DATEDIFF(day,log_time,GETDATE())<=@P1 ORDER BY player_id,log_time",
        {static_cast<std::int64_t>(days)}});
}

bool CDBIncrementLog::Append(IWorldDbExecutor& database,
                             const std::int32_t playerId,
                             const IncrementLogEntry& entry)
{
    return database.Execute({
        "INSERT INTO increment_log(type,money,description,log_time,player_id) "
        "VALUES(@P1,@P2,@P3,@P4,@P5)",
        {static_cast<std::int64_t>(entry.type), static_cast<std::int64_t>(entry.money),
         entry.description,
         std::string(std::to_string(entry.time.year) + "-" + std::to_string(entry.time.month) + "-" +
                     std::to_string(entry.time.day) + " " + std::to_string(entry.time.hour) + ":" +
                     std::to_string(entry.time.minute) + ":" + std::to_string(entry.time.second)),
         static_cast<std::int64_t>(playerId)}}).success;
}
