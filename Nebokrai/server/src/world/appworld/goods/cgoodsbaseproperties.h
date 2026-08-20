#pragma once

#include <cstdint>
#include <span>
#include <string>
#include <utility>
#include <vector>

/*
 * Исходный владелец: WorldServer/appworld/goods/cgoodsbaseproperties.cpp/.h.
 * Числовые GOODS_TYPE/EQUIP_PLACE/GAP и поведение запросов первого
 * совпавшего addon-property подтверждены WorldServer.pdb и Rust. Полный
 * загрузчик ordered-token конфигурации будет собран вместе с CGoodsFactory;
 * этот owner пока хранит уже разобранные значения без старой STL-машинерии.
 */
class CGoodsBaseProperties
{
public:
    enum class GoodsType : std::int32_t {
        Useless = 0,
        Consumable = 1,
        Equipment = 2,
    };

    enum class EquipPlace : std::int32_t {
        Unknown = 0,
        Head = 1,
        Body = 2,
        Hand = 3,
        Glove = 4,
        Boot = 5,
        Ornaments = 6,
        Medal = 7,
        Posterior = 8,
        Jewelry = 9,
        Headgear = 10,
        Talisman = 11,
        Frock = 12,
        Wing = 13,
        Manteau = 14,
        Fairy = 15,
        LingBao = 16,
    };

    enum class AddonType : std::int32_t {
        Unknown = 0,
        ParticularAttribute = 0x0D,
        GoodsStackingLimit = 0x26,
        WeaponLevel = 0x30,
    };

    struct AddonValueModifier {
        std::uint32_t probability{};
        std::int32_t lowerLimit{};
        std::int32_t upperLimit{};
    };

    struct AddonValue {
        std::uint32_t id{};
        std::int32_t baseValue{};
        bool modifierEnabled{};
        std::vector<AddonValueModifier> modifiers;
    };

    struct AddonProperty {
        AddonType type{AddonType::Unknown};
        bool enabled{};
        bool implicitAttribute{};
        std::uint32_t occurProbability{};
        std::vector<AddonValue> values;
    };

    [[nodiscard]] GoodsType GetGoodsType() const noexcept { return m_GoodsType; }
    [[nodiscard]] EquipPlace GetEquipPlace() const noexcept { return m_EquipPlace; }
    [[nodiscard]] std::span<const AddonValue> GetAddonPropertyValues(AddonType type) const noexcept;
    [[nodiscard]] std::uint32_t GetOccurProbability(AddonType type) const noexcept;
    [[nodiscard]] std::uint32_t GetWeight() const noexcept { return m_Weight; }
    [[nodiscard]] std::uint32_t GetPrice() const noexcept { return m_Price; }
    [[nodiscard]] const std::string& GetName() const noexcept { return m_Name; }
    [[nodiscard]] const std::string& GetOriginalName() const noexcept { return m_OriginalName; }
    [[nodiscard]] const std::string& GetDescription() const noexcept { return m_Description; }

    void SetGoodsType(GoodsType value) noexcept { m_GoodsType = value; }
    void SetEquipPlace(EquipPlace value) noexcept { m_EquipPlace = value; }
    void SetWeight(std::uint32_t value) noexcept { m_Weight = value; }
    void SetPrice(std::uint32_t value) noexcept { m_Price = value; }
    void SetName(std::string value) { m_Name = std::move(value); }
    void SetOriginalName(std::string value) { m_OriginalName = std::move(value); }
    void SetDescription(std::string value) { m_Description = std::move(value); }
    [[nodiscard]] std::vector<AddonProperty>& AddonProperties() noexcept { return m_AddonProperties; }
    [[nodiscard]] const std::vector<AddonProperty>& AddonProperties() const noexcept { return m_AddonProperties; }

private:
    std::string m_OriginalName;
    std::string m_Name;
    std::uint32_t m_Price{};
    std::uint32_t m_Weight{};
    std::string m_Description;
    GoodsType m_GoodsType{GoodsType::Useless};
    EquipPlace m_EquipPlace{EquipPlace::Unknown};
    std::vector<AddonProperty> m_AddonProperties;
};
