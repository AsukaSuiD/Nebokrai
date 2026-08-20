#include "faction.h"

#include <algorithm>
#include <cstring>
#include <limits>

namespace {
template <class T> void Append(std::vector<std::uint8_t>& out, const T value)
{
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
    out.insert(out.end(), bytes, bytes + sizeof(value));
}
}

CFaction::CFaction(const std::int32_t id,
                   const std::int32_t masterId,
                   const OrganizingTime established,
                   std::string name)
    : m_Id(id), m_Name(std::move(name)), m_MasterId(masterId), m_EstablishedTime(established)
{
}

std::int32_t CFaction::IsMember(const std::int32_t playerId) const noexcept
{
    return m_Members.contains(playerId) ? m_Id : 0;
}

std::string CFaction::GetMemberTitle(const std::int32_t playerId) const
{
    const auto member = m_Members.find(playerId);
    return member == m_Members.end() ? std::string{} : std::string(member->second.Title());
}

bool CFaction::IsContributeMember(const std::int32_t playerId) const noexcept
{
    const auto member = m_Members.find(playerId);
    return member != m_Members.end() && member->second.contribute;
}

bool CFaction::AddMember(OrganizingMemberInfo member)
{
    const auto [position, inserted] = m_Members.emplace(member.id, std::move(member));
    if (!inserted) return false;
    m_Property.memberCount = static_cast<std::int32_t>(m_Members.size());
    SetChangeData(2);
    return true;
}

bool CFaction::RemoveMember(const std::int32_t playerId)
{
    if (m_Members.erase(playerId) == 0) return false;
    m_Property.memberCount = static_cast<std::int32_t>(m_Members.size());
    SetChangeData(2);
    return true;
}

bool CFaction::SetMemberLevel(const std::int32_t playerId, const std::int32_t level) noexcept
{
    const auto member = m_Members.find(playerId);
    if (member == m_Members.end()) return false;
    member->second.level = level;
    SetChangeData(2);
    return true;
}

bool CFaction::SetMemberRegion(const std::int32_t playerId, const std::string_view region) noexcept
{
    const auto member = m_Members.find(playerId);
    if (member == m_Members.end() || !member->second.SetRegion(region)) return false;
    SetChangeData(2);
    return true;
}

bool CFaction::SetMemberRight(const std::int32_t playerId,
                              const std::size_t index,
                              const EPurviewOwnState state) noexcept
{
    const auto member = m_Members.find(playerId);
    if (member == m_Members.end() || index >= member->second.purview.size()) return false;
    member->second.purview[index] = state;
    SetChangeData(2);
    return true;
}

bool CFaction::AddOwnedCity(const std::int32_t regionId)
{
    if (std::ranges::find(m_OwnedCities, regionId) != m_OwnedCities.end()) return false;
    m_OwnedCities.push_back(regionId);
    SetChangeData(4);
    return true;
}

bool CFaction::RemoveOwnedCity(const std::int32_t regionId)
{
    const auto position = std::ranges::find(m_OwnedCities, regionId);
    if (position == m_OwnedCities.end()) return false;
    m_OwnedCities.erase(position);
    SetChangeData(4);
    return true;
}

void CFaction::SetChangeData(const std::int32_t mask) noexcept
{
    if (mask == 0) m_ChangeMask = 0;
    else m_ChangeMask |= mask;
}

std::optional<FactionSaveSnapshot> CFaction::CloneSaveData() const
{
    if (m_ChangeMask == 0) return std::nullopt;
    return FactionSaveSnapshot{m_Id, m_Name, m_MasterId, m_Members, m_Property,
        m_EstablishedTime, m_OwnedCities, m_EnemyFactions, m_CityWarEnemyFactions,
        m_DeleteRemainTime, m_ChangeMask};
}

bool CFaction::SerializeMembers(std::vector<std::uint8_t>& output) const
{
    if (m_Members.size() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max())) return false;
    Append(output, static_cast<std::int32_t>(m_Members.size()));
    for (const auto& [id, member] : m_Members) {
        (void)id;
        if (!member.Serialize(output)) return false;
    }
    return true;
}
