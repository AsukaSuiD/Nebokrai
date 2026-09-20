#pragma once

#include "worldregion.h"

#include <array>
#include <string>
#include <vector>

class WorldCountryWarRegion final : public CWorldRegion
{
public:
    struct Gate { std::array<std::uint8_t, 0x2C> header{}; std::string name; std::string script; };
    struct Flag { std::array<std::uint8_t, 0x28> header{}; std::string name; std::string script; };
    struct Area { std::array<std::uint8_t, 0x14> header{}; };

    bool AddToByteArray(std::vector<std::uint8_t>& output,
                        bool includeChild) const override;
    [[nodiscard]] bool LoadCountrySetup(std::string_view text);

    [[nodiscard]] std::vector<Gate>& DefendGates() noexcept { return m_DefendGates; }
    [[nodiscard]] std::vector<Gate>& AttackGates() noexcept { return m_AttackGates; }
    [[nodiscard]] std::vector<Flag>& DefendFlags() noexcept { return m_DefendFlags; }
    [[nodiscard]] std::vector<Flag>& AttackFlags() noexcept { return m_AttackFlags; }
    [[nodiscard]] std::vector<Area>& DefendAreas() noexcept { return m_DefendAreas; }
    [[nodiscard]] std::vector<Area>& AttackAreas() noexcept { return m_AttackAreas; }

private:
    std::vector<Gate> m_DefendGates;
    std::vector<Gate> m_AttackGates;
    std::vector<Flag> m_DefendFlags;
    std::vector<Flag> m_AttackFlags;
    std::vector<Area> m_DefendAreas;
    std::vector<Area> m_AttackAreas;
};
