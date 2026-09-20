#include "hitlevelsetup.h"

#include <fstream>
#include <limits>
#include <string>

namespace
{
template<class T> void Append(std::vector<std::uint8_t>& output, const T& value)
{
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
    output.insert(output.end(), bytes, bytes + sizeof(value));
}
}

bool CHitLevelSetup::Load(const std::filesystem::path& path)
{
    std::ifstream input(path);
    if (!input) return false;
    std::vector<Entry> entries;
    std::string token;
    while (input >> token) {
        if (token != "*") continue;
        Entry entry;
        if (!(input >> entry.level >> entry.hit >> entry.experience)) return false;
        entries.push_back(entry);
    }
    if (entries.empty()) return false;
    m_Entries = std::move(entries);
    return true;
}

bool CHitLevelSetup::Serialize(std::vector<std::uint8_t>& output) const
{
    if (m_Entries.size() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()))
        return false;
    Append(output, static_cast<std::int32_t>(m_Entries.size()));
    for (const Entry& entry : m_Entries) Append(output, entry);
    return true;
}
