#include "factionwarsys.h"

#include <algorithm>

namespace {
bool SamePair(const EnemyFactionRelation& value, const std::int32_t first, const std::int32_t second)
{
    return (value.firstFaction == first && value.secondFaction == second) ||
           (value.firstFaction == second && value.secondFaction == first);
}
}

void CFactionWarSys::AddWarType(const FactionWarType type) { m_Types[type.type] = type; }

std::int32_t CFactionWarSys::GetDeclareMoney(const std::int32_t type) const noexcept
{
    const auto found = m_Types.find(type);
    return found == m_Types.end() ? 0 : found->second.money;
}

bool CFactionWarSys::IsEnemyRelation(const std::int32_t first, const std::int32_t second) const noexcept
{
    return std::ranges::any_of(m_Relations, [=](const auto& value) { return SamePair(value, first, second); });
}

void CFactionWarSys::AddEnemyRelation(const std::int32_t first,
                                      const std::int32_t second,
                                      const std::uint32_t duration)
{
    const auto relation = std::ranges::find_if(m_Relations, [=](const auto& value) { return SamePair(value, first, second); });
    if (relation != m_Relations.end()) relation->remainingMilliseconds = duration;
    else m_Relations.push_back({first, second, duration});
}

bool CFactionWarSys::RemoveEnemyRelation(const std::int32_t first, const std::int32_t second)
{
    const auto relation = std::ranges::find_if(m_Relations, [=](const auto& value) { return SamePair(value, first, second); });
    if (relation == m_Relations.end()) return false;
    m_Relations.erase(relation);
    return true;
}

std::vector<EnemyFactionRelation> CFactionWarSys::Run(const std::uint32_t elapsed)
{
    std::vector<EnemyFactionRelation> expired;
    for (auto position = m_Relations.begin(); position != m_Relations.end();) {
        if (position->remainingMilliseconds <= elapsed) {
            expired.push_back(*position);
            position = m_Relations.erase(position);
        } else {
            position->remainingMilliseconds -= elapsed;
            ++position;
        }
    }
    return expired;
}
