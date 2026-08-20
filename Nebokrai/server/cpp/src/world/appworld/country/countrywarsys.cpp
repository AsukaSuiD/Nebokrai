#include "countrywarsys.h"

void CountryWarSys::OnFlagDestroyed(std::int32_t country, Context& context)
{
    for (auto& [regionId, state] : m_Regions) {
        if (state.defender == 0 || state.attacker == 0) continue;
        state.clear = false;
        const auto defender = state.defender;
        const auto attacker = state.attacker;
        const auto name = context.regionName ? context.regionName(regionId) : std::nullopt;
        std::optional<bool> attackerWon;
        if (name && context.countryExists && context.countryExists(static_cast<std::uint8_t>(defender)) && context.countryExists(static_cast<std::uint8_t>(attacker))) {
            if (context.sendDestroyedFlag) context.sendDestroyedFlag(static_cast<std::uint8_t>(country));
            if (defender == country) {
                if (context.setWarResult) { context.setWarResult(static_cast<std::uint8_t>(defender), 2); context.setWarResult(static_cast<std::uint8_t>(attacker), 1); }
                attackerWon = false;
            } else if (attacker == country) {
                if (context.setWarResult) { context.setWarResult(static_cast<std::uint8_t>(defender), 1); context.setWarResult(static_cast<std::uint8_t>(attacker), 2); }
                attackerWon = true;
            }
        }
        std::string text;
        if (attackerWon && name && context.formatNotice) text = context.formatNotice(*attackerWon, attacker, defender, *name);
        state.defender = 0; state.attacker = 0;
        if (context.sendCountryInfo) context.sendCountryInfo(text, 0xFFFFFE92U, 0xFFFF0000U);
    }
}
