#include "union.h"

#include <limits>
#include <utility>

namespace {
template <class T> void Append(std::vector<std::uint8_t>& out, const T value)
{
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
    out.insert(out.end(), bytes, bytes + sizeof(value));
}
}

CUnion::CUnion(const std::int32_t id, const std::int32_t masterId, std::string name)
    : m_Id(id), m_Name(std::move(name)), m_MasterId(masterId)
{
}

std::int32_t CUnion::IsMember(const std::int32_t factionId) const noexcept
{
    return m_Members.contains(factionId) ? m_Id : 0;
}

bool CUnion::AddFaction(OrganizingMemberInfo faction)
{
    const auto [position, inserted] = m_Members.emplace(faction.id, std::move(faction));
    if (inserted) SetChangeData(2);
    return inserted;
}

bool CUnion::RemoveFaction(const std::int32_t factionId)
{
    if (m_Members.erase(factionId) == 0) return false;
    SetChangeData(2);
    return true;
}

void CUnion::SetChangeData(const std::int32_t mask) noexcept
{
    if (mask == 0) m_ChangeMask = 0;
    else m_ChangeMask |= mask;
}

std::optional<UnionSaveSnapshot> CUnion::CloneSaveData() const
{
    if (m_ChangeMask == 0) return std::nullopt;
    return UnionSaveSnapshot{m_Id, m_Name, m_MasterId, m_Members, m_EstablishedTime, m_ChangeMask};
}

bool CUnion::SerializeMembers(std::vector<std::uint8_t>& output) const
{
    if (m_Members.size() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max())) return false;
    Append(output, static_cast<std::int32_t>(m_Members.size()));
    for (const auto& [id, member] : m_Members) {
        (void)id;
        if (!member.Serialize(output)) return false;
    }
    return true;
}
