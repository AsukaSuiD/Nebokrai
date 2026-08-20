#include "worldwarregion.h"

#include <charconv>

void CWorldWarRegion::SetSymbols(std::int32_t total,
                                 std::int32_t win,
                                 std::int32_t victory) noexcept
{
    m_SymbolTotal = total;
    m_WinVictorySymbols = win;
    m_VictorySymbols = victory;
}

bool CWorldWarRegion::LoadWarSetup(std::string_view text)
{
    const auto marker = text.find('#');
    if (marker == std::string_view::npos) return true;
    text.remove_prefix(marker + 1);
    std::int32_t values[3]{};
    for (auto& value : values) {
        const auto first = text.find_first_not_of(" \t\r\n");
        if (first == std::string_view::npos) return false;
        text.remove_prefix(first);
        const auto end = text.find_first_of(" \t\r\n");
        const auto token = text.substr(0, end);
        const auto [parsed, error] = std::from_chars(token.data(), token.data() + token.size(), value);
        if (error != std::errc{} || parsed != token.data() + token.size()) return false;
        text.remove_prefix(end == std::string_view::npos ? text.size() : end);
    }
    SetSymbols(values[0], values[1], values[2]);
    return true;
}

bool CWorldWarRegion::AddToByteArray(std::vector<std::uint8_t>& output,
                                     bool includeChild) const
{
    if (!m_SymbolTotal || !m_WinVictorySymbols || !m_VictorySymbols ||
        !CWorldRegion::AddToByteArray(output, includeChild)) return false;
    for (const auto value : {*m_SymbolTotal, *m_WinVictorySymbols, *m_VictorySymbols}) {
        const auto* bytes = reinterpret_cast<const std::uint8_t*>(&value);
        output.insert(output.end(), bytes, bytes + sizeof(value));
    }
    return true;
}
