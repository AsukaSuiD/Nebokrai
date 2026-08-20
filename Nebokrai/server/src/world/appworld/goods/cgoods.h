#pragma once

#include "../shape.h"
#include "cgoodsbaseproperties.h"

#include <cstddef>
#include <cstdint>
#include <span>
#include <string>
#include <string_view>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/goods/cgoods.cpp / cgoods.h.
 * Constructor type=700, amount=1, addon lookup, Release и полный snapshot-
 * формат подтверждены Nworldserver.exe/PDB и поздней Rust-реконструкцией.
 * Декодирование принимает span и ограничивает legacy C-string описания 1024
 * байт; это безопасная замена старого локального буфера, не новый wire-формат.
 */
class CGoods : public CShape
{
public:
    struct AddonValue {
        std::uint32_t id{};
        std::int32_t baseValue{};
        std::int32_t modifier{};
    };

    struct AddonProperty {
        CGoodsBaseProperties::AddonType type{CGoodsBaseProperties::AddonType::Unknown};
        bool enabled{};
        bool implicitAttribute{};
        std::vector<AddonValue> values;
    };

    CGoods();
    ~CGoods() override = default;

    bool AddToByteArray(std::vector<std::uint8_t>& output,
                        bool includeChild) const override;
    bool DecordFromByteArray(std::span<const std::uint8_t> input,
                             std::size_t& offset,
                             bool includeChild) override;
    void Release() noexcept;

    [[nodiscard]] std::uint32_t GetBasePropertiesIndex() const noexcept { return m_BasePropertiesIndex; }
    [[nodiscard]] std::uint32_t GetAmount() const noexcept { return m_Amount; }
    [[nodiscard]] std::uint32_t GetPrice() const noexcept { return m_Price; }
    [[nodiscard]] std::uint32_t GetWeight(const CGoodsBaseProperties* properties) const noexcept;
    [[nodiscard]] std::uint32_t GetMaxStackNumber(const CGoodsBaseProperties* properties) const noexcept;
    [[nodiscard]] std::int32_t GetAddonPropertyValue(CGoodsBaseProperties::AddonType type,
                                                     std::uint32_t id) const noexcept;
    [[nodiscard]] bool HasAddonProperty(CGoodsBaseProperties::AddonType type) const noexcept;
    [[nodiscard]] const std::string& GetDescription() const noexcept { return m_Description; }
    [[nodiscard]] std::vector<AddonProperty>& AddonProperties() noexcept { return m_AddonProperties; }
    [[nodiscard]] const std::vector<AddonProperty>& AddonProperties() const noexcept { return m_AddonProperties; }

    void SetBasePropertiesIndex(std::uint32_t value) noexcept { m_BasePropertiesIndex = value; }
    void SetAmount(std::uint32_t value) noexcept { m_Amount = value; }
    void SetPrice(std::uint32_t value) noexcept { m_Price = value; }
    void SetGoodsDescription(std::string_view value);
    [[nodiscard]] bool CloneInto(CGoods& destination) const;

private:
    std::uint32_t m_BasePropertiesIndex{};
    std::uint32_t m_Amount{1};
    std::uint32_t m_Price{};
    std::string m_Description;
    std::vector<AddonProperty> m_AddonProperties;
};
