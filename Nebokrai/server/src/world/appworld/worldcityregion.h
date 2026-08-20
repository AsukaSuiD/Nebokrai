#pragma once

#include "worldwarregion.h"

#include <array>
#include <optional>
#include <string>
#include <vector>

class CWorldCityRegion final : public CWorldWarRegion
{
public:
    struct Build { std::array<std::uint8_t, 0x2C> header{}; std::string name; std::string script; };

    CWorldCityRegion() { SetSymbols(3, 3, 2); }
    bool AddToByteArray(std::vector<std::uint8_t>& output,
                        bool includeChild) const override;
    void SetDefenceSetup(const Setup& setup) noexcept { m_DefenceSetup = setup; }
    [[nodiscard]] std::vector<Build>& Gates() noexcept { return m_Gates; }

private:
    std::optional<Setup> m_DefenceSetup;
    std::vector<Build> m_Gates;
};
