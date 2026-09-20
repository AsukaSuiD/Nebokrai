#include "fournationwarsys.h"

#include <utility>

bool CFourNationWarSys::AddSchedule(FourNationWarSchedule schedule)
{
    return m_Schedules.emplace(schedule.warNumber, std::move(schedule)).second;
}

void CFourNationWarSys::SetFund(const std::int32_t country, const FourNationFund fund)
{
    m_Funds[country] = fund;
}

const FourNationFund* CFourNationWarSys::GetFund(const std::int32_t country) const noexcept
{
    const auto found = m_Funds.find(country);
    return found == m_Funds.end() ? nullptr : &found->second;
}

FourNationWarSchedule* CFourNationWarSys::GetSchedule(const std::int32_t number) noexcept
{
    const auto found = m_Schedules.find(number);
    return found == m_Schedules.end() ? nullptr : &found->second;
}

bool CFourNationWarSys::SetPhase(const std::int32_t number, const ECityState state) noexcept
{
    auto* schedule = GetSchedule(number);
    if (schedule == nullptr) return false;
    schedule->state = state;
    return true;
}

bool CFourNationWarSys::BeginResultCollection(const std::int32_t number, const std::size_t expected)
{
    if (GetSchedule(number) == nullptr || expected == 0) return false;
    return m_Pending.emplace(number, Pending{expected, {}}).second;
}

std::optional<std::vector<FourNationResult>> CFourNationWarSys::AddResult(FourNationResult result)
{
    const auto pending = m_Pending.find(result.warNumber);
    if (pending == m_Pending.end()) return std::nullopt;
    pending->second.byCountry[result.country] = result;
    if (pending->second.byCountry.size() < pending->second.expected) return std::nullopt;
    std::vector<FourNationResult> complete;
    complete.reserve(pending->second.byCountry.size());
    for (const auto& [country, value] : pending->second.byCountry) {
        (void)country;
        complete.push_back(value);
    }
    m_Pending.erase(pending);
    return complete;
}
