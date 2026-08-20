#include "cgoods.h"

#include <algorithm>
#include <cstring>
#include <limits>
#include <type_traits>

namespace
{
template <class T>
void AppendScalar(std::vector<std::uint8_t>& output, const T& value)
{
    static_assert(std::is_trivially_copyable_v<T>);
    const auto* first = reinterpret_cast<const std::uint8_t*>(&value);
    output.insert(output.end(), first, first + sizeof(T));
}

template <class T>
bool ReadScalar(std::span<const std::uint8_t> input, std::size_t& offset, T& value)
{
    static_assert(std::is_trivially_copyable_v<T>);
    if (offset > input.size() || input.size() - offset < sizeof(T)) {
        return false;
    }
    std::memcpy(&value, input.data() + offset, sizeof(T));
    offset += sizeof(T);
    return true;
}

bool ReadCString(std::span<const std::uint8_t> input,
                 std::size_t& offset,
                 std::size_t limit,
                 std::string& value)
{
    if (offset > input.size()) {
        return false;
    }
    const std::size_t available = std::min(limit + 1, input.size() - offset);
    const auto bytes = input.subspan(offset, available);
    const auto terminator = std::ranges::find(bytes, std::uint8_t{0});
    if (terminator == bytes.end() || static_cast<std::size_t>(terminator - bytes.begin()) > limit) {
        return false;
    }
    const std::size_t length = static_cast<std::size_t>(terminator - bytes.begin());
    value.assign(reinterpret_cast<const char*>(bytes.data()), length);
    offset += length + 1;
    return true;
}
}

CGoods::CGoods()
{
    SetType(700);
}

bool CGoods::AddToByteArray(std::vector<std::uint8_t>& output, bool includeChild) const
{
    if (!CShape::AddToByteArray(output, includeChild) ||
        m_AddonProperties.size() > std::numeric_limits<std::uint32_t>::max()) {
        return false;
    }
    AppendScalar(output, m_BasePropertiesIndex);
    AppendScalar(output, m_Amount);
    AppendScalar(output, m_Price);
    output.insert(output.end(), m_Description.begin(), m_Description.end());
    output.push_back(0);
    AppendScalar(output, static_cast<std::uint32_t>(m_AddonProperties.size()));

    for (const AddonProperty& property : m_AddonProperties) {
        if (property.values.size() > std::numeric_limits<std::uint32_t>::max()) {
            return false;
        }
        AppendScalar(output, static_cast<std::int32_t>(property.type));
        AppendScalar(output, static_cast<std::int32_t>(property.enabled));
        AppendScalar(output, static_cast<std::int32_t>(property.implicitAttribute));
        AppendScalar(output, static_cast<std::uint32_t>(property.values.size()));
        for (const AddonValue& value : property.values) {
            AppendScalar(output, value.id);
            AppendScalar(output, value.baseValue);
            AppendScalar(output, value.modifier);
        }
    }
    return true;
}

bool CGoods::DecordFromByteArray(std::span<const std::uint8_t> input,
                                 std::size_t& offset,
                                 bool includeChild)
{
    Release();
    if (!CShape::DecordFromByteArray(input, offset, includeChild) ||
        !ReadScalar(input, offset, m_BasePropertiesIndex) ||
        !ReadScalar(input, offset, m_Amount) || !ReadScalar(input, offset, m_Price) ||
        !ReadCString(input, offset, 1024, m_Description)) {
        return false;
    }

    std::uint32_t propertyCount{};
    if (!ReadScalar(input, offset, propertyCount)) {
        return false;
    }
    m_AddonProperties.reserve(propertyCount);
    for (std::uint32_t propertyIndex = 0; propertyIndex < propertyCount; ++propertyIndex) {
        std::int32_t type{};
        std::int32_t enabled{};
        std::int32_t implicit{};
        std::uint32_t valueCount{};
        if (!ReadScalar(input, offset, type) || !ReadScalar(input, offset, enabled) ||
            !ReadScalar(input, offset, implicit) || !ReadScalar(input, offset, valueCount)) {
            return false;
        }
        AddonProperty property;
        property.type = static_cast<CGoodsBaseProperties::AddonType>(type);
        property.enabled = enabled != 0;
        property.implicitAttribute = implicit != 0;
        property.values.reserve(valueCount);
        for (std::uint32_t valueIndex = 0; valueIndex < valueCount; ++valueIndex) {
            AddonValue value;
            if (!ReadScalar(input, offset, value.id) || !ReadScalar(input, offset, value.baseValue) ||
                !ReadScalar(input, offset, value.modifier)) {
                return false;
            }
            property.values.push_back(value);
        }
        m_AddonProperties.push_back(std::move(property));
    }
    return true;
}

void CGoods::Release() noexcept
{
    m_BasePropertiesIndex = 0;
    m_Amount = 0;
    m_Price = 0;
    m_Description.clear();
    m_AddonProperties.clear();
}

std::uint32_t CGoods::GetWeight(const CGoodsBaseProperties* properties) const noexcept
{
    return properties == nullptr ? 0U : properties->GetWeight() * m_Amount;
}

std::uint32_t CGoods::GetMaxStackNumber(const CGoodsBaseProperties* properties) const noexcept
{
    if (properties == nullptr ||
        (properties->GetGoodsType() != CGoodsBaseProperties::GoodsType::Useless &&
         properties->GetGoodsType() != CGoodsBaseProperties::GoodsType::Consumable)) {
        return 1;
    }
    const auto values = properties->GetAddonPropertyValues(
        CGoodsBaseProperties::AddonType::GoodsStackingLimit);
    const auto found = std::ranges::find(values, 1U, &CGoodsBaseProperties::AddonValue::id);
    return found == values.end() ? 1U : static_cast<std::uint32_t>(found->baseValue);
}

std::int32_t CGoods::GetAddonPropertyValue(CGoodsBaseProperties::AddonType type,
                                            std::uint32_t id) const noexcept
{
    const auto property = std::ranges::find(m_AddonProperties, type, &AddonProperty::type);
    if (property == m_AddonProperties.end()) {
        return 0;
    }
    const auto value = std::ranges::find(property->values, id, &AddonValue::id);
    return value == property->values.end() ? 0 : value->baseValue + value->modifier;
}

bool CGoods::HasAddonProperty(CGoodsBaseProperties::AddonType type) const noexcept
{
    return std::ranges::find(m_AddonProperties, type, &AddonProperty::type) !=
           m_AddonProperties.end();
}

void CGoods::SetGoodsDescription(std::string_view value)
{
    m_Description.assign(value.substr(0, value.find('\0')));
}

bool CGoods::CloneInto(CGoods& destination) const
{
    std::vector<std::uint8_t> snapshot;
    if (!AddToByteArray(snapshot, true)) {
        return false;
    }
    std::size_t offset{};
    return destination.DecordFromByteArray(snapshot, offset, true) && offset == snapshot.size();
}
