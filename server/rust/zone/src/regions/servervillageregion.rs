//! Context-контракты, wire/log эффекты и скалярные правила деревенского
//! war-региона `CServerVillageRegion` исторического GameServer, перенесённые в
//! Zone `regions/` волной Z-M-Xf (семья war-регионов war+godsbattle+village).
//! Исходный владелец — `appserver/servervillageregion.h/.cpp`. Переходный
//! агрегат остаётся в старом пакете: хранит hub `CServerWarRegion`, ordered
//! goods-list и flag-owner колонку (runtime `CGame` перезаписывает обе
//! напрямую при re-init региона), schedule-запросы `CVillageWarSys` и
//! фазовые/tail-вызовы над этими колонками; этому модулю делегируются
//! owner-context, типы эффектов и чистые scalar-решения нужных товаров и
//! flag-owner после захвата символа.
//!
//! Точная пара: `GameServer/gameserver.exe` (SHA-256
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`) +
//! `GameServer/GameServer.pdb` (RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53`,
//! age 2). Фазовые callbacks RVA `0x001D1310`, `0x001D13C0`,
//! `0x001D1590..0x001D16D0`, victory `0x001D12C0`, clear `0x001D1370`,
//! membership `0x001D12F0`, direct timeout-forwarding `0x001D14F0` и
//! `AddNeedGood` `0x001D1900` имеют статус `IMPLEMENTED`; decoder-forwarding
//! `0x001D1280` и ownership query `0x001D1980` — `IMPLEMENTED,
//! VERIFIED_DISASSEMBLY` (наследие шапки старого владельца, без повышения).
//! PDB подтверждает наследование `CServerWarRegion`, ordered goods-list
//! `+0x270` и `m_lFlagOwnerFacID +0x27C`.
//!
//! Timeout намеренно игнорирует аргумент и шлёт текущие `(war, region,
//! flag-owner, 0)` как `0x60136`. `OnFactionWinOneSymbol` переносит flag
//! owner только по symbol `0`, остальные символы не меняют колонку.
//! `AddNeedGood` игнорирует пустое имя и сохраняет порядок списка.

/// Ищет сначала `s_mapRegion`, а при miss либо null — `FindProxyRegion`;
/// owned city faction читается у найденного schedule context owner-а.
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
