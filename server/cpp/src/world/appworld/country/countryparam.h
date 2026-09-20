#pragma once

#include <array>
#include <cstdint>
#include <map>
#include <optional>
#include <span>
#include <string_view>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/country/countryparam.cpp/.h.
 * 39 параметров, CountryParam.ini markers `* # +`, map overwrite и точный
 * wire-порядок подтверждены EXE/PDB и реализованной Rust-реконструкцией.
 * Неинициализированные constructor scalars остаются optional; singleton и
 * MSVC trees заменены явным owner-ом/std::map.
 */
class CCountryParam
{
public:
    struct Rect { std::int32_t left{}, top{}, right{}, bottom{}; };
    struct ReturnPoint { std::int32_t regionId{}; Rect rect{}; std::int32_t direction{}; };
    struct TechLevel { std::int32_t experience{}; std::int32_t power{}; };
    static constexpr std::size_t ParameterCount = 39;

    bool Load(std::optional<std::string_view> source);
    bool Initialize(std::optional<std::string_view> source) { return Load(source); }
    [[nodiscard]] ReturnPoint MainReturnPoint(std::uint8_t country);
    [[nodiscard]] std::optional<std::int32_t> MaxKingControlPoint() const noexcept { return m_Parameters[10]; }
    [[nodiscard]] std::optional<std::int32_t> MaxKingMaterialPoint() const noexcept { return m_Parameters[26]; }
    [[nodiscard]] std::optional<std::int32_t> MaxKingWarPoint() const noexcept { return m_Parameters[28]; }
    [[nodiscard]] std::optional<std::int32_t> Parameter(std::size_t index) const noexcept;
    bool AddToByteArray(std::vector<std::uint8_t>& output) const;

private:
    std::array<std::optional<std::int32_t>, ParameterCount> m_Parameters{};
    std::map<std::uint8_t, std::int32_t> m_StartRegions;
    std::map<std::uint8_t, Rect> m_StartRects;
    std::map<std::uint8_t, std::int32_t> m_StartDirections;
    std::map<std::uint8_t, std::int32_t> m_MainRegions;
    std::map<std::uint8_t, Rect> m_MainRects;
    std::map<std::uint8_t, std::int32_t> m_MainDirections;
    std::map<std::int32_t, TechLevel> m_TechLevels;
    std::map<std::uint8_t, Rect> m_ExileRects;
};
