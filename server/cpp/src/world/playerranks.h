#pragma once

#include <cstdint>
#include <string>
#include <vector>

/*
 * Исходный владелец: WorldServer/worldserver/playerranks.cpp/.h. EXE
 * подтверждает ordered rank-list и wire player/name/occupation/level/faction.
 * DB запрос и суточное планирование принадлежат process-owner; этот класс
 * атомарно публикует готовый снимок.
 */
struct PlayerRank { std::int32_t playerId{}; std::string name; std::uint16_t occupation{},level{}; std::string factionName; };
class CPlayerRanks {
public:
    explicit CPlayerRanks(std::size_t maximum=0) noexcept:m_Maximum(maximum){}
    bool Add(PlayerRank rank);
    void Replace(std::vector<PlayerRank> ranks);
    [[nodiscard]] const std::vector<PlayerRank>& Ranks() const noexcept{return m_Ranks;}
    [[nodiscard]] bool Serialize(std::vector<std::uint8_t>& output) const;
private:
    std::size_t m_Maximum{};
    std::vector<PlayerRank> m_Ranks;
};
