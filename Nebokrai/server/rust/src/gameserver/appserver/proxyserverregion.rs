//! Proxy-region GameServer `CProxyServerRegion`.
//!
//! Constructor/destructor RVA `0x001CA8C0/0x001CA8E0` и полный decoder
//! `DecordFromByteArray` RVA `0x001CA910` имеют статус `IMPLEMENTED,
//! VERIFIED_DISASSEMBLY`; точная пара `GameServer/gameserver.exe +
//! GameServer/GameServer.pdb`, SHA-256 EXE
//! `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`, PDB
//! `B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016`.
//! Исходный owner `server/gameserver/appserver/proxyserverregion.cpp`.
//!
//! Wire намеренно не является полным `CServerRegion`: он вызывает только
//! `CBaseObject::DecordFromByteArray`, затем читает country byte, war-region
//! type и точный `0x24`-байтовый `tagRegionParam`. Это совпадает с парным
//! `CWorldRegion::AddToByteArrayForProxy`. Owned Rust-поля заменяют наследование
//! и ручное владение, не объявляя layout копией x86 ABI. Короткий bounded input
//! останавливается typed error-ом; старый decoder длину buffer-а не принимал.
//! Других неизвестных domain-полей этот конкретный wire-owner не читает.

use super::baseobject::{BaseObjectDecodeError, CBaseObject};
use super::serverregion::RegionParamState;
use std::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ProxyRegionDecodeError {
    Base(BaseObjectDecodeError),
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

impl fmt::Display for ProxyRegionDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Base(error) => write!(formatter, "proxy region base decode: {error:?}"),
            Self::UnexpectedEnd {
                field,
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "proxy region обрывается на {field} в {offset}: нужно {needed}, доступно {available}"
            ),
        }
    }
}

impl std::error::Error for ProxyRegionDecodeError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CProxyServerRegion {
    base_object: CBaseObject,
    country: u8,
    war_region_type: i32,
    param: RegionParamState,
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

    pub(crate) const fn war_region_type(&self) -> i32 {
        self.war_region_type
    }

    pub(crate) const fn param(&self) -> &RegionParamState {
        &self.param
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
    let offset = *cursor;
    let Some(value) = source.get(offset).copied() else {
        return Err(ProxyRegionDecodeError::UnexpectedEnd {
            field,
            offset,
            needed: 1,
            available: source.len().saturating_sub(offset),
        });
    };
    *cursor = offset + 1;
    Ok(value)
}

fn read_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, ProxyRegionDecodeError> {
    let bytes = read_bytes(source, cursor, 4, field)?;
    Ok(i32::from_le_bytes(
        bytes.try_into().expect("проверены четыре байта"),
    ))
}

fn read_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    needed: usize,
    field: &'static str,
) -> Result<&'a [u8], ProxyRegionDecodeError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(needed) else {
        return Err(ProxyRegionDecodeError::UnexpectedEnd {
            field,
            offset,
            needed,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(ProxyRegionDecodeError::UnexpectedEnd {
            field,
            offset,
            needed,
            available,
        });
    };
    *cursor = end;
    Ok(bytes)
}

fn read_param_i32(bytes: &[u8], offset: usize) -> i32 {
    i32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("m_Param содержит девять DWORD"),
    )
}

fn read_param_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(
        bytes[offset..offset + 4]
            .try_into()
            .expect("m_Param содержит девять DWORD"),
    )
}
