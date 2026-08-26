//! Proxy-region GameServer `CProxyServerRegion`, восстановленный по точной
//! паре `GameServer/gameserver.exe + GameServer/GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/proxyserverregion.cpp`.
//!
//! Wire намеренно не является полным `CServerRegion`: он вызывает только
//! `CBaseObject::DecordFromByteArray`, затем читает country byte, war-region
//! type и точный `0x24`-байтовый `tagRegionParam`. Это совпадает с парным
//! `CWorldRegion::AddToByteArrayForProxy`. Owned Rust-поля заменяют наследование
//! и ручное владение, не объявляя layout копией x86 ABI. Короткий bounded input
//! останавливается typed error-ом; старый decoder длину buffer-а не принимал.
//! Других неизвестных domain-полей этот конкретный wire-owner не читает.

use super::baseobject::{BaseObjectDecodeError, CBaseObject};
use super::legacycodec::LegacyReader;
use super::serverregion::RegionParamState;
use thiserror::Error;

#[derive(Clone, Debug, Eq, Error, PartialEq)]
pub(crate) enum ProxyRegionDecodeError {
    #[error("proxy region base decode: {0:?}")]
    Base(BaseObjectDecodeError),
    #[error("proxy region обрывается на {field} в {offset}: нужно {needed}, доступно {available}")]
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CProxyServerRegion {
    base_object: CBaseObject,
    country: u8,
    war_region_type: i32,
    param: RegionParamState,
    war_number: i32,
    city_state: i32,
}

impl Default for CProxyServerRegion {
    fn default() -> Self {
        let mut base_object = CBaseObject::with_reached_constructor_defaults();
        base_object.set_type(200);
        Self {
            base_object,
            country: 0,
            war_region_type: 0,
            param: RegionParamState::default(),
            war_number: 0,
            city_state: 0,
        }
    }
}

impl CProxyServerRegion {
    pub(crate) const fn get_id(&self) -> i32 {
        self.base_object.get_id()
    }

    pub(crate) fn get_name(&self) -> &[u8] {
        self.base_object.get_name()
    }

    pub(crate) const fn country(&self) -> u8 {
        self.country
    }

    pub(crate) const fn set_country(&mut self, country: u8) {
        self.country = country;
    }

    /// Proxy наследует virtual `CServerRegion::ReSetWarState`; короткий wire
    /// эти runtime-поля не заменяет.
    pub(crate) const fn reset_war_state(&mut self, war_number: i32, state: i32) {
        self.war_number = war_number;
        self.city_state = state;
    }

    pub(crate) const fn war_state(&self) -> (i32, i32) {
        (self.war_number, self.city_state)
    }

    pub(crate) const fn on_war_declare(&mut self, war_number: i32) {
        self.war_number = war_number;
        self.city_state = 1;
    }

    pub(crate) const fn on_war_start(&mut self) {
        self.city_state = 3;
    }

    pub(crate) const fn on_war_end(&mut self) {
        self.war_number = 0;
        self.city_state = 0;
    }

    pub(crate) const fn on_war_mass(&mut self) {
        self.city_state = 2;
    }

    pub(crate) const fn set_owned_city_org(&mut self, faction_id: i32, union_id: i32) {
        self.param.owned_faction_id = faction_id;
        self.param.owned_union_id = union_id;
    }

    pub(crate) const fn owned_city_org(&self) -> (i32, i32) {
        (self.param.owned_faction_id, self.param.owned_union_id)
    }

    pub(crate) const fn war_region_type(&self) -> i32 {
        self.war_region_type
    }

    pub(crate) const fn param(&self) -> &RegionParamState {
        &self.param
    }

    /// Применяет авторитетный налоговый снимок World без локальной публикации.
    pub(crate) const fn set_tax_snapshot(
        &mut self,
        today_total_tax: u32,
        total_tax: u32,
        current_tax_rate: i32,
    ) {
        self.param.today_total_tax = today_total_tax;
        self.param.total_tax = total_tax;
        self.param.current_tax_rate = current_tax_rate;
    }

    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
    ) -> Result<bool, ProxyRegionDecodeError> {
        self.base_object
            .decord_from_byte_array(source, cursor, include_child)
            .map_err(ProxyRegionDecodeError::Base)?;
        self.country = read_u8(source, cursor, "m_btCountry")?;
        self.war_region_type = read_i32(source, cursor, "m_WarRegionType")?;

        let bytes = read_bytes(source, cursor, 0x24, "m_Param")?;
        self.param = RegionParamState {
            region_id: read_param_i32(bytes, 0x00),
            max_tax_rate: read_param_i32(bytes, 0x04),
            current_tax_rate: read_param_i32(bytes, 0x08),
            total_tax: read_param_u32(bytes, 0x0C),
            today_total_tax: read_param_u32(bytes, 0x10),
            superior_region_id: read_param_i32(bytes, 0x14),
            turn_in_tax_rate: read_param_i32(bytes, 0x18),
            owned_faction_id: read_param_i32(bytes, 0x1C),
            owned_union_id: read_param_i32(bytes, 0x20),
        };
        Ok(true)
    }
}

fn read_u8(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u8, ProxyRegionDecodeError> {
    let mut reader = proxy_reader(source, *cursor, field, 1)?;
    let value = reader.read_u8().map_err(|block| proxy_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, ProxyRegionDecodeError> {
    let mut reader = proxy_reader(source, *cursor, field, 4)?;
    let value = reader.read_i32().map_err(|block| proxy_error(field, block))?;
    *cursor = reader.position();
    Ok(value)
}

fn read_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    needed: usize,
    field: &'static str,
) -> Result<&'a [u8], ProxyRegionDecodeError> {
    let mut reader = proxy_reader(source, *cursor, field, needed)?;
    let bytes = reader.read_bytes(needed).map_err(|block| proxy_error(field, block))?;
    *cursor = reader.position();
    Ok(bytes)
}

fn read_param_i32(bytes: &[u8], offset: usize) -> i32 {
    LegacyReader::at(bytes, offset)
        .and_then(|mut reader| reader.read_i32())
        .expect("m_Param содержит девять DWORD")
}

fn read_param_u32(bytes: &[u8], offset: usize) -> u32 {
    LegacyReader::at(bytes, offset)
        .and_then(|mut reader| reader.read_u32())
        .expect("m_Param содержит девять DWORD")
}

fn proxy_reader<'source>(source: &'source [u8], cursor: usize, field: &'static str, needed: usize) -> Result<LegacyReader<'source>, ProxyRegionDecodeError> {
    LegacyReader::at(source, cursor).map_err(|block| ProxyRegionDecodeError::UnexpectedEnd { field, offset: block.offset, needed, available: block.available })
}

fn proxy_error(field: &'static str, block: super::legacycodec::LegacyReadBlock) -> ProxyRegionDecodeError {
    ProxyRegionDecodeError::UnexpectedEnd { field, offset: block.offset, needed: block.needed, available: block.available }
}
