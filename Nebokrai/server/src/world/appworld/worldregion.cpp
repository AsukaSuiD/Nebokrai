#include "worldregion.h"

#include <algorithm>
#include <charconv>
#include <cstring>
#include <limits>
#include <string_view>

namespace
{
template <class T>
void Append(std::vector<std::uint8_t>& output, const T& value)
{
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
    output.insert(output.end(), bytes, bytes + sizeof(T));
}

void AppendString(std::vector<std::uint8_t>& output, std::string_view value)
{
    value = value.substr(0, value.find('\0'));
    output.insert(output.end(), value.begin(), value.end());
    output.push_back(0);
}

bool AppendCount(std::vector<std::uint8_t>& output, std::size_t count)
{
    if (count > std::numeric_limits<std::uint32_t>::max()) {
        return false;
    }
    Append(output, static_cast<std::uint32_t>(count));
    return true;
}

void AppendParam(std::vector<std::uint8_t>& output, const RegionParam& param)
{
    Append(output, param.id); Append(output, param.maxTaxRate); Append(output, param.currentTaxRate);
    Append(output, param.totalTax); Append(output, param.todayTotalTax);
    Append(output, param.superiorRegionId); Append(output, param.turnInTaxRate);
    Append(output, param.ownedFactionId); Append(output, param.ownedUnionId);
}

std::vector<std::string_view> Tokens(std::string_view text)
{
    std::vector<std::string_view> result;
    while (!text.empty()) {
        const auto first = text.find_first_not_of(" \t\r\n");
        if (first == std::string_view::npos) break;
        text.remove_prefix(first);
        const auto end = text.find_first_of(" \t\r\n");
        result.push_back(text.substr(0, end));
        if (end == std::string_view::npos) break;
        text.remove_prefix(end);
    }
    return result;
}
}

bool CWorldRegion::AddToByteArray(std::vector<std::uint8_t>& output, bool includeChild) const
{
    if (!m_Setup || !CRegion::AddToByteArray(output, includeChild)) {
        return false;
    }
    Append(output, static_cast<std::int32_t>(m_WarRegionType));
    output.push_back(static_cast<std::uint8_t>(m_NoPk));
    output.push_back(static_cast<std::uint8_t>(m_NoContribute));
    if (!AppendCount(output, m_Npcs.size())) return false;
    for (const auto& npc : m_Npcs) {
        output.insert(output.end(), npc.header.begin(), npc.header.end());
        AppendString(output, npc.name); AppendString(output, npc.script);
    }
    if (!AppendCount(output, m_Monsters.size())) return false;
    for (const auto& monster : m_Monsters) {
        output.insert(output.end(), monster.header.begin(), monster.header.end());
        if (!AppendCount(output, monster.variants.size())) return false;
        for (const auto& variant : monster.variants) {
            output.insert(output.end(), variant.legacyPrefix.begin(), variant.legacyPrefix.end());
            AppendString(output, variant.name); AppendString(output, variant.script);
        }
    }
    if (!AppendCount(output, m_Weather.size())) return false;
    for (const auto& time : m_Weather) {
        Append(output, time.time);
        if (!AppendCount(output, time.options.size())) return false;
        for (const auto& option : time.options) {
            Append(output, option.cumulativeOdds);
            if (!AppendCount(output, option.weather.size())) return false;
            for (const auto& weather : option.weather) {
                Append(output, weather.index); Append(output, weather.fogColor);
            }
        }
    }
    const Setup& setup = *m_Setup;
    Append(output, setup.returnRegionId); Append(output, setup.returnPoint);
    Append(output, setup.recallWhenLost); Append(output, setup.moveMonsterWhenRefresh);
    Append(output, setup.use);
    if (!AppendCount(output, m_ForbiddenMakeGoods.size())) return false;
    for (const auto& name : m_ForbiddenMakeGoods) AppendString(output, name);
    AppendParam(output, m_Param);
    return true;
}

bool CWorldRegion::AddToByteArrayForProxy(std::vector<std::uint8_t>& output,
                                           bool includeChild) const
{
    if (!GetCountry() || !CBaseObject::AddToByteArray(output, includeChild)) {
        return false;
    }
    output.push_back(*GetCountry());
    Append(output, static_cast<std::int32_t>(m_WarRegionType));
    AppendParam(output, m_Param);
    return true;
}

bool CWorldRegion::DecordRegionParamFromByteArray(std::span<const std::uint8_t> input,
                                                  std::size_t& offset)
{
    if (offset > input.size() || input.size() - offset < sizeof(RegionParam)) {
        return false;
    }
    RegionParam incoming{};
    std::memcpy(&incoming, input.data() + offset, sizeof(incoming));
    offset += sizeof(incoming);
    m_Param.currentTaxRate = incoming.currentTaxRate;
    m_Param.totalTax = incoming.totalTax;
    m_Param.todayTotalTax = incoming.todayTotalTax;
    m_Param.ownedFactionId = incoming.ownedFactionId;
    return true;
}

bool CWorldRegion::LoadSetup(std::string_view text)
{
    const auto tokens = Tokens(text);
    std::size_t index{};
    Setup setup{};
    bool foundSetup = false;
    for (; index < tokens.size() && tokens[index] != "<end>"; ++index) {
        if (tokens[index] != "*") continue;
        if (tokens.size() - index - 1 < 8) return false;
        std::int32_t* fields[] = {&setup.returnRegionId, &setup.returnPoint.left,
            &setup.returnPoint.top, &setup.returnPoint.right, &setup.returnPoint.bottom,
            &setup.recallWhenLost, &setup.moveMonsterWhenRefresh, &setup.use};
        for (auto* field : fields) {
            const auto token = tokens[++index];
            const auto [end, error] = std::from_chars(token.data(), token.data() + token.size(), *field);
            if (error != std::errc{} || end != token.data() + token.size()) return false;
        }
        foundSetup = true;
    }
    if (!foundSetup) return false;
    m_Setup = setup;
    m_ForbiddenMakeGoods.clear();
    for (++index; index < tokens.size() && tokens[index] != "<end>"; ++index) {
        if (tokens[index] == "#" && index + 1 < tokens.size()) {
            m_ForbiddenMakeGoods.emplace_back(tokens[++index]);
        }
    }
    return true;
}

void CWorldRegion::SetParamFromDB(std::int32_t ownedFactionId,
                                  std::int32_t ownedUnionId,
                                  std::int32_t currentTaxRate,
                                  std::int32_t todayTotalTax,
                                  std::int32_t totalTax) noexcept
{
    m_Param.ownedFactionId = ownedFactionId;
    m_Param.ownedUnionId = ownedUnionId;
    m_Param.currentTaxRate = std::min(currentTaxRate, m_Param.maxTaxRate);
    m_Param.todayTotalTax = static_cast<std::uint32_t>(todayTotalTax);
    m_Param.totalTax = static_cast<std::uint32_t>(totalTax);
}

void CWorldRegion::SetParamFromGame(std::int32_t currentTaxRate,
                                    std::uint32_t todayTotalTax,
                                    std::uint32_t totalTax) noexcept
{
    m_Param.currentTaxRate = currentTaxRate;
    m_Param.todayTotalTax = todayTotalTax;
    m_Param.totalTax = totalTax;
}

void CWorldRegion::SetOwnedCityOrg(std::int32_t factionId, std::int32_t unionId) noexcept
{
    m_Param.ownedFactionId = factionId;
    m_Param.ownedUnionId = unionId;
}
