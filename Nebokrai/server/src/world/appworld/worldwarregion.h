#pragma once

#include "worldregion.h"

#include <optional>

class CWorldWarRegion : public CWorldRegion
{
public:
    CWorldWarRegion() = default;
    ~CWorldWarRegion() override = default;

    void SetSymbols(std::int32_t total, std::int32_t win, std::int32_t victory) noexcept;
    bool LoadWarSetup(std::string_view text);
    bool AddToByteArray(std::vector<std::uint8_t>& output,
                        bool includeChild) const override;
    bool DecordFromByteArray(std::span<const std::uint8_t>,
                             std::size_t&,
                             bool) override { return true; }

protected:
    std::optional<std::int32_t> m_SymbolTotal;
    std::optional<std::int32_t> m_WinVictorySymbols;
    std::optional<std::int32_t> m_VictorySymbols;
};
