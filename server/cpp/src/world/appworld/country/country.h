#pragma once

#include "king.h"
#include "minister.h"

#include <array>
#include <cstdint>
#include <map>
#include <memory>
#include <optional>
#include <string>

/*
 * Исходный владелец: WorldServer/appworld/country/country.cpp/.h.
 * Country scalar state и CloneSaveData подтверждены PDB и реализованы в
 * Rust-реконструкции. Save clone берёт максимум шесть ministers по ID type
 * 2..7 и clamp-ит king points. unique_ptr/std::map заменяют только старое
 * владение и MSVC tree; неподтверждённый AI остаётся callback-границей.
 */
struct CountryMinisterSnapshot {
    std::int32_t id{};
    std::string name;
    std::uint8_t job{};
    bool online{};
    bool appointed{};
    bool salaryReceived{};
};

struct CountryKingSnapshot : CountryMinisterSnapshot {
    std::int32_t controlPoint{};
    std::int32_t materialPoint{};
    std::int32_t warPoint{};
};

struct CountrySaveSnapshot {
    std::uint8_t countryId{};
    std::int32_t treasury{};
    std::int32_t power{};
    std::int32_t techExperience{};
    std::int32_t techLevel{};
    CountryKingSnapshot king;
    std::int32_t warResult{};
    std::array<std::optional<CountryMinisterSnapshot>, 6> ministers;
};

class CCountry
{
public:
    struct SaveLimits { std::int32_t controlPoint{}, materialPoint{}, warPoint{}; };

    explicit CCountry(std::uint8_t id = 0) noexcept : m_CountryId(id) {}
    [[nodiscard]] CountrySaveSnapshot CloneSaveData(SaveLimits limits) const;
    [[nodiscard]] std::uint8_t GetID() const noexcept { return m_CountryId; }
    [[nodiscard]] CKing& King() noexcept { return m_King; }
    [[nodiscard]] const CKing& King() const noexcept { return m_King; }
    [[nodiscard]] std::map<std::uint8_t, std::unique_ptr<CMinister>>& Ministers() noexcept { return m_Ministers; }
    [[nodiscard]] std::int32_t SetPower(std::int32_t value, std::int32_t maximum) noexcept;
    [[nodiscard]] std::int32_t SetTreasury(std::int32_t value, std::int32_t maximum) noexcept;
    [[nodiscard]] std::int32_t SetTechnologyExperience(std::int32_t value) noexcept;
    void SetTechnologyLevel(std::int32_t value) noexcept { m_TechnologyLevel = value; }
    void SetTechnologyLevelUpExperience(std::int32_t value) noexcept { m_TechnologyLevelUpExperience = value; }
    void SetWarResult(std::int32_t value) noexcept { m_WarResult = value; }
    [[nodiscard]] std::int32_t GetWarResult() const noexcept { return m_WarResult; }
    void Restore(const CountrySaveSnapshot& snapshot) noexcept;

private:
    std::uint8_t m_CountryId{};
    std::int32_t m_Treasury{};
    std::int32_t m_Power{};
    std::int32_t m_TechnologyExperience{};
    std::int32_t m_TechnologyLevelUpExperience{};
    std::int32_t m_TechnologyLevel{};
    CKing m_King;
    std::map<std::uint8_t, std::unique_ptr<CMinister>> m_Ministers;
    std::int32_t m_WarResult{};
};
