//! Wire-проекция `tagRegionParam` регионов GameServer (налоги и владение
//! городом).
//!
//! Точные девять DWORD в wire-порядке подтверждены обеими сторонами пары:
//! Game `CProxyServerRegion::DecordFromByteArray` (`0x4FC440` base, country
//! byte, war-region type, затем блок `0x24`) и World
//! `CWorldRegion::AddToByteArrayForProxy` (зеркальная запись блока из `+0xFC`).

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RegionParamState {
    pub region_id: i32,
    pub max_tax_rate: i32,
    pub current_tax_rate: i32,
    pub total_tax: u32,
    pub today_total_tax: u32,
    pub superior_region_id: i32,
    pub turn_in_tax_rate: i32,
    pub owned_faction_id: i32,
    pub owned_union_id: i32,
}
