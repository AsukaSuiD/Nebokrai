#pragma once

#include "../organizingsystem/organizing.h"

#include <cstdint>
#include <map>
#include <mutex>
#include <string>
#include <string_view>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/incrementlog/incrementlog.cpp/.h.
 * EXE подтверждает player-keyed порядок, страницу из 17 записей, обратную
 * выдачу внутри страницы и wire time/type/money/description. DB persistence
 * остаётся отдельным worlddb owner; mutex сохраняет общий concurrent registry.
 */
struct IncrementLogEntry {
    OrganizingTime time{};
    std::uint8_t type{};
    std::int32_t money{};
    std::string description;
};
struct IncrementLogPage { std::int32_t pageCount{}; std::vector<IncrementLogEntry> entries; };

class CIncrementLog {
public:
    bool Add(std::int32_t playerId, IncrementLogEntry entry);
    [[nodiscard]] std::size_t Size(std::int32_t playerId) const;
    [[nodiscard]] IncrementLogPage Page(std::int32_t playerId, std::int32_t page) const;
    [[nodiscard]] bool SerializePage(std::vector<std::uint8_t>& output,
                                     std::int32_t playerId,
                                     std::int32_t page) const;
private:
    mutable std::mutex m_Mutex;
    std::map<std::int32_t, std::vector<IncrementLogEntry>> m_Entries;
};
