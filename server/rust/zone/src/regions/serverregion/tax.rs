//! Tax data-контракты начисления и сбора налогов региона исторического
//! GameServer (порция 1). Исходный владелец — `appserver/serverregion.h/.cpp`;
//! точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`.
//! Двухфазный `CNetSession`-endpoint налогового диалога остаётся у
//! переходного владельца до своей порции.

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegionTaxAddition {
    pub region_id: i32,
    pub retained: u32,
    pub superior_region_id: Option<i32>,
    pub superior_share: u32,
    pub today_total_tax: u32,
    pub total_tax: u32,
    pub current_tax_rate: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RegionTaxCollection {
    pub region_id: i32,
    pub collected: u32,
    pub today_total_tax: u32,
    pub total_tax: u32,
    pub current_tax_rate: i32,
}
