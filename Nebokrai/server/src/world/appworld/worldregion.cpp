#include "worldregion.h"

#include <algorithm>
#include <cctype>
#include <charconv>
#include <cstring>
#include <limits>
#include <sstream>
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

template<class T>
void WriteAt(auto& destination, const std::size_t offset, const T value)
{
    std::memcpy(destination.data() + offset, &value, sizeof(value));
}

std::string NormalizeScript(std::string value)
{
    std::ranges::transform(value, value.begin(), [](unsigned char ch) {
        if (ch == '\\') ch = '/';
        return static_cast<char>(std::tolower(ch));
    });
    return value;
}

bool ParseColor(const std::string_view text, std::uint32_t& color)
{
    if (text.size() < 6) return false;
    color = 0;
    for (const char ch : text.substr(0, 6)) {
        const std::uint32_t digit = ch >= '0' && ch <= '9' ? static_cast<std::uint32_t>(ch - '0')
            : ch >= 'A' && ch <= 'F' ? static_cast<std::uint32_t>(ch - 'A' + 10)
            : ch >= 'a' && ch <= 'f' ? static_cast<std::uint32_t>(ch - 'a' + 10) : 0U;
        color = color * 16U + digit;
    }
    color |= 0xFF000000U;
    return true;
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

bool CWorldRegion::LoadNpcList(
    const std::string_view text,
    const std::function<std::string(std::string_view)>& resolveName)
{
    if (!resolveName) return false;
    const auto tokens = Tokens(text);
    std::vector<NpcRecord> loaded;
    for (std::size_t index{}; index < tokens.size(); ++index) {
        if (tokens[index] != "#") continue;
        if (tokens.size() - index - 1 < 10) return false;
        std::int32_t values[8]{};
        for (std::size_t field{}; field < 5; ++field) {
            const auto token = tokens[++index];
            const auto [end, ec] = std::from_chars(token.data(), token.data() + token.size(), values[field]);
            if (ec != std::errc{} || end != token.data() + token.size()) return false;
        }
        const std::string_view stringId = tokens[++index];
        for (std::size_t field = 5; field < 8; ++field) {
            const auto token = tokens[++index];
            const auto [end, ec] = std::from_chars(token.data(), token.data() + token.size(), values[field]);
            if (ec != std::errc{} || end != token.data() + token.size()) return false;
        }
        NpcRecord record;
        record.header[0] = static_cast<std::uint8_t>(values[0] != 0);
        WriteAt(record.header, 4, values[5]);
        WriteAt(record.header, 8, values[1]);
        WriteAt(record.header, 0x0C, values[2]);
        WriteAt(record.header, 0x10, values[3]);
        WriteAt(record.header, 0x14, values[4]);
        WriteAt(record.header, 0x18, values[6]);
        WriteAt(record.header, 0x1C, values[7]);
        record.name = resolveName(stringId);
        record.script = NormalizeScript(std::string(tokens[++index]));
        loaded.push_back(std::move(record));
    }
    m_Npcs = std::move(loaded);
    return true;
}

bool CWorldRegion::LoadMonsterList(const std::string_view text, const float countScale)
{
    const auto tokens = Tokens(text);
    std::size_t index{};
    std::vector<MonsterRecord> monsters;
    for (; index < tokens.size(); ++index) {
        if (tokens[index] == "<end>") { ++index; break; }
        if (tokens[index] != "#") continue;
        if (tokens.size() - index - 1 < 9) return false;
        std::int32_t fields[9]{};
        for (auto& field : fields) {
            const auto token = tokens[++index];
            const auto [end, ec] = std::from_chars(token.data(), token.data() + token.size(), field);
            if (ec != std::errc{} || end != token.data() + token.size()) return false;
        }
        if (fields[5] > 1) {
            fields[5] = static_cast<std::int32_t>(static_cast<float>(fields[5]) * countScale);
            if (fields[5] == 0) fields[5] = 1;
        }
        fields[6] *= 1000;
        fields[7] *= 1000;
        MonsterRecord record;
        std::memcpy(record.header.data(), fields, sizeof(fields));
        monsters.push_back(std::move(record));
    }

    MonsterRecord* current{};
    std::uint16_t cumulative{};
    for (; index < tokens.size(); ++index) {
        if (tokens[index] == "id") {
            if (++index == tokens.size()) return false;
            std::int32_t id{};
            const auto [end, ec] = std::from_chars(tokens[index].data(),
                tokens[index].data() + tokens[index].size(), id);
            if (ec != std::errc{} || end != tokens[index].data() + tokens[index].size()) return false;
            current = nullptr;
            for (auto& monster : monsters) {
                std::int32_t candidate{};
                std::memcpy(&candidate, monster.header.data(), sizeof(candidate));
                if (candidate == id) { current = &monster; break; }
            }
            cumulative = 0;
            continue;
        }
        if (tokens[index] == "<end>") { current = nullptr; continue; }
        if (tokens[index] != "#") continue;
        if (tokens.size() - index - 1 < 6) return false;
        const std::string name(tokens[++index]);
        std::uint16_t values[4]{};
        for (auto& value : values) {
            const auto token = tokens[++index];
            unsigned parsed{};
            const auto [end, ec] = std::from_chars(token.data(), token.data() + token.size(), parsed);
            if (ec != std::errc{} || end != token.data() + token.size() || parsed > 0xFFFFU) return false;
            value = static_cast<std::uint16_t>(parsed);
        }
        const std::string script = NormalizeScript(std::string(tokens[++index]));
        if (!current) continue;
        cumulative = static_cast<std::uint16_t>(cumulative + values[0]);
        if (name.size() >= 0x10) return false;
        MonsterVariant variant;
        const std::uint16_t prefix[4]{cumulative, values[1], values[2], values[3]};
        std::memcpy(variant.legacyPrefix.data(), prefix, sizeof(prefix));
        std::memcpy(variant.legacyPrefix.data() + 0x0C, name.data(), name.size());
        WriteAt(variant.legacyPrefix, 0x1C, static_cast<std::uint32_t>(name.size()));
        WriteAt(variant.legacyPrefix, 0x20, static_cast<std::uint16_t>(0x0F));
        variant.name = name;
        variant.script = script;
        current->variants.push_back(std::move(variant));
    }
    m_Monsters = std::move(monsters);
    return true;
}

bool CWorldRegion::LoadWeatherSetup(const std::optional<std::string_view> text)
{
    m_Weather.clear();
    if (!text) return true;
    const auto tokens = Tokens(*text);
    std::size_t index{};
    while (index < tokens.size()) {
        if (tokens[index++] != "time") continue;
        if (tokens.size() - index < 3) return false;
        WeatherTime time;
        auto parseI32 = [&](std::int32_t& value) {
            if (index >= tokens.size()) return false;
            const auto token = tokens[index++];
            const auto [end, ec] = std::from_chars(token.data(), token.data() + token.size(), value);
            return ec == std::errc{} && end == token.data() + token.size();
        };
        std::int32_t optionCount{};
        if (!parseI32(time.time) || tokens[index++] != "num" || !parseI32(optionCount) || optionCount < 0) return false;
        std::int32_t cumulative{};
        for (std::int32_t optionIndex{}; optionIndex < optionCount; ++optionIndex) {
            WeatherOption option;
            std::int32_t odds{}, weatherCount{};
            if (!parseI32(odds) || !parseI32(weatherCount) || weatherCount < 0) return false;
            cumulative += odds;
            option.cumulativeOdds = cumulative;
            for (std::int32_t weatherIndex{}; weatherIndex < weatherCount; ++weatherIndex) {
                Weather weather;
                if (!parseI32(weather.index)) return false;
                if (weather.index / 100 == 4) {
                    if (index >= tokens.size() || !ParseColor(tokens[index++], weather.fogColor)) return false;
                }
                option.weather.push_back(weather);
            }
            time.options.push_back(std::move(option));
        }
        m_Weather.push_back(std::move(time));
    }
    return true;
}

bool CWorldRegion::LoadTaxParam(const std::optional<std::string_view> text)
{
    if (text) {
        const auto tokens = Tokens(*text);
        const auto marker = std::ranges::find(tokens, "*");
        if (marker != tokens.end()) {
            const auto position = static_cast<std::size_t>(std::distance(tokens.begin(), marker));
            if (tokens.size() - position - 1 < 3) return false;
            std::int32_t* values[]{&m_Param.maxTaxRate, &m_Param.superiorRegionId,
                                   &m_Param.turnInTaxRate};
            for (std::size_t i{}; i < 3; ++i) {
                const auto token = tokens[position + i + 1];
                const auto [end, ec] = std::from_chars(token.data(), token.data() + token.size(), *values[i]);
                if (ec != std::errc{} || end != token.data() + token.size()) return false;
            }
        }
    }
    m_Param.id = GetID();
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
