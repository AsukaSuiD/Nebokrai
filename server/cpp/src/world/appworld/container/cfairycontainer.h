#pragma once

#include "cvolumelimitgoodscontainer.h"

#include <array>
#include <cstdint>

class CFairyContainer final : public CVolumeLimitGoodsContainer
{
public:
    explicit CFairyContainer(const CGoodsFactory& factory)
        : CVolumeLimitGoodsContainer(factory) {}

    void SetHatchTime(std::size_t slot, std::uint32_t value);
    [[nodiscard]] std::uint32_t GetHatchTime(std::size_t slot) const;

private:
    std::array<std::uint32_t, 5> m_HatchTime{};
};
