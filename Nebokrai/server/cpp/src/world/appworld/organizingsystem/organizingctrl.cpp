#include "organizingctrl.h"

#include <algorithm>
#include <utility>

COrganizingCtrl::COrganizingCtrl(Clock clock) : m_Clock(std::move(clock)) {}

bool COrganizingCtrl::AddFaction(std::unique_ptr<CFaction> faction)
{
    if (!faction) return false;
    return m_Factions.emplace(faction->GetID(), std::move(faction)).second;
}

bool COrganizingCtrl::AddUnion(std::unique_ptr<CUnion> unionValue)
{
    if (!unionValue) return false;
    return m_Unions.emplace(unionValue->GetID(), std::move(unionValue)).second;
}

CFaction* COrganizingCtrl::GetFaction(const std::int32_t id) noexcept
{
    const auto found = m_Factions.find(id);
    return found == m_Factions.end() ? nullptr : found->second.get();
}

const CFaction* COrganizingCtrl::GetFaction(const std::int32_t id) const noexcept
{
    const auto found = m_Factions.find(id);
    return found == m_Factions.end() ? nullptr : found->second.get();
}

CUnion* COrganizingCtrl::GetUnion(const std::int32_t id) noexcept
{
    const auto found = m_Unions.find(id);
    return found == m_Unions.end() ? nullptr : found->second.get();
}

std::int32_t COrganizingCtrl::FindPlayerFaction(const std::int32_t playerId) const noexcept
{
    for (const auto& [id, faction] : m_Factions) if (faction->IsMember(playerId) > 0) return id;
    return 0;
}

std::int32_t COrganizingCtrl::FindFactionUnion(const std::int32_t factionId) const noexcept
{
    for (const auto& [id, unionValue] : m_Unions) if (unionValue->IsMember(factionId) > 0) return id;
    return 0;
}

bool COrganizingCtrl::DeleteFaction(const std::int32_t id)
{
    if (m_Factions.erase(id) == 0) return false;
    m_DeletedFactions.push_back(id);
    return true;
}

bool COrganizingCtrl::DeleteUnion(const std::int32_t id)
{
    if (m_Unions.erase(id) == 0) return false;
    m_DeletedUnions.push_back(id);
    return true;
}

OrganizingSaveBatch COrganizingCtrl::GenerateSaveData(const bool forceAll)
{
    OrganizingSaveBatch output;
    for (auto& [id, faction] : m_Factions) {
        (void)id;
        if (forceAll) faction->SetChangeData(1 | 2 | 4 | 8);
        if (auto snapshot = faction->CloneSaveData()) {
            output.factions.push_back(std::move(*snapshot));
            faction->SetChangeData(0);
        }
    }
    for (auto& [id, unionValue] : m_Unions) {
        (void)id;
        if (forceAll) unionValue->SetChangeData(1 | 2 | 4 | 8);
        if (auto snapshot = unionValue->CloneSaveData()) {
            output.unions.push_back(std::move(*snapshot));
            unionValue->SetChangeData(0);
        }
    }
    output.deletedFactions.assign(m_DeletedFactions.begin(), m_DeletedFactions.end());
    output.deletedUnions.assign(m_DeletedUnions.begin(), m_DeletedUnions.end());
    m_DeletedFactions.clear();
    m_DeletedUnions.clear();
    return output;
}

std::int32_t COrganizingCtrl::AddTopInfo(const std::int32_t timerFlag,
                                         const std::int32_t parameter,
                                         std::string info)
{
    const auto id = m_NextTopInfoId++;
    m_TopInfo.push_back({id, timerFlag, parameter, Now(), std::move(info)});
    return id;
}

bool COrganizingCtrl::RemoveTopInfo(const std::int32_t id)
{
    const auto position = std::ranges::find(m_TopInfo, id, &OrganizingTopInfo::id);
    if (position == m_TopInfo.end()) return false;
    m_TopInfo.erase(position);
    return true;
}

std::vector<OrganizingTopInfo> COrganizingCtrl::ActiveTopInfo() const
{
    const auto now = Now();
    std::vector<OrganizingTopInfo> result;
    for (const auto& info : m_TopInfo) {
        if (info.timerFlag == 2 && now - info.startedAtMilliseconds >= static_cast<std::uint32_t>(info.parameter)) continue;
        auto copy = info;
        if (copy.timerFlag == 2) copy.parameter -= static_cast<std::int32_t>(now - copy.startedAtMilliseconds);
        result.push_back(std::move(copy));
    }
    return result;
}

std::size_t COrganizingCtrl::Run(
    const std::int32_t elapsedMinutes,
    const std::function<void(std::int32_t, std::int32_t)>& disband)
{
    std::vector<std::pair<std::int32_t, std::int32_t>> pending;
    for (auto& [id, faction] : m_Factions) {
        if (faction->GetDeleteRemainTime() <= 0) continue;
        faction->SetDeleteRemainTime(faction->GetDeleteRemainTime() - elapsedMinutes);
        if (faction->GetDeleteRemainTime() <= 0) pending.emplace_back(id, faction->GetMasterID());
    }
    for (const auto [id, master] : pending) disband(master, id);
    const auto before = m_TopInfo.size();
    const auto now = Now();
    std::erase_if(m_TopInfo, [now](const OrganizingTopInfo& info) {
        return info.timerFlag == 2 && static_cast<std::uint32_t>(info.parameter) <= now - info.startedAtMilliseconds;
    });
    return before - m_TopInfo.size();
}

std::uint32_t COrganizingCtrl::Now() const
{
    if (m_Clock) return m_Clock();
    const auto milliseconds = std::chrono::duration_cast<std::chrono::milliseconds>(
        std::chrono::steady_clock::now().time_since_epoch()).count();
    return static_cast<std::uint32_t>(milliseconds);
}
