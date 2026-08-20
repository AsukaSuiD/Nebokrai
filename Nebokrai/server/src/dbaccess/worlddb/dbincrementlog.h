#pragma once

#include "rssetup.h"
#include "../../world/appworld/incrementlog/incrementlog.h"

#include <cstdint>
#include <vector>

/*
 * Исходный владелец: dbaccess/worlddb/dbincrementlog.cpp/.h.
 * SQL-выборка по возрасту и сортировка player/time подтверждены PDB/Rust.
 * Ограничение страницы из 17 записей остаётся в CIncrementLog, не в БД.
 */
class CDBIncrementLog {
public:
    [[nodiscard]] static WorldDbResult LoadRecent(IWorldDbExecutor&, std::int32_t days);
    [[nodiscard]] static bool Append(IWorldDbExecutor&,
                                     std::int32_t playerId,
                                     const IncrementLogEntry&);
};
