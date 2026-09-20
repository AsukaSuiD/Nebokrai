#include "worldcityregion.h"

#include <cstring>
#include <limits>
#include <sstream>

void CWorldCityRegion::SetDefenceSetup(const Setup& setup) noexcept
{
    m_DefenceSetup = {setup.returnRegionId, setup.returnPoint.left, setup.returnPoint.top,
                      setup.returnPoint.right, setup.returnPoint.bottom,
                      setup.recallWhenLost, setup.moveMonsterWhenRefresh, setup.use};
}

bool CWorldCityRegion::LoadCitySetup(
    const std::string_view text,
    const std::function<std::string(std::string_view)>& resolveName)
{
    std::istringstream input{std::string(text)};
    std::vector<Build> gates;
    std::string token;
    while (input >> token) {
        if (token == "<end>") break;
        if (token != "#") continue;
        Build gate;
        std::int32_t index{};
        std::string stringId;
        if (!(input >> index >> stringId)) return false;
        std::memcpy(gate.header.data(), &index, sizeof(index));
        for (std::size_t field = 1; field < 11; ++field) {
            std::int32_t value{}; if (!(input >> value)) return false;
            std::memcpy(gate.header.data() + field * 4, &value, 4);
        }
        if (!(input >> gate.script)) return false;
        gate.name = resolveName(stringId);
        gates.push_back(std::move(gate));
    }
    std::array<std::optional<std::int32_t>, 8> defence;
    bool foundDefence{};
    while (input >> token) {
        if (token == "<end>") break;
        if (token != "#") continue;
        foundDefence = true;
        for (std::size_t index{}; index < 5; ++index) {
            std::int32_t value{}; if (!(input >> value)) return false;
            defence[index] = value;
        }
    }
    if (!foundDefence) return false;
    // В исходном EXE эти три DWORD оставались неинициализированными.
    for (std::size_t index = 5; index < defence.size(); ++index) defence[index] = 0;
    m_Gates = std::move(gates);
    m_DefenceSetup = defence;
    return true;
}

bool CWorldCityRegion::AddToByteArray(std::vector<std::uint8_t>& output,
                                      bool includeChild) const
{
    if (m_Gates.size() > std::numeric_limits<std::int32_t>::max() ||
        !CWorldWarRegion::AddToByteArray(output, includeChild)) return false;
    for (const auto& value : m_DefenceSetup) if (!value) return false;
    const auto append = [&](const auto& value) {
        const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
        output.insert(output.end(), bytes, bytes + sizeof(value));
    };
    for (const auto& value : m_DefenceSetup) append(*value);
    append(static_cast<std::int32_t>(m_Gates.size()));
    for (const auto& gate : m_Gates) {
        output.insert(output.end(), gate.header.begin(), gate.header.end());
        for (std::string_view value : {std::string_view{gate.name}, std::string_view{gate.script}}) {
            value = value.substr(0, value.find('\0'));
            output.insert(output.end(), value.begin(), value.end()); output.push_back(0);
        }
    }
    return true;
}
