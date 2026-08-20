#include "cbattlefairyproperty.h"

#include <algorithm>

const BattleFairyLevel* CBattleFairyProperty::FindLevel(const std::int32_t level) const noexcept
{
    const auto found = std::ranges::find(m_Levels, level, &BattleFairyLevel::equipmentLevel);
    return found == m_Levels.end() ? nullptr : &*found;
}
