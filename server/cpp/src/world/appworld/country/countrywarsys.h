#pragma once

#include <cstdint>
#include <functional>
#include <map>
#include <optional>
#include <string>
#include <string_view>

/*
 * Исходный владелец: WorldServer/appworld/country/countrywarsys.cpp/.h.
 * War-region state, clear ordering и defender/attacker results 2/1 либо 1/2
 * подтверждены exact EXE и Rust owner-ом. Network/localization/region lookup
 * оставлены callbacks, чтобы этот доменный класс не владел transport-ом.
 */
class CountryWarSys
{
public:
    struct RegionState { bool clear{}; std::int32_t defender{}; std::int32_t attacker{}; };
    struct Context {
        std::function<std::optional<std::string>(std::int32_t)> regionName;
        std::function<bool(std::uint8_t)> countryExists;
        std::function<void(std::uint8_t)> sendDestroyedFlag;
        std::function<void(std::uint8_t, std::int32_t)> setWarResult;
        std::function<std::string(bool, std::int32_t, std::int32_t, std::string_view)> formatNotice;
        std::function<void(std::string_view, std::uint32_t, std::uint32_t)> sendCountryInfo;
    };

    [[nodiscard]] std::map<std::int32_t, RegionState>& Regions() noexcept { return m_Regions; }
    void OnFlagDestroyed(std::int32_t country, Context& context);

private:
    std::map<std::int32_t, RegionState> m_Regions;
};
