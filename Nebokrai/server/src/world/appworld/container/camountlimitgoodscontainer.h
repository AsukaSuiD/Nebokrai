#pragma once

#include "cgoodscontainer.h"

#include <cstdint>
#include <vector>

/*
 * Исходный владелец: camountlimitgoodscontainer.cpp/.h. Default limit=1,
 * unsigned full-check, GUID-keyed storage и Release-reset подтверждены
 * WorldServer.pdb/Rust. Locked GUID-ы не меняют владение товаром.
 */
class CAmountLimitGoodsContainer : public CGoodsContainer
{
public:
    explicit CAmountLimitGoodsContainer(const CGoodsFactory& factory)
        : CGoodsContainer(factory) {}

    bool Add(GoodsPtr& goods, void* context = nullptr) override;
    bool IsFull() const noexcept override { return GetGoodsAmount() >= m_GoodsAmountLimit; }
    void Clear(void* context = nullptr) override;
    void Release(void* context = nullptr) override;

    void SetGoodsAmountLimit(std::uint32_t value) noexcept { m_GoodsAmountLimit = value; }
    [[nodiscard]] std::uint32_t GetGoodsAmountLimit() const noexcept { return m_GoodsAmountLimit; }
    [[nodiscard]] bool Lock(const CGUID& guid);
    [[nodiscard]] bool Unlock(const CGUID& guid) noexcept;
    [[nodiscard]] bool IsLocked(const CGUID& guid) const noexcept;
    [[nodiscard]] std::uint32_t GetContentsWeight() const noexcept;

private:
    std::uint32_t m_GoodsAmountLimit{1};
    std::vector<CGUID> m_LockedGoods;
};
