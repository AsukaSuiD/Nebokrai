#pragma once

#include "camountlimitgoodscontainer.h"

#include <cstdint>
#include <vector>

/*
 * Исходный владелец: cvolumelimitgoodscontainer.cpp/.h. Cell-array хранит
 * GUID, size setter сначала Release, затем создаёт пустые клетки и ставит
 * amount limit=size. Эти порядок и defaults подтверждены Rust/PDB.
 */
class CVolumeLimitGoodsContainer : public CAmountLimitGoodsContainer
{
public:
    explicit CVolumeLimitGoodsContainer(const CGoodsFactory& factory)
        : CAmountLimitGoodsContainer(factory) {}

    void SetContainerVolume(std::uint32_t size);
    void SetContainerVolume(std::uint32_t rows, std::uint32_t columns);
    void Clear(void* context = nullptr) override;
    void Release(void* context = nullptr) override;
    bool Add(GoodsPtr& goods, void* context = nullptr) override;
    bool AddAt(std::uint32_t position, GoodsPtr& goods, void* context = nullptr);
    bool AddFromDB(GoodsPtr& goods,
                   std::uint32_t position = 0,
                   void* context = nullptr) override;
    GoodsPtr Remove(const CGUID& guid, void* context = nullptr) override;
    [[nodiscard]] CGoods* GetGoods(std::uint32_t position) const noexcept override;
    [[nodiscard]] bool QueryGoodsPosition(const CGUID& guid,
                                          std::uint32_t& position) const noexcept;
    [[nodiscard]] bool IsSpaceEnough(std::uint32_t position) const noexcept;
    [[nodiscard]] std::uint32_t GetVolume() const noexcept { return m_Size; }

private:
    [[nodiscard]] bool FindFreePosition(std::uint32_t& position) const noexcept;

    std::uint32_t m_Size{};
    std::vector<CGUID> m_Cells;
};
