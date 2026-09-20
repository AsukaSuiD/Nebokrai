#pragma once

#include "rssetup.h"

#include <cstdint>
#include <string>
#include <vector>

/*
 * Исходный владелец: dbaccess/worlddb/largess.cpp/.h.
 * Подтверждены незавершённые выдачи Largess, счётчик ObtainedNum и два журнала.
 * Небезопасный sprintf и char[256] заменены параметрами; порядок выдачи и
 * удаление элемента только после успешного UPDATE сохраняет process-owner.
 */
struct LargessDbRecord {
    std::int32_t sendId{};
    std::string cdKey;
    std::int32_t playerId{};
    std::uint32_t goodsIndex{};
    std::string goodsName;
    std::int32_t goodsLevel{};
    std::int32_t sendNumber{};
    std::int32_t worldId{};
    std::string sendTime;
    std::int32_t obtainedNumber{};
};

class CLargess {
public:
    [[nodiscard]] static WorldDbResult LoadPending(IWorldDbExecutor&, std::int32_t worldId);
    [[nodiscard]] static bool Insert(IWorldDbExecutor&, const LargessDbRecord&);
    [[nodiscard]] static bool UpdateObtained(IWorldDbExecutor&, std::int32_t sendId,
                                             std::int32_t playerId, std::int32_t obtained);
    [[nodiscard]] static bool WriteResult(IWorldDbExecutor&, const LargessDbRecord&,
                                          std::string result, bool success);
};
