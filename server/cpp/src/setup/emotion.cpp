#include "emotion.h"

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

bool CEmotion::Load(const std::filesystem::path& path)
{
    std::ifstream input(path);
    if (!input) return false;
    std::map<std::int32_t, std::int32_t> emotions;
    std::string token;
    while (input >> token) {
        if (token != "*") continue;
        std::int32_t id{}, value{};
        if (!(input >> id >> value)) return false;
        emotions.insert_or_assign(id, value);
        std::getline(input, token);
    }
    if (emotions.empty()) return false;
    m_Emotions = std::move(emotions);
    return true;
}

bool CEmotion::Serialize(std::vector<std::uint8_t>& output) const
{
    if (m_Emotions.size() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()))
        return false;
    Append(output, static_cast<std::int32_t>(m_Emotions.size()));
    for (const auto& [id, value] : m_Emotions) { Append(output, id); Append(output, value); }
    return true;
}

std::int32_t CEmotion::Repeated(const std::int32_t id) const noexcept
{
    const auto found = m_Emotions.find(id);
    return found == m_Emotions.end() ? 0 : found->second;
}
