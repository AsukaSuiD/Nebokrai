//! Владелец промежуточного `CWorldWarRegion` исторического WorldServer.
//!
//! Constructor RVA `0x000DD3B0`, `Load` RVA `0x000DD480` и serializer RVA
//! `0x000DD3F0` имеют статус `IMPLEMENTED`; exact EXE подтверждает три signed
//! DWORD по offsets `+0x120/+0x124/+0x128` после единственного
//! `CWorldRegion`. Сам base-constructor их не назначает, поэтому Rust хранит
//! `Option<i32>`; достигнутые Village/City constructors задают собственные
//! `1/1/1` и `3/3/2`. `Load` всегда сначала выполняет полный base Load, затем
//! независимо читает первый `#` из optional `regions/{id}.war`, но возвращает
//! именно base-result. Serializer дописывает три DWORD после base snapshot.
//! Virtual `DecordFromByteArray` RVA `0x000DD440` также `IMPLEMENTED`: он
//! вызывает доказанный no-op World decoder, не меняет cursor и возвращает
//! `true`. STL/compiler noise и destructors удалены в пользу стандартных
//! Rust-механизмов.
//! Точная пара `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! SHA-256 EXE `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! PDB `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходные owners `worldwarregion.cpp/.h`. STL stream и allocation заменены
//! заимствованными resource bytes и владением Rust; старый ABI не копируется.

use super::worldregion::{
    CWorldRegion, WorldRegionLoadError, WorldRegionLoadedCounts, WorldRegionResourceContext,
    WorldRegionSerializationBlock,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldWarRegionLoadError {
    Base(WorldRegionLoadError),
    MissingValue { field: &'static str },
    InvalidValue { field: &'static str },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldWarRegionSerializationBlock {
    Base(WorldRegionSerializationBlock),
    UninitializedField { field: &'static str },
}

pub(crate) struct CWorldWarRegion {
    base: CWorldRegion,
    symbol_total_num: Option<i32>,
    win_vic_symbol_num: Option<i32>,
    vic_symbol_num: Option<i32>,
}

impl CWorldWarRegion {
    /// Создаёт literal base-constructor state; три собственных DWORD не заданы.
    pub(crate) const fn with_constructor_base() -> Self {
        Self {
            base: CWorldRegion::with_constructor_region_base(),
            symbol_total_num: None,
            win_vic_symbol_num: None,
            vic_symbol_num: None,
        }
    }

    /// Применяет три значения конкретного derived constructor-а.
    pub(crate) const fn set_constructor_symbols(&mut self, total: i32, win: i32, vic: i32) {
        self.symbol_total_num = Some(total);
        self.win_vic_symbol_num = Some(win);
        self.vic_symbol_num = Some(vic);
    }

    pub(crate) const fn base(&self) -> &CWorldRegion {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CWorldRegion {
        &mut self.base
    }

    /// Выполняет base Load, затем optional war override, сохраняя base-result.
    pub(crate) fn load_from_context<Context>(
        &mut self,
        context: &mut Context,
    ) -> Result<WorldRegionLoadedCounts, WorldWarRegionLoadError>
    where
        Context: WorldRegionResourceContext + ?Sized,
    {
        let loaded = self
            .base
            .load_from_context(context)
            .map_err(WorldWarRegionLoadError::Base)?;
        let path = format!("regions/{}.war", self.base.get_id()).into_bytes();
        let war = context.read_resource(&path);
        self.load_war_bytes(war.as_deref())?;
        Ok(loaded)
    }

    /// Missing `.war` сохраняет constructor/previous values.
    pub(crate) fn load_war_bytes(
        &mut self,
        bytes: Option<&[u8]>,
    ) -> Result<bool, WorldWarRegionLoadError> {
        let Some(bytes) = bytes else {
            return Ok(true);
        };
        let mut tokens = bytes
            .split(|byte| byte.is_ascii_whitespace())
            .filter(|token| !token.is_empty());
        while let Some(token) = tokens.next() {
            if token != b"#" {
                continue;
            }
            self.symbol_total_num = Some(parse_next_i32(&mut tokens, "m_lSymbolTotalNum")?);
            self.win_vic_symbol_num = Some(parse_next_i32(&mut tokens, "m_lWinVicSymbolNum")?);
            self.vic_symbol_num = Some(parse_next_i32(&mut tokens, "m_lVicSymbolNum")?);
            break;
        }
        Ok(true)
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, WorldWarRegionSerializationBlock> {
        let _ = self
            .base
            .add_to_byte_array(destination, include_child)
            .map_err(WorldWarRegionSerializationBlock::Base)?;
        for (field, value) in [
            ("m_lSymbolTotalNum", self.symbol_total_num),
            ("m_lWinVicSymbolNum", self.win_vic_symbol_num),
            ("m_lVicSymbolNum", self.vic_symbol_num),
        ] {
            let value =
                value.ok_or(WorldWarRegionSerializationBlock::UninitializedField { field })?;
            destination.extend_from_slice(&value.to_le_bytes());
        }
        Ok(true)
    }

    /// Derived override сохраняет доказанный no-op base decoder и возвращает `true`.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
    ) -> bool {
        let _ = self
            .base
            .decord_from_byte_array(source, cursor, include_child);
        true
    }
}

fn parse_next_i32<'a, Tokens>(
    tokens: &mut Tokens,
    field: &'static str,
) -> Result<i32, WorldWarRegionLoadError>
where
    Tokens: Iterator<Item = &'a [u8]>,
{
    let value = tokens
        .next()
        .ok_or(WorldWarRegionLoadError::MissingValue { field })?;
    std::str::from_utf8(value)
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or(WorldWarRegionLoadError::InvalidValue { field })
}
