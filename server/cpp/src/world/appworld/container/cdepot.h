#pragma once

#include "cvolumelimitgoodscontainer.h"

class CDepot final : public CVolumeLimitGoodsContainer
{
public:
    explicit CDepot(const CGoodsFactory& factory) : CVolumeLimitGoodsContainer(factory) {}

    bool Add(GoodsPtr& goods, void* context = nullptr) override;
    bool AddAt(std::uint32_t position, GoodsPtr& goods, void* context = nullptr);
    GoodsPtr Remove(const CGUID& guid, void* context = nullptr) override;
    void SetLocked(bool value) noexcept { m_Locked = value; }
    [[nodiscard]] bool IsLocked() const noexcept { return m_Locked; }

private:
    bool m_Locked{};
};
