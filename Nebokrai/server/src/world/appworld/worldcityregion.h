#pragma once

#include "worldwarregion.h"

#include <array>
#include <functional>
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
    [[nodiscard]] bool LoadCitySetup(
        std::string_view text,
        const std::function<std::string(std::string_view)>& resolveName);
    void SetDefenceSetup(const Setup& setup) noexcept;
    [[nodiscard]] std::vector<Build>& Gates() noexcept { return m_Gates; }

private:
    std::array<std::optional<std::int32_t>, 8> m_DefenceSetup;
    std::vector<Build> m_Gates;
};
