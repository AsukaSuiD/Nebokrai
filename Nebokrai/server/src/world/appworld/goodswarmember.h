#pragma once

#include <cstddef>
#include <cstdint>
#include <map>
#include <set>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/goodswarmember.cpp/.h. Декомпилят
 * подтверждает member->faction map, ordered faction-ID set и счётчик побед.
 * Старый intrusive list заменён map; DB и message callbacks остаются снаружи.
 */
struct GoodsWarFactionCount { std::int32_t factionId{}, wins{}; };
class CGoodsWarMember {
public:
    bool AddMember(std::int32_t playerId, std::int32_t factionId);
    bool RemoveMember(std::int32_t playerId);
    std::size_t RemoveFaction(std::int32_t factionId);
    bool AddFaction(std::int32_t factionId);
    bool FactionWin(std::int32_t factionId);
    [[nodiscard]] bool ContainsFaction(std::int32_t factionId) const noexcept { return m_Factions.contains(factionId); }
    [[nodiscard]] std::vector<GoodsWarFactionCount> Counts() const;
private:
    std::map<std::int32_t,std::int32_t> m_Members;
    std::map<std::int32_t,std::int32_t> m_Counts;
    std::set<std::int32_t> m_Factions;
};
