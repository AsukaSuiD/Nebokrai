#pragma once

#include "rssetup.h"

#include <cstdint>

/*
 * Исходный владелец: dbaccess/worlddb/rsjjcsys.cpp/.h.
 * Одиннадцать аргументов sp_JJccUpdatePlayer подтверждены EXE/Rust. Reset-
 * процедуры не называются наугад: process-owner передаёт подтверждённое имя,
 * когда оно загружено из server setup.
 */
struct JjcDbData {
    std::int32_t playerId{};
    std::int32_t level{};
    std::int32_t score{};
    std::int32_t weekJoin{}, weekWin{}, weekLose{}, weekTie{};
    std::int32_t seasonJoin{}, seasonWin{}, seasonLose{}, seasonTie{};
};

class CRsJjcSys {
public:
    [[nodiscard]] static WorldDbResult Load(IWorldDbExecutor&, std::int32_t playerId);
    [[nodiscard]] static bool Save(IWorldDbExecutor&, const JjcDbData&);
    [[nodiscard]] static bool RunConfirmedReset(IWorldDbExecutor&, std::string_view procedure);
};
