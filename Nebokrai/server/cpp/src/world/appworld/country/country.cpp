#include "country.h"

#include <algorithm>

namespace {
CountryMinisterSnapshot Snapshot(const COfficer& officer) {
    return {officer.GetID(), std::string(officer.GetName()), officer.GetJob(), officer.IsOnline(), officer.IsAppointed(), officer.SalaryReceived()};
}
}

CountrySaveSnapshot CCountry::CloneSaveData(SaveLimits limits) const
{
    CountrySaveSnapshot result;
    result.countryId = m_CountryId; result.treasury = m_Treasury; result.power = m_Power;
    result.techExperience = m_TechnologyExperience; result.techLevel = m_TechnologyLevel;
    static_cast<CountryMinisterSnapshot&>(result.king) = Snapshot(m_King);
    result.king.controlPoint = std::min(m_King.GetControlPoint(), limits.controlPoint);
    result.king.materialPoint = std::min(m_King.GetMaterialPoint(), limits.materialPoint);
    result.king.warPoint = std::min(m_King.GetWarPoint(), limits.warPoint);
    result.warResult = m_WarResult;
    std::size_t copied{};
    for (const auto& [key, minister] : m_Ministers) {
        (void)key;
        if (minister == nullptr || copied == result.ministers.size()) continue;
        const auto job = minister->GetJob();
        if (job >= 2 && job <= 7 && !result.ministers[job - 2]) {
            result.ministers[job - 2] = Snapshot(*minister); ++copied;
        }
    }
    return result;
}

std::int32_t CCountry::SetPower(std::int32_t value, std::int32_t maximum) noexcept
{
    const auto previous = m_Power; m_Power = std::clamp(value, 0, maximum); return previous;
}

std::int32_t CCountry::SetTreasury(std::int32_t value, std::int32_t maximum) noexcept
{
    const auto previous = m_Treasury; m_Treasury = std::clamp(value, 0, maximum); return previous;
}

std::int32_t CCountry::SetTechnologyExperience(std::int32_t value) noexcept
{
    const auto previous = m_TechnologyExperience;
    m_TechnologyExperience = std::min(value, m_TechnologyLevelUpExperience);
    return previous;
}

void CCountry::Restore(const CountrySaveSnapshot& snapshot) noexcept
{
    m_CountryId = snapshot.countryId;
    m_Treasury = snapshot.treasury;
    m_Power = snapshot.power;
    m_TechnologyExperience = snapshot.techExperience;
    m_TechnologyLevel = snapshot.techLevel;
    m_WarResult = snapshot.warResult;

    m_King.SetID(snapshot.king.id);
    m_King.SetName(snapshot.king.name);
    m_King.SetJob(snapshot.king.job);
    m_King.SetOnline(snapshot.king.online);
    m_King.SetAppointed(snapshot.king.appointed);
    m_King.SetSalaryReceived(snapshot.king.salaryReceived);
    m_King.SetControlPoint(snapshot.king.controlPoint, snapshot.king.controlPoint);
    m_King.SetMaterialPoint(snapshot.king.materialPoint, snapshot.king.materialPoint);
    m_King.SetWarPoint(snapshot.king.warPoint, snapshot.king.warPoint);

    m_Ministers.clear();
    for (std::size_t index = 0; index < snapshot.ministers.size(); ++index) {
        if (!snapshot.ministers[index]) continue;
        const CountryMinisterSnapshot& source = *snapshot.ministers[index];
        auto minister = std::make_unique<CMinister>();
        minister->SetID(source.id);
        minister->SetName(source.name);
        minister->SetJob(source.job == 0 ? static_cast<std::uint8_t>(index + 2) : source.job);
        minister->SetOnline(source.online);
        minister->SetAppointed(source.appointed);
        minister->SetSalaryReceived(source.salaryReceived);
        m_Ministers.emplace(static_cast<std::uint8_t>(index + 2), std::move(minister));
    }
}
