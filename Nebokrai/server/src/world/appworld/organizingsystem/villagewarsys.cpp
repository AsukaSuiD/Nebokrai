#include "villagewarsys.h"

#include <algorithm>

bool CVillageWarSys::AddSchedule(VillageWarSchedule schedule)
{
    return m_Wars.emplace(schedule.warNumber, std::move(schedule)).second;
}

VillageWarSchedule* CVillageWarSys::GetSchedule(const std::int32_t number) noexcept
{
    const auto found = m_Wars.find(number);
    return found == m_Wars.end() ? nullptr : &found->second;
}

const VillageWarSchedule* CVillageWarSys::GetSchedule(const std::int32_t number) const noexcept
{
    const auto found = m_Wars.find(number);
    return found == m_Wars.end() ? nullptr : &found->second;
}

ECityState CVillageWarSys::GetRegionState(const std::int32_t regionId) const noexcept
{
    for (const auto& [number, schedule] : m_Wars) {
        (void)number;
        if (schedule.warRegionId == regionId || schedule.villageRegionId == regionId) return schedule.state;
    }
    return ECityState::None;
}

bool CVillageWarSys::IsAlreadyDeclared(const std::int32_t factionId) const noexcept
{
    for (const auto& [number, schedule] : m_Wars) {
        (void)number;
        if (std::ranges::find(schedule.declaringFactions, factionId) != schedule.declaringFactions.end()) return true;
    }
    return false;
}

bool CVillageWarSys::Declare(const std::int32_t number, const std::int32_t factionId)
{
    auto* schedule = GetSchedule(number);
    if (schedule == nullptr || schedule->state != ECityState::Declare || IsAlreadyDeclared(factionId)) return false;
    schedule->declaringFactions.push_back(factionId);
    return true;
}

bool CVillageWarSys::SetPhase(const std::int32_t number, const ECityState state) noexcept
{
    auto* schedule = GetSchedule(number);
    if (schedule == nullptr) return false;
    schedule->state = state;
    if (state == ECityState::Fight) schedule->awaitingResult = true;
    return true;
}

std::optional<VillageWarResult> CVillageWarSys::Finish(const std::int32_t number,
                                                       const std::int32_t factionId)
{
    auto* schedule = GetSchedule(number);
    if (schedule == nullptr || !schedule->awaitingResult || schedule->resultCompleted) return std::nullopt;
    schedule->awaitingResult = false;
    schedule->resultCompleted = true;
    schedule->state = ECityState::None;
    return VillageWarResult{number, schedule->villageRegionId, factionId};
}
