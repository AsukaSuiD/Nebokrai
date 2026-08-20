#include "worldcountrywarregion.h"

#include <cstring>
#include <limits>
#include <sstream>

namespace
{
void AppendString(std::vector<std::uint8_t>& output, std::string_view value)
{
    value = value.substr(0, value.find('\0'));
    output.insert(output.end(), value.begin(), value.end()); output.push_back(0);
}
}

bool WorldCountryWarRegion::LoadCountrySetup(const std::string_view text)
{
    std::istringstream input{std::string(text)};
    auto parseHeader = [&](auto& header, const std::size_t count) {
        for (std::size_t index{}; index < count; ++index) {
            std::int32_t value{};
            if (!(input >> value)) return false;
            std::memcpy(header.data() + index * sizeof(value), &value, sizeof(value));
        }
        return true;
    };
    auto parseGateSection = [&](std::vector<Gate>& output) {
        output.clear();
        std::string token;
        while (input >> token) {
            if (token == "<end>") return true;
            if (token != "#") continue;
            Gate value;
            std::int32_t first{};
            if (!(input >> first >> value.name)) return false;
            std::memcpy(value.header.data(), &first, sizeof(first));
            for (std::size_t index = 1; index < 11; ++index) {
                std::int32_t field{}; if (!(input >> field)) return false;
                std::memcpy(value.header.data() + index * 4, &field, 4);
            }
            if (!(input >> value.script)) return false;
            output.push_back(std::move(value));
        }
        return false;
    };
    auto parseFlagSection = [&](std::vector<Flag>& output) {
        output.clear();
        std::string token;
        while (input >> token) {
            if (token == "<end>") return true;
            if (token != "#") continue;
            Flag value;
            std::int32_t first{};
            if (!(input >> first >> value.name)) return false;
            std::memcpy(value.header.data(), &first, sizeof(first));
            for (std::size_t index = 1; index < 10; ++index) {
                std::int32_t field{}; if (!(input >> field)) return false;
                std::memcpy(value.header.data() + index * 4, &field, 4);
            }
            if (!(input >> value.script)) return false;
            output.push_back(std::move(value));
        }
        return false;
    };
    auto parseAreaSection = [&](std::vector<Area>& output) {
        output.clear();
        std::string token;
        while (input >> token) {
            if (token == "<end>") return true;
            if (token != "#") continue;
            Area value;
            if (!parseHeader(value.header, 5)) return false;
            output.push_back(value);
        }
        return false;
    };
    return parseGateSection(m_DefendGates) && parseGateSection(m_AttackGates) &&
           parseFlagSection(m_DefendFlags) && parseFlagSection(m_AttackFlags) &&
           parseAreaSection(m_DefendAreas) && parseAreaSection(m_AttackAreas);
}

namespace
{
template <class Record, class Body>
bool AppendSection(std::vector<std::uint8_t>& output,
                   const std::vector<Record>& records,
                   Body body)
{
    if (records.size() > std::numeric_limits<std::int32_t>::max()) return false;
    const auto count = static_cast<std::int32_t>(records.size());
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&count);
    output.insert(output.end(), bytes, bytes + sizeof(count));
    for (const auto& record : records) body(record);
    return true;
}
}

bool WorldCountryWarRegion::AddToByteArray(std::vector<std::uint8_t>& output,
                                           bool includeChild) const
{
    if (!CWorldRegion::AddToByteArray(output, includeChild)) return false;
    const auto gates = [&](const auto& records) {
        return AppendSection(output, records, [&](const Gate& record) {
            output.insert(output.end(), record.header.begin(), record.header.end());
            AppendString(output, record.name); AppendString(output, record.script);
        });
    };
    const auto flags = [&](const auto& records) {
        return AppendSection(output, records, [&](const Flag& record) {
            output.insert(output.end(), record.header.begin(), record.header.end());
            AppendString(output, record.name); AppendString(output, record.script);
        });
    };
    const auto areas = [&](const auto& records) {
        return AppendSection(output, records, [&](const Area& record) {
            output.insert(output.end(), record.header.begin(), record.header.end());
        });
    };
    return gates(m_DefendGates) && gates(m_AttackGates) &&
           flags(m_DefendFlags) && flags(m_AttackFlags) &&
           areas(m_DefendAreas) && areas(m_AttackAreas);
}
