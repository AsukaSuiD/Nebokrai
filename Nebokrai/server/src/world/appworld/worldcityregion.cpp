#include "worldcityregion.h"

#include <limits>

bool CWorldCityRegion::AddToByteArray(std::vector<std::uint8_t>& output,
                                      bool includeChild) const
{
    if (!m_DefenceSetup || m_Gates.size() > std::numeric_limits<std::int32_t>::max() ||
        !CWorldWarRegion::AddToByteArray(output, includeChild)) return false;
    const Setup& setup = *m_DefenceSetup;
    const auto append = [&](const auto& value) {
        const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
        output.insert(output.end(), bytes, bytes + sizeof(value));
    };
    append(setup.returnRegionId); append(setup.returnPoint);
    append(setup.recallWhenLost); append(setup.moveMonsterWhenRefresh); append(setup.use);
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
