#include "organizingparam.h"

#include <charconv>
#include <sstream>
#include <string>
#include <vector>

namespace {
bool ParseInt(const std::string_view token, std::int32_t& value)
{
    const auto [end, error] = std::from_chars(token.data(), token.data() + token.size(), value);
    return error == std::errc{} && end == token.data() + token.size();
}

bool ParseTime(const std::string_view token, OrganizingTime& value)
{
    std::array<std::int32_t, 6> fields{};
    std::size_t offset = 0;
    for (auto& field : fields) {
        const auto delimiter = token.find(':', offset);
        const auto part = token.substr(offset, delimiter == std::string_view::npos ? delimiter : delimiter - offset);
        if (!ParseInt(part, field) || field < 0 || field > 65535) return false;
        if (delimiter == std::string_view::npos) {
            offset = token.size();
            if (&field != &fields.back()) return false;
        } else offset = delimiter + 1;
    }
    if (offset != token.size()) return false;
    value.year = static_cast<std::uint16_t>(fields[0]);
    value.month = static_cast<std::uint16_t>(fields[1]);
    value.day = static_cast<std::uint16_t>(fields[2]);
    value.hour = static_cast<std::uint16_t>(fields[3]);
    value.minute = static_cast<std::uint16_t>(fields[4]);
    value.second = static_cast<std::uint16_t>(fields[5]);
    return true;
}

std::vector<std::string> Tokens(const std::string& line)
{
    std::istringstream input(line);
    std::vector<std::string> values;
    for (std::string value; input >> value;) values.push_back(std::move(value));
    return values;
}
}

bool COrganizingParam::Load(const std::string_view source)
{
    COrganizingParam next;
    std::istringstream input{std::string(source)};
    std::string line;
    std::size_t header = 0;
    std::size_t levels = 0;
    while (std::getline(input, line)) {
        const auto tokens = Tokens(line);
        if (tokens.empty() || tokens.front().starts_with('-')) continue;
        if (tokens.front() == "*") {
            if (tokens.size() != 7 || levels >= next.m_Levels.size()) return false;
            std::array<std::int32_t, 5> numbers{};
            if (!ParseInt(tokens[1], numbers[0]) || !ParseInt(tokens[2], numbers[1]) ||
                !ParseInt(tokens[3], numbers[2]) || !ParseInt(tokens[4], numbers[3]) ||
                !ParseInt(tokens[5], numbers[4]) || numbers[0] != static_cast<std::int32_t>(levels + 1)) return false;
            next.m_Levels[levels++] = OrganizingLevelParam{
                numbers[1], numbers[2], numbers[3], numbers[4], tokens[6] == "0" ? std::string{} : tokens[6]};
            continue;
        }
        if (header >= 12) continue;
        const auto& values = tokens;
        auto number = [&](const std::size_t index, std::int32_t& destination) {
            return index < values.size() && ParseInt(values[index], destination);
        };
        switch (header++) {
        case 0: if (values.size() < 3 || !number(1, next.uploadIconMinimumLevel) || !number(2, next.uploadIconIntervalMinutes)) return false; break;
        case 1: if (!number(1, next.pronounceMinimumLevel)) return false; break;
        case 2: if (!number(1, next.leaveWordMinimumLevel)) return false; break;
        case 3: if (!number(1, next.rightsMinimumLevel)) return false; break;
        case 4: if (!number(1, next.createUnionMinimumLevel)) return false; break;
        case 5: if (!number(1, next.attackVillageMinimumLevel)) return false; break;
        case 6: if (!number(1, next.attackCityMinimumLevel)) return false; break;
        case 7: if (values.size() < 3 || !number(1, next.disbandMinimumMembers) || !number(2, next.disbandMinutes)) return false; break;
        case 8: if (!number(1, next.maximumContributors)) return false; break;
        case 9:
            if (values.size() < 4 || !number(1, next.createFactionPlayerLevel) || !number(3, next.createFactionMoney)) return false;
            next.createFactionGoods = values[2] == "0" ? std::string{} : values[2];
            break;
        case 10: if (values.size() < 2 || !ParseTime(values[1], next.taxTime)) return false; break;
        case 11:
            if (values.size() < 3 || !ParseTime(values[1], next.rankTime) || !number(2, next.playerRanksCount)) return false;
            break;
        }
    }
    if (header != 12 || levels != next.m_Levels.size()) return false;
    *this = std::move(next);
    return true;
}

const OrganizingLevelParam* COrganizingParam::GetLevel(const std::int32_t level) const noexcept
{
    if (level < 1 || level > static_cast<std::int32_t>(m_Levels.size())) return nullptr;
    const auto& value = m_Levels[static_cast<std::size_t>(level - 1)];
    return value ? &*value : nullptr;
}

std::int32_t COrganizingParam::GetMaxNumberByLevel(const std::int32_t level) const noexcept
{
    const auto* value = GetLevel(level);
    return value == nullptr ? 0 : value->maximumMembers;
}
