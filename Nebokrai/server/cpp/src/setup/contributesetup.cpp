#include "contributesetup.h"

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
void AppendString(std::vector<std::uint8_t>& output, const std::string& value)
{
    output.insert(output.end(), value.begin(), value.end());
    output.push_back(0);
}
}

bool CContributeSetup::Load(const std::filesystem::path& path)
{
    std::ifstream input(path);
    if (!input) return false;
    std::array<std::int32_t, 11> parameters{};
    std::string label;
    for (auto& value : parameters) if (!(input >> label >> value)) return false;

    std::vector<Item> items;
    std::string token;
    while (input >> token) {
        if (token != "#") continue;
        Item item;
        if (!(input >> item.low >> item.high >> item.name >> item.count)) return false;
        items.push_back(std::move(item));
    }
    if (items.empty()) return false;
    m_Parameters = parameters;
    m_Items = std::move(items);
    return true;
}

bool CContributeSetup::Serialize(std::vector<std::uint8_t>& output) const
{
    if (m_Items.size() > static_cast<std::size_t>(std::numeric_limits<std::int32_t>::max()))
        return false;
    for (const auto value : m_Parameters) Append(output, value);
    Append(output, static_cast<std::int32_t>(m_Items.size()));
    for (const Item& item : m_Items) {
        Append(output, item.low); Append(output, item.high); Append(output, item.count);
        AppendString(output, item.name);
    }
    return true;
}
