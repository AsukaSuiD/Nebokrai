#include "honorelimilateconfig.h"

#include <fstream>
#include <string>

namespace
{
void Append(std::vector<std::uint8_t>& output, const std::int32_t value)
{
    const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
    output.insert(output.end(), bytes, bytes + sizeof(value));
}
}

bool HonorElimilateConfig::Load(const std::filesystem::path& path)
{
    std::ifstream input(path);
    std::string label;
    std::int32_t difference{}, minimum{};
    if (!input || !(input >> label >> difference >> label >> minimum)) return false;
    m_LevelDifference = difference;
    m_MinimumLevel = minimum;
    return true;
}

void HonorElimilateConfig::Serialize(std::vector<std::uint8_t>& output) const
{
    Append(output, m_LevelDifference);
    Append(output, m_MinimumLevel);
}
