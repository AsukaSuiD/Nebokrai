#pragma once

#include "ccontainer.h"
#include "../goods/cgoodsfactory.h"

#include <cstddef>
#include <cstdint>
#include <map>
#include <memory>
#include <span>
#include <vector>

/*
 * Исходный владелец: cequipmentcontainer.cpp/.h. Колонки 0..16 и соответствие
 * EQUIP_PLACE подтверждены PDB/Rust; ornaments допускает ровно два слота.
 * map владеет товарами напрямую и тем самым заменяет старые raw pointers.
 */
class CEquipmentContainer final : public CContainer
{
public:
    enum class Column : std::uint32_t {
        Head = 0, Body, Hand, Glove, Boot, Jewelry, OrnamentsOne,
        OrnamentsTwo, Medal, Posterior, Headgear, Talisman, Frock,
        Wing, Manteau, Fairy, LingBao,
    };

    explicit CEquipmentContainer(const CGoodsFactory& factory) noexcept : m_Factory(factory) {}
    bool Add(Column column, std::unique_ptr<CGoods>& goods, void* context = nullptr);
    bool AddFromDB(std::uint32_t position,
                   std::unique_ptr<CGoods>& goods,
                   void* context = nullptr);
    std::unique_ptr<CGoods> Remove(const CGUID& guid, void* context = nullptr);
    void Clear(void* context = nullptr);
    void Release(void* context = nullptr);
    [[nodiscard]] CGoods* GetGoods(Column column) const noexcept;
    [[nodiscard]] CGoods* GetGoods(std::uint32_t position) const noexcept;
    [[nodiscard]] std::uint32_t GetGoodsAmount() const noexcept;
    [[nodiscard]] std::uint32_t GetContentsWeight() const noexcept;
    bool Serialize(std::vector<std::uint8_t>& output) const;
    bool Unserialize(std::span<const std::uint8_t> input, std::size_t& offset);
    void Traverse(CContainerListener& listener) override;
    void AI();

private:
    [[nodiscard]] static bool Accepts(Column column,
                                      CGoodsBaseProperties::EquipPlace place) noexcept;
    [[nodiscard]] static bool IsColumn(std::uint32_t value) noexcept;

    const CGoodsFactory& m_Factory;
    std::map<Column, std::unique_ptr<CGoods>> m_Equipment;
};
