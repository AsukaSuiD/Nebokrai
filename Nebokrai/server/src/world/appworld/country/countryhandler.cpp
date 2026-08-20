#include "countryhandler.h"

#include <algorithm>
#include <chrono>

std::uint32_t CCountryHandler::Now() noexcept
{
    return static_cast<std::uint32_t>(std::chrono::duration_cast<std::chrono::milliseconds>(
        std::chrono::steady_clock::now().time_since_epoch()).count());
}

CCountry* CCountryHandler::GetCountry(std::uint8_t id) const noexcept
{
    if (id == 0) return nullptr;
    const auto found = m_Countries.find(id); return found == m_Countries.end() ? nullptr : found->second.get();
}

bool CCountryHandler::AddCountry(std::unique_ptr<CCountry> country)
{
    if (!country || country->GetID() == 0 || m_Countries.contains(country->GetID())) return false;
    m_Countries.emplace(country->GetID(), std::move(country)); return true;
}

bool CCountryHandler::SetCountryWarResult(std::uint8_t id, std::int32_t result) noexcept
{
    CCountry* country = GetCountry(id); if (country == nullptr) return false; country->SetWarResult(result); return true;
}

std::vector<CountrySaveSnapshot> CCountryHandler::GenerateSaveData(CCountry::SaveLimits limits) const
{
    std::vector<CountrySaveSnapshot> result; result.reserve(m_Countries.size());
    for (const auto& [id, country] : m_Countries) { (void)id; if (country) result.push_back(country->CloneSaveData(limits)); }
    return result;
}

std::int32_t CCountryHandler::AddOneTopInfo(std::int32_t timerFlag,
                                            std::int32_t parameter,
                                            std::string_view info)
{
    const auto id = m_NextTopInfoId++;
    m_TopInfos.push_back({id, timerFlag, parameter, Now(), std::string(info.substr(0, info.find('\0')))});
    return id;
}

void CCountryHandler::Run(std::int32_t minuteDelta, const CountryAi& countryAi)
{
    (void)minuteDelta;
    std::erase_if(m_TopInfos, [](const TopInfo& info) {
        return info.timerFlag == 2 && static_cast<std::uint32_t>(Now() - info.startedAt) >= static_cast<std::uint32_t>(info.parameter);
    });
    if (!countryAi) return;
    for (auto& [id, country] : m_Countries) {
        (void)id;
        if (country && country->King().GetID() != 0 && !country->King().GetName().empty()) countryAi(*country);
    }
}

std::vector<CCountryHandler::ClientInfo> CCountryHandler::TopInfoSnapshot() const
{
    std::vector<ClientInfo> result; result.reserve(m_TopInfos.size());
    for (const auto& info : m_TopInfos) result.push_back({info.text, 0, 0});
    return result;
}
