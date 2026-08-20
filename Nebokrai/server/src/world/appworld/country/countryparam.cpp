#include "countryparam.h"

#include <limits>
#include <sstream>
#include <type_traits>

namespace {
template <class T> void Append(std::vector<std::uint8_t>& output, T value) {
    static_assert(std::is_trivially_copyable_v<T>);
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
    output.insert(output.end(), bytes, bytes + sizeof(T));
}
template <class Collection> bool AppendCount(std::vector<std::uint8_t>& out, const Collection& c) {
    if (c.size() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max())) return false;
    Append(out, static_cast<std::int32_t>(c.size())); return true;
}
bool Seek(std::istringstream& stream, std::string_view marker) {
    std::string token; while (stream >> token) { if (token == marker) return true; if (token == "<end>") return false; } return false;
}
bool ReadRect(std::istringstream& stream, CCountryParam::Rect& rect) {
    return static_cast<bool>(stream >> rect.left >> rect.top >> rect.right >> rect.bottom);
}
}

bool CCountryParam::Load(std::optional<std::string_view> source)
{
    m_StartRegions.clear(); m_StartRects.clear(); m_StartDirections.clear();
    m_MainRegions.clear(); m_MainRects.clear(); m_MainDirections.clear();
    if (!source) return true;
    std::istringstream stream{std::string(*source)};
    std::string label;
    for (auto& parameter : m_Parameters) {
        std::int32_t value{}; if (!(stream >> label >> value)) return false; parameter = value;
    }
    while (Seek(stream, "*")) {
        std::int32_t rawCountry{}, startRegion{}, startDirection{}, mainRegion{}, mainDirection{}; Rect start{}, main{};
        if (!(stream >> rawCountry >> startRegion) || !ReadRect(stream, start) || !(stream >> startDirection >> mainRegion) || !ReadRect(stream, main) || !(stream >> mainDirection)) return false;
        const auto country = static_cast<std::uint8_t>(rawCountry);
        m_StartRegions[country] = startRegion; m_StartRects[country] = start; m_StartDirections[country] = startDirection;
        m_MainRegions[country] = mainRegion; m_MainRects[country] = main; m_MainDirections[country] = mainDirection;
    }
    stream.clear();
    while (Seek(stream, "#")) {
        std::int32_t level{}; TechLevel tech; if (!(stream >> level >> tech.experience >> tech.power)) return false; m_TechLevels[level] = tech;
    }
    stream.clear();
    while (Seek(stream, "+")) {
        std::int32_t rawCountry{}; Rect rect; if (!(stream >> rawCountry) || !ReadRect(stream, rect)) return false; m_ExileRects[static_cast<std::uint8_t>(rawCountry)] = rect;
    }
    return true;
}

CCountryParam::ReturnPoint CCountryParam::MainReturnPoint(std::uint8_t country)
{
    return {m_MainRegions[country], m_MainRects[country], m_MainDirections[country]};
}

std::optional<std::int32_t> CCountryParam::Parameter(std::size_t index) const noexcept
{
    return index < m_Parameters.size() ? m_Parameters[index] : std::nullopt;
}

bool CCountryParam::AddToByteArray(std::vector<std::uint8_t>& output) const
{
    for (const auto value : m_Parameters) { if (!value) return false; Append(output, *value); }
    if (!AppendCount(output, m_MainRegions)) return false;
    for (const auto [country, region] : m_MainRegions) { Append(output, country); Append(output, region); }
    if (!AppendCount(output, m_MainRects)) return false;
    for (const auto [country, rect] : m_MainRects) { Append(output, country); Append(output, rect.left); Append(output, rect.top); Append(output, rect.right); Append(output, rect.bottom); }
    if (!AppendCount(output, m_MainDirections)) return false;
    for (const auto [country, direction] : m_MainDirections) { Append(output, country); Append(output, direction); }
    if (!AppendCount(output, m_TechLevels)) return false;
    for (const auto [level, tech] : m_TechLevels) { Append(output, level); Append(output, tech.power); Append(output, tech.experience); }
    if (!AppendCount(output, m_ExileRects)) return false;
    for (const auto [country, rect] : m_ExileRects) { Append(output, country); Append(output, rect.left); Append(output, rect.top); Append(output, rect.right); Append(output, rect.bottom); }
    return true;
}
