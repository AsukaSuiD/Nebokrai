#pragma once

#include "country.h"

#include <cstdint>
#include <deque>
#include <functional>
#include <map>
#include <memory>
#include <string>
#include <string_view>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/country/countryhandler.cpp/.h.
 * Unsigned country map, top-info expiry и ordered save generation подтверждены
 * EXE/PDB/Rust. Глобальный singleton, CGame и CMessage заменены явными
 * callbacks; payload-поля 0x7FA03 остаются неизменными на message-границе.
 */
class CCountryHandler
{
public:
    struct ClientInfo { std::string text; std::uint32_t title{}; std::uint32_t color{}; };
    using CountryAi = std::function<void(CCountry&)>;

    [[nodiscard]] CCountry* GetCountry(std::uint8_t id) const noexcept;
    bool AddCountry(std::unique_ptr<CCountry> country);
    bool SetCountryWarResult(std::uint8_t id, std::int32_t result) noexcept;
    [[nodiscard]] std::vector<CountrySaveSnapshot> GenerateSaveData(CCountry::SaveLimits limits) const;
    [[nodiscard]] std::int32_t AddOneTopInfo(std::int32_t timerFlag,
                                             std::int32_t parameter,
                                             std::string_view info);
    void Run(std::int32_t minuteDelta, const CountryAi& countryAi);
    [[nodiscard]] std::vector<ClientInfo> TopInfoSnapshot() const;

private:
    struct TopInfo {
        std::int32_t id{};
        std::int32_t timerFlag{};
        std::int32_t parameter{};
        std::uint32_t startedAt{};
        std::string text;
    };
    static std::uint32_t Now() noexcept;
    std::int32_t m_NextTopInfoId{1};
    std::map<std::uint8_t, std::unique_ptr<CCountry>> m_Countries;
    std::deque<TopInfo> m_TopInfos;
};
