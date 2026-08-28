//! Национальный окружной охранник с мечом (AI19).
//!
//! Источник: точная пара gameserver.exe + GameServer.pdb, исходный владелец
//! appserver/ai/nationcouguardwithsword.cpp. Общий владелец окружной охраны
//! сохраняет точку поста, дальность преследования и правило минимальной
//! дистанции навыка; этот наследник добавляет фильтр страны монстра и статус
//! преступника. Поиск игроков выполняется раньше поиска питомцев, а игрок
//! побеждает при равном итоговом расстоянии.

use super::vilcouguardwithsword::{CountryGuardTarget, consider_country_guard_target};

pub(crate) fn consider_nation_country_guard_player(
    selected: Option<CountryGuardTarget>,
    candidate: CountryGuardTarget,
    guard_range: i32,
    minimum_skill_distance: i32,
    region_country: u8,
    guard_country: u32,
    player_country: u8,
    player_is_badman: bool,
) -> Option<CountryGuardTarget> {
    if (region_country != 0 && player_country == region_country)
        || (!player_is_badman && u32::from(player_country) == guard_country)
    {
        return selected;
    }
    consider_country_guard_target(
        selected,
        candidate,
        guard_range,
        minimum_skill_distance,
    )
}
