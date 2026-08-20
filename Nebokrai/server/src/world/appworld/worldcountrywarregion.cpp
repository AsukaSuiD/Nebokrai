#include "worldcountrywarregion.h"

#include <limits>

namespace
{
void AppendString(std::vector<std::uint8_t>& output, std::string_view value)
{
    value = value.substr(0, value.find('\0'));
    output.insert(output.end(), value.begin(), value.end()); output.push_back(0);
}
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
