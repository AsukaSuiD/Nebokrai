#include "shape.h"

#include <array>
#include <cmath>
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

std::int32_t LegacyTile(float value) noexcept
{
    if (!std::isfinite(value) || value < static_cast<float>(std::numeric_limits<std::int32_t>::min()) ||
        value >= 2147483648.0F) {
        return std::numeric_limits<std::int32_t>::min();
    }
    return static_cast<std::int32_t>(std::trunc(value));
}
}

bool CShape::AddToByteArray(std::vector<std::uint8_t>& output, bool includeChild) const
{
    return CBaseObject::AddToByteArray(output, includeChild) && AddShapeToByteArray(output);
}

bool CShape::DecordFromByteArray(std::span<const std::uint8_t> input,
                                 std::size_t& offset,
                                 bool includeChild)
{
    return CBaseObject::DecordFromByteArray(input, offset, includeChild) &&
           DecordShapeFromByteArray(input, offset);
}

bool CShape::AddShapeToByteArray(std::vector<std::uint8_t>& output) const
{
    if (m_ExId.IsInvalided()) {
        output.push_back(0);
    } else {
        output.push_back(16);
        const auto bytes = m_ExId.Bytes();
        output.insert(output.end(), bytes.begin(), bytes.end());
    }
    AppendScalar(output, m_RegionId);
    AppendScalar(output, m_PosX);
    AppendScalar(output, m_PosY);
    AppendScalar(output, m_Direction);
    AppendScalar(output, m_Position);
    AppendScalar(output, m_Speed);
    AppendScalar(output, m_State);
    AppendScalar(output, m_Action);
    return true;
}

bool CShape::DecordShapeFromByteArray(std::span<const std::uint8_t> input,
                                      std::size_t& offset)
{
    std::uint8_t marker{};
    if (!ReadScalar(input, offset, marker)) {
        return false;
    }
    if (marker == 0) {
        m_ExId = CGUID::GUID_INVALID;
    } else {
        if (offset > input.size() || input.size() - offset < 16) {
            return false;
        }
        std::array<std::uint8_t, 16> bytes{};
        std::memcpy(bytes.data(), input.data() + offset, bytes.size());
        offset += bytes.size();
        m_ExId = CGUID::FromLegacyBytes(bytes);
    }

    std::int32_t serializedPosition{};
    if (!ReadScalar(input, offset, m_RegionId) || !ReadScalar(input, offset, m_PosX) ||
        !ReadScalar(input, offset, m_PosY) || !ReadScalar(input, offset, m_Direction) ||
        !ReadScalar(input, offset, serializedPosition) || !ReadScalar(input, offset, m_Speed) ||
        !ReadScalar(input, offset, m_State) || !ReadScalar(input, offset, m_Action)) {
        return false;
    }
    (void)serializedPosition;
    m_Position = 0;
    return true;
}

std::int32_t CShape::GetTileX() const noexcept
{
    return LegacyTile(m_PosX);
}

std::int32_t CShape::GetTileY() const noexcept
{
    return LegacyTile(m_PosY);
}

void CShape::SetTileXY(std::int32_t tileX, std::int32_t tileY) noexcept
{
    m_PosX = static_cast<float>(tileX) + 0.5F;
    m_PosY = static_cast<float>(tileY) + 0.5F;
}
