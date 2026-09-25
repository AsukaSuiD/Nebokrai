//! War-фазовое и city-ownership состояние `CServerRegion` исторического
//! GameServer (порция 4): phase defaults `OnWarDeclare/Start/End/Mass` (RVA
//! `0x00085560..0x000855B0`) и ownership/state accessors `SetOwnedCityOrg`,
//! `SetWarNum`, `SetCityState`, `ReSetWarState` (`0x000855D0..0x00085680`).
//! Исходный владелец — `appserver/serverregion.h/.cpp`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb`.
//!
//! Базовые тела записывают только два скаляра переходного владельца
//! (`m_lWarNum +0x238`, `m_CityState +0x23C`) в исходном порядке полей;
//! ownership-пара живёт в `RegionParamState`. Общий
//! `OnWarTimeOut/OnClearOtherPlayer/OnRefreshRegion` — один PDB-symbol
//! `0x00201A70` (`ret 4`), наблюдаемых действий не имеет и отдельного ядра
//! не требует. Чистые чтения (`GetWarNum` `0x00084650`, `GetCityState`
//! `0x00084660`, `GetOwnedCityFaction/Union` `0x000845F0/0x00084600`) остаются
//! обвязками над полями переходного агрегата.

use crate::regions::regionparam::RegionParamState;

use super::geometry::{CITY_STATE_DECLARE, CITY_STATE_FIGHT, CITY_STATE_MASS, CITY_STATE_NONE};

/// Тело `OnWarDeclare`: фаза DECLARE, затем номер войны.
pub fn on_war_declare(war_number: &mut i32, city_state: &mut i32, declared_war: i32) {
    *city_state = CITY_STATE_DECLARE;
    *war_number = declared_war;
}

/// Тело `OnWarStart`: только фаза FIGHT, номер войны не меняется.
pub fn on_war_start(city_state: &mut i32) {
    *city_state = CITY_STATE_FIGHT;
}

/// Тело `OnWarEnd`: фаза NONE и сброс номера войны в исходном порядке полей.
pub fn on_war_end(war_number: &mut i32, city_state: &mut i32) {
    *city_state = CITY_STATE_NONE;
    *war_number = 0;
}

/// Тело `OnWarMass`: только фаза MASS.
pub fn on_war_mass(city_state: &mut i32) {
    *city_state = CITY_STATE_MASS;
}

/// Тело `SetWarNum` (`0x00084640`).
pub fn set_war_number(war_number: &mut i32, declared_war: i32) {
    *war_number = declared_war;
}

/// Тело `SetCityState` (`0x00084670`).
pub fn set_city_state(city_state: &mut i32, state: i32) {
    *city_state = state;
}

/// Тело `ReSetWarState` (`0x00084680`): номер войны, затем фаза.
pub fn reset_war_state(war_number: &mut i32, city_state: &mut i32, war: i32, state: i32) {
    *war_number = war;
    *city_state = state;
}

/// Тело `SetOwnedCityOrg` (`0x000845D0`) над ownership-парой region param.
pub fn set_owned_city_org(param: &mut RegionParamState, faction_id: i32, union_id: i32) {
    param.owned_faction_id = faction_id;
    param.owned_union_id = union_id;
}
