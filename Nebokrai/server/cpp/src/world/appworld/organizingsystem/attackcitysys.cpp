#include "attackcitysys.h"

#include <algorithm>

bool CAttackCitySys::AddSchedule(AttackCitySchedule schedule)
{
    return m_Attacks.emplace(schedule.warNumber, std::move(schedule)).second;
}

AttackCitySchedule* CAttackCitySys::GetSchedule(const std::int32_t warNumber) noexcept
{
    const auto found = m_Attacks.find(warNumber);
    return found == m_Attacks.end() ? nullptr : &found->second;
}

const AttackCitySchedule* CAttackCitySys::GetSchedule(const std::int32_t warNumber) const noexcept
{
    const auto found = m_Attacks.find(warNumber);
    return found == m_Attacks.end() ? nullptr : &found->second;
}

ECityState CAttackCitySys::GetCityState(const std::int32_t regionId) const noexcept
{
    for (const auto& [number, schedule] : m_Attacks) {
        (void)number;
        if (schedule.cityRegionId == regionId) return schedule.state;
    }
    return ECityState::None;
}

ECityState CAttackCitySys::GetStateByWarNumber(const std::int32_t warNumber) const noexcept
{
    const auto* schedule = GetSchedule(warNumber);
    return schedule == nullptr ? ECityState::None : schedule->state;
}

bool CAttackCitySys::IsAlreadyDeclared(const std::int32_t factionId) const noexcept
{
    for (const auto& [number, schedule] : m_Attacks) {
        (void)number;
        if (std::ranges::find(schedule.declaringFactions, factionId) != schedule.declaringFactions.end()) return true;
    }
    return false;
}

bool CAttackCitySys::Declare(const std::int32_t warNumber, const std::int32_t factionId)
{
    auto* schedule = GetSchedule(warNumber);
    if (schedule == nullptr || schedule->state != ECityState::Declare || IsAlreadyDeclared(factionId)) return false;
    schedule->declaringFactions.push_back(factionId);
    return true;
}

bool CAttackCitySys::SetPhase(const std::int32_t warNumber, const ECityState state) noexcept
{
    auto* schedule = GetSchedule(warNumber);
    if (schedule == nullptr) return false;
    schedule->state = state;
    if (state == ECityState::Fight) schedule->awaitingResult = true;
    return true;
}

std::optional<AttackCityResult> CAttackCitySys::Finish(const std::int32_t warNumber,
                                                       const std::int32_t factionId,
                                                       const std::int32_t unionId)
{
    auto* schedule = GetSchedule(warNumber);
    if (schedule == nullptr || !schedule->awaitingResult || schedule->resultCompleted) return std::nullopt;
    schedule->resultCompleted = true;
    schedule->awaitingResult = false;
    schedule->state = ECityState::None;
    return AttackCityResult{warNumber, schedule->cityRegionId, factionId, unionId};
}
