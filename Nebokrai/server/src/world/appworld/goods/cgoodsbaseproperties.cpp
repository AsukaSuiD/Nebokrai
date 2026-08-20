#include "cgoodsbaseproperties.h"

#include <algorithm>

std::span<const CGoodsBaseProperties::AddonValue>
CGoodsBaseProperties::GetAddonPropertyValues(AddonType type) const noexcept
{
    const auto found = std::ranges::find(m_AddonProperties, type, &AddonProperty::type);
    return found == m_AddonProperties.end() ? std::span<const AddonValue>{} : found->values;
}

std::uint32_t CGoodsBaseProperties::GetOccurProbability(AddonType type) const noexcept
{
    const auto found = std::ranges::find(m_AddonProperties, type, &AddonProperty::type);
    return found == m_AddonProperties.end() ? 0U : found->occurProbability;
}
