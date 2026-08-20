#include "region.h"

#include <algorithm>
#include <cstring>
#include <limits>
#include <type_traits>

namespace
{
constexpr std::array<std::uint8_t, 7> kHeader{'C', 'L', 'S', '-', 'R', 'G', 'N'};

template <class T>
void Append(std::vector<std::uint8_t>& output, const T& value)
{
    static_assert(std::is_trivially_copyable_v<T>);
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
    output.insert(output.end(), bytes, bytes + sizeof(T));
}

template <class T>
bool Read(std::span<const std::uint8_t> input, std::size_t& offset, T& value)
{
    static_assert(std::is_trivially_copyable_v<T>);
    if (offset > input.size() || input.size() - offset < sizeof(T)) {
        return false;
    }
    std::memcpy(&value, input.data() + offset, sizeof(T));
    offset += sizeof(T);
    return true;
}
}

CRegion::CRegion()
{
    SetType(200);
}

bool CRegion::LoadResource(std::string_view path, std::span<const std::uint8_t> source)
{
    if (source.size() < kHeader.size() ||
        !std::equal(kHeader.begin(), kHeader.end(), source.begin())) {
        return false;
    }
    std::size_t offset = kHeader.size();
    std::int32_t version{};
    if (!Read(source, offset, version) || version != 1 ||
        !Read(source, offset, m_RegionType) || !Read(source, offset, m_Width) ||
        !Read(source, offset, m_Height) || m_Width <= 0 || m_Height <= 0) {
        return false;
    }
    const auto count64 = static_cast<std::uint64_t>(m_Width) * static_cast<std::uint64_t>(m_Height);
    if (count64 > std::numeric_limits<std::size_t>::max() ||
        count64 > (source.size() - offset) / sizeof(Cell)) {
        return false;
    }
    m_FileName.assign(path);
    m_Cells.resize(static_cast<std::size_t>(count64));
    std::memcpy(m_Cells.data(), source.data() + offset, m_Cells.size() * sizeof(Cell));
    offset += m_Cells.size() * sizeof(Cell);

    std::int32_t switchCount{};
    if (!Read(source, offset, switchCount) || switchCount < 0 ||
        static_cast<std::size_t>(switchCount) > (source.size() - offset) / sizeof(Switch)) {
        return false;
    }
    m_Switches.resize(static_cast<std::size_t>(switchCount));
    std::memcpy(m_Switches.data(), source.data() + offset, m_Switches.size() * sizeof(Switch));
    return true;
}

bool CRegion::New()
{
    if (m_Width < 0 || m_Height < 0) {
        return false;
    }
    const auto count64 = static_cast<std::uint64_t>(m_Width) * static_cast<std::uint64_t>(m_Height);
    if (count64 > std::numeric_limits<std::size_t>::max()) {
        return false;
    }
    m_Cells.assign(static_cast<std::size_t>(count64), Cell{});
    m_Switches.clear();
    return true;
}

bool CRegion::AddToByteArray(std::vector<std::uint8_t>& output, bool includeChild) const
{
    if (!m_Country || !m_Notify || m_Switches.size() > std::numeric_limits<std::int32_t>::max() ||
        !CBaseObject::AddToByteArray(output, includeChild)) {
        return false;
    }
    Append(output, m_RegionType);
    Append(output, m_ResourceId);
    Append(output, m_ExpScale);
    Append(output, m_Width);
    Append(output, m_Height);
    Append(output, *m_Country);
    Append(output, *m_Notify);
    if (!m_Cells.empty()) {
        output.insert(output.end(), reinterpret_cast<const std::uint8_t*>(m_Cells.data()),
                      reinterpret_cast<const std::uint8_t*>(m_Cells.data()) + m_Cells.size() * sizeof(Cell));
    }
    Append(output, static_cast<std::int32_t>(m_Switches.size()));
    if (!m_Switches.empty()) {
        output.insert(output.end(), reinterpret_cast<const std::uint8_t*>(m_Switches.data()),
                      reinterpret_cast<const std::uint8_t*>(m_Switches.data()) + m_Switches.size() * sizeof(Switch));
    }
    return true;
}

bool CRegion::DecordFromByteArray(std::span<const std::uint8_t> input,
                                  std::size_t& offset,
                                  bool includeChild)
{
    if (!CBaseObject::DecordFromByteArray(input, offset, includeChild) ||
        !Read(input, offset, m_RegionType) || !Read(input, offset, m_ResourceId) ||
        !Read(input, offset, m_ExpScale) || !Read(input, offset, m_Width) ||
        !Read(input, offset, m_Height)) {
        return false;
    }
    std::uint8_t country{};
    std::int32_t notify{};
    if (!Read(input, offset, country) || !Read(input, offset, notify) || m_Width < 0 || m_Height < 0) {
        return false;
    }
    m_Country = country;
    m_Notify = notify;
    const auto count = static_cast<std::uint64_t>(m_Width) * static_cast<std::uint64_t>(m_Height);
    if (count > (input.size() - offset) / sizeof(Cell)) {
        return false;
    }
    m_Cells.resize(static_cast<std::size_t>(count));
    std::memcpy(m_Cells.data(), input.data() + offset, m_Cells.size() * sizeof(Cell));
    offset += m_Cells.size() * sizeof(Cell);
    std::int32_t switchCount{};
    if (!Read(input, offset, switchCount) || switchCount < 0 ||
        static_cast<std::size_t>(switchCount) > (input.size() - offset) / sizeof(Switch)) {
        return false;
    }
    m_Switches.resize(static_cast<std::size_t>(switchCount));
    std::memcpy(m_Switches.data(), input.data() + offset, m_Switches.size() * sizeof(Switch));
    offset += m_Switches.size() * sizeof(Switch);
    return true;
}

bool CRegion::GetRandomPosInRange(std::int32_t& x,
                                  std::int32_t& y,
                                  std::int32_t startX,
                                  std::int32_t startY,
                                  std::int32_t width,
                                  std::int32_t height,
                                  const std::function<std::int32_t(std::int32_t)>& random) const
{
    if (!random || width <= 0 || height <= 0) {
        return false;
    }
    for (int attempt = 0; attempt < 1000; ++attempt) {
        const std::int32_t candidateX = startX + random(width);
        const std::int32_t candidateY = startY + random(height);
        if (IsWalkable(candidateX, candidateY)) {
            x = candidateX;
            y = candidateY;
            return true;
        }
    }
    for (std::int32_t candidateX = std::max(0, startX - 10);
         candidateX < std::min(m_Width, startX + width + 10); ++candidateX) {
        for (std::int32_t candidateY = std::max(0, startY - 10);
             candidateY < std::min(m_Height, startY + height + 10); ++candidateY) {
            if (IsWalkable(candidateX, candidateY)) {
                x = candidateX;
                y = candidateY;
                return true;
            }
        }
    }
    x = startX;
    y = startY;
    return false;
}

bool CRegion::IsWalkable(std::int32_t x, std::int32_t y) const noexcept
{
    if (x < 0 || y < 0 || x >= m_Width || y >= m_Height) {
        return false;
    }
    const Cell& cell = m_Cells[static_cast<std::size_t>(x) * static_cast<std::size_t>(m_Height) +
                               static_cast<std::size_t>(y)];
    const std::uint16_t upper = static_cast<std::uint16_t>(cell[2]) |
                                static_cast<std::uint16_t>(cell[3] << 8U);
    return (cell[0] & 7U) == 0 && upper == 0;
}
