//! War-фазовое и city-ownership состояние `CServerRegion` исторического
//! GameServer (порция 4): phase defaults `OnWarDeclare/Start/End/Mass` (RVA
//! `0x00085560..0x000855B0`) и ownership/state accessors `SetOwnedCityOrg`,
//! `SetWarNum`, `SetCityState`, `ReSetWarState` (`0x000855D0..0x00085680`).
//! Исходный владелец — `appserver/serverregion.h/.cpp`; точная пара
//! `GameServer/gameserver.exe + GameServer/GameServer.pdb` (машинная
//! досверка 2026-09-26 подтвердила все тела построчно).
//!
//! Базовые тела записывают только скаляры переходного владельца
//! (`m_lWarNum +0x238`, `m_CityState +0x23C` — имена полей INFERRED по
//! именам методов, числа смещений машинные); порядок записей машинный:
//! `OnWarDeclare`/`OnWarEnd` — сначала `+0x23C`, затем `+0x238` (обратный
//! адресному), `ReSetWarState` — прямой. Ownership-пара живёт в
//! `RegionParamState` (`+0x230`/`+0x234`). Общий пустой символ
//! `0x00201A70` (`ret 4`) свёрнут ICF для ЧЕТЫРЁХ методов:
//! `OnWarTimeOut`, `OnClearOtherPlayer`, `OnRefreshRegion` и
//! `UpdateCityGateToClient` — наблюдаемых действий нет. Чистые чтения
//! (pub-офсеты `845F0/84600/84650/84660`, истинные RVA +0x1000) остаются
//! обвязками над полями переходного агрегата. Метки enum
//! `CITY_STATE_*` (0..3) — INFERRED по именам методов (числа и офсеты
//! машинные, сам enum в пабах не раскрыт).

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

/// Тело `OnWarEnd`: фаза NONE, затем сброс номера войны (порядок `+0x23C` до `+0x238` — машинный).
pub fn on_war_end(war_number: &mut i32, city_state: &mut i32) {
    *city_state = CITY_STATE_NONE;
    *war_number = 0;
}

/// Тело `OnWarMass`: только фаза MASS.
pub fn on_war_mass(city_state: &mut i32) {
    *city_state = CITY_STATE_MASS;
}

/// Тело `SetWarNum` (RVA `0x00085640`).
pub fn set_war_number(war_number: &mut i32, declared_war: i32) {
    *war_number = declared_war;
}

/// Тело `SetCityState` (RVA `0x00085670`).
pub fn set_city_state(city_state: &mut i32, state: i32) {
    *city_state = state;
}

/// Тело `ReSetWarState` (RVA `0x00085680`): номер войны, затем фаза (прямой адресный порядок — машинный факт).
pub fn reset_war_state(war_number: &mut i32, city_state: &mut i32, war: i32, state: i32) {
    *war_number = war;
    *city_state = state;
}

/// Тело `SetOwnedCityOrg` (RVA `0x000855D0`) над ownership-парой region param (`+0x230`, затем `+0x234`).
pub fn set_owned_city_org(param: &mut RegionParamState, faction_id: i32, union_id: i32) {
    param.owned_faction_id = faction_id;
    param.owned_union_id = union_id;
}
