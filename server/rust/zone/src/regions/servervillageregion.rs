//! Context-контракты, wire/log эффекты и скалярные правила деревенского
//! war-региона `CServerVillageRegion`. Исходный владелец —
//! `appserver/servervillageregion.h/.cpp`; сверка по точной паре
//! `gameserver.exe` + `GameServer.pdb`. Переходный агрегат остаётся в старом
//! пакете: хранит hub `CServerWarRegion`, ordered goods-list и flag-owner
//! колонку (runtime `CGame` перезаписывает обе при re-init региона), а этому
//! модулю делегированы owner-context, типы эффектов и чистые scalar-решения
//! нужных товаров и flag-owner после захвата символа. Машинные статусы
//! унаследованы от шапки старого владельца без повышения.
//!
//! Quirk-и: timeout намеренно игнорирует аргумент и шлёт текущие `(war, region,
//! flag-owner, 0)` как `0x60136`; `OnFactionWinOneSymbol` переносит flag owner
//! только по symbol `0`; `AddNeedGood` игнорирует пустое имя и сохраняет порядок
//! списка. PDB подтверждает ordered goods-list `+0x270` и
//! `m_lFlagOwnerFacID +0x27C`.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#war-регионы

pub trait VillageOwnerContext {
    type Region: Copy;
    fn find_region_then_proxy(&mut self, region_id: i32) -> Option<Self::Region>;
    fn owned_city_faction(&mut self, region: Self::Region) -> i32;
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VillageTimeoutEffect {
    pub war_number: i32,
    pub region_id: i32,
    pub flag_owner_faction_id: i32,
    pub region_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VillageWarLogEffect {
    pub string_id: &'static str,
    pub region_name: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VillageWarEndTargets {
    pub region_id: i32,
    pub player_ids: Vec<i32>,
    pub goods: Vec<String>,
}

/// Exact `AddNeedGood` `0x001D1900`: пустое имя не меняет ordered list.
pub fn village_add_need_good(goods: &mut Vec<String>, good_name: &str) {
    if !good_name.is_empty() {
        goods.push(good_name.to_owned());
    }
}

/// Exact `OnFactionWinOneSymbol`: только symbol `0` переносит flag owner на
/// победившую faction; для остальных symbols колонка сохраняется.
pub const fn village_flag_owner_after_symbol_win(
    current_flag_owner: i32,
    faction_id: i32,
    symbol_id: i32,
) -> i32 {
    if symbol_id == 0 {
        faction_id
    } else {
        current_flag_owner
    }
}
