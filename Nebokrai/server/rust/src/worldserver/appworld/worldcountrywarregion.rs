//! Владелец `WorldCountryWarRegion` исторического WorldServer — `IMPLEMENTED`.
//!
//! Constructor RVA `0x00078790`, virtual `Load` RVA `0x00078920` и serializer
//! RVA `0x00077F30` восстановлены для точной пары
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! source `worldcountrywarregion.cpp:5,13,197`.
//!
//! Owner содержит один прямой `CWorldRegion` и шесть ordered list-ов. `Load`
//! сначала полностью выполняет base Load, затем читает `regions/{id}.country`.
//! Успешный open очищает и заполняет секции в порядке defend/attack gates,
//! defend/attack flags, defend/attack areas; missing resource сохраняет прежние
//! списки и возвращает `0`. Exact disassembly подтверждает `eax=0` на ветке
//! `00478986 -> 0047950A` и `eax=1` после разбора по `00479503`.
//!
//! Gate wire состоит из `0x2C` scalar bytes и двух C-строк, flag — из `0x28`
//! bytes и двух C-строк, area — из `0x14` bytes. Serializer сначала вызывает
//! base с тем же `include_child`, затем пишет шесть signed count-ов и записи в
//! том же порядке. Независимый GameServer decoder
//! `ServerCountryRegion::DecordFromByteArray` RVA `0x001CD3F0` подтверждает
//! размеры и порядок. Имена и scripts сохраняются byte-exact: COUNTRY loader,
//! в отличие от CITY, не обращается к StringTable.
//!
//! `std::list`, stream, allocation, destructors и compiler cleanup заменены
//! `Vec`, заимствованными resource bytes и обычным `Drop`; Rust layout не
//! объявляется старым ABI. Полный сырой декомпилят удалён после переноса.

use super::worldregion::{
    CWorldRegion, WorldRegionLoadError, WorldRegionLoadedCounts, WorldRegionResourceContext,
    WorldRegionSerializationBlock,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryWarRegionTextLoadError {
    MissingValue { field: &'static str },
    InvalidValue { field: &'static str },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryWarRegionLoadError {
    Base(WorldRegionLoadError),
    Country(WorldCountryWarRegionTextLoadError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum WorldCountryWarRegionSerializationBlock {
    Base(WorldRegionSerializationBlock),
    TooManyEntries { section: &'static str, count: usize },
}

#[derive(Clone, Debug)]
struct WorldCountryWarGate {
    header: [u8; 0x2C],
    name: Vec<u8>,
    script: Vec<u8>,
}

#[derive(Clone, Debug)]
struct WorldCountryWarFlag {
    header: [u8; 0x28],
    name: Vec<u8>,
    script: Vec<u8>,
}

#[derive(Clone, Debug)]
struct WorldCountryWarArea {
    header: [u8; 0x14],
}

#[derive(Clone, Debug)]
pub(crate) struct WorldCountryWarRegionLoadOutcome {
    pub(crate) counts: WorldRegionLoadedCounts,
    pub(crate) loaded: bool,
}

pub(crate) struct WorldCountryWarRegion {
    base: CWorldRegion,
    defend_gates: Vec<WorldCountryWarGate>,
    defend_flags: Vec<WorldCountryWarFlag>,
    defend_areas: Vec<WorldCountryWarArea>,
    attack_gates: Vec<WorldCountryWarGate>,
    attack_flags: Vec<WorldCountryWarFlag>,
    attack_areas: Vec<WorldCountryWarArea>,
}

impl WorldCountryWarRegion {
    pub(crate) const fn with_constructor_state() -> Self {
        Self {
            base: CWorldRegion::with_constructor_region_base(),
            defend_gates: Vec::new(),
            defend_flags: Vec::new(),
            defend_areas: Vec::new(),
            attack_gates: Vec::new(),
            attack_flags: Vec::new(),
            attack_areas: Vec::new(),
        }
    }

    pub(crate) const fn base(&self) -> &CWorldRegion {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CWorldRegion {
        &mut self.base
    }

    pub(crate) fn load_from_context<Context>(
        &mut self,
        context: &mut Context,
    ) -> Result<WorldCountryWarRegionLoadOutcome, WorldCountryWarRegionLoadError>
    where
        Context: WorldRegionResourceContext + ?Sized,
    {
        let counts = self
            .base
            .load_from_context(context)
            .map_err(WorldCountryWarRegionLoadError::Base)?;
        let path = format!("regions/{}.country", self.base.get_id()).into_bytes();
        let country = context.read_resource(&path);
        let loaded = self
            .load_country_bytes(country.as_deref())
            .map_err(WorldCountryWarRegionLoadError::Country)?;
        Ok(WorldCountryWarRegionLoadOutcome { counts, loaded })
    }

    /// Missing `.country` возвращает legacy `0` и не очищает прежние списки.
    pub(crate) fn load_country_bytes(
        &mut self,
        bytes: Option<&[u8]>,
    ) -> Result<bool, WorldCountryWarRegionTextLoadError> {
        let Some(bytes) = bytes else {
            return Ok(false);
        };
        let mut tokens = CountryTokens::new(bytes);
        let defend_gates = parse_gate_section(&mut tokens)?;
        let attack_gates = parse_gate_section(&mut tokens)?;
        let defend_flags = parse_flag_section(&mut tokens)?;
        let attack_flags = parse_flag_section(&mut tokens)?;
        let defend_areas = parse_area_section(&mut tokens)?;
        let attack_areas = parse_area_section(&mut tokens)?;

        self.defend_gates = defend_gates;
        self.attack_gates = attack_gates;
        self.defend_flags = defend_flags;
        self.attack_flags = attack_flags;
        self.defend_areas = defend_areas;
        self.attack_areas = attack_areas;
        Ok(true)
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, WorldCountryWarRegionSerializationBlock> {
        let _ = self
            .base
            .add_to_byte_array(destination, include_child)
            .map_err(WorldCountryWarRegionSerializationBlock::Base)?;
        append_gate_section(destination, "_defend_gates", &self.defend_gates)?;
        append_gate_section(destination, "_attack_gates", &self.attack_gates)?;
        append_flag_section(destination, "_defend_flags", &self.defend_flags)?;
        append_flag_section(destination, "_attack_flags", &self.attack_flags)?;
        append_area_section(destination, "_defend_areas", &self.defend_areas)?;
        append_area_section(destination, "_attack_areas", &self.attack_areas)?;
        Ok(true)
    }
}

fn parse_gate_section(
    tokens: &mut CountryTokens<'_>,
) -> Result<Vec<WorldCountryWarGate>, WorldCountryWarRegionTextLoadError> {
    let mut records = Vec::new();
    while let Some(token) = tokens.next_optional() {
        if token == b"<end>" {
            break;
        }
        if token != b"#" {
            continue;
        }
        let mut header = [0; 0x2C];
        write_i32(&mut header, 0, tokens.next_i32("tagGate.field_00")?);
        let name = tokens.next("tagGate.strName")?.to_vec();
        for (index, field) in GATE_FIELDS.iter().enumerate().skip(1) {
            write_i32(&mut header, index, tokens.next_i32(field)?);
        }
        let script = tokens.next("tagGate.strScript")?.to_vec();
        records.push(WorldCountryWarGate {
            header,
            name,
            script,
        });
    }
    Ok(records)
}

fn parse_flag_section(
    tokens: &mut CountryTokens<'_>,
) -> Result<Vec<WorldCountryWarFlag>, WorldCountryWarRegionTextLoadError> {
    let mut records = Vec::new();
    while let Some(token) = tokens.next_optional() {
        if token == b"<end>" {
            break;
        }
        if token != b"#" {
            continue;
        }
        let mut header = [0; 0x28];
        write_i32(&mut header, 0, tokens.next_i32("tagFlag.field_00")?);
        let name = tokens.next("tagFlag.strName")?.to_vec();
        for (index, field) in FLAG_FIELDS.iter().enumerate().skip(1) {
            write_i32(&mut header, index, tokens.next_i32(field)?);
        }
        let script = tokens.next("tagFlag.strScript")?.to_vec();
        records.push(WorldCountryWarFlag {
            header,
            name,
            script,
        });
    }
    Ok(records)
}

fn parse_area_section(
    tokens: &mut CountryTokens<'_>,
) -> Result<Vec<WorldCountryWarArea>, WorldCountryWarRegionTextLoadError> {
    let mut records = Vec::new();
    while let Some(token) = tokens.next_optional() {
        if token == b"<end>" {
            break;
        }
        if token != b"#" {
            continue;
        }
        let mut header = [0; 0x14];
        for (index, field) in AREA_FIELDS.iter().enumerate() {
            write_i32(&mut header, index, tokens.next_i32(field)?);
        }
        records.push(WorldCountryWarArea { header });
    }
    Ok(records)
}

fn append_gate_section(
    destination: &mut Vec<u8>,
    section: &'static str,
    records: &[WorldCountryWarGate],
) -> Result<(), WorldCountryWarRegionSerializationBlock> {
    append_count(destination, section, records.len())?;
    for record in records {
        destination.extend_from_slice(&record.header);
        append_country_c_string(destination, &record.name);
        append_country_c_string(destination, &record.script);
    }
    Ok(())
}

fn append_flag_section(
    destination: &mut Vec<u8>,
    section: &'static str,
    records: &[WorldCountryWarFlag],
) -> Result<(), WorldCountryWarRegionSerializationBlock> {
    append_count(destination, section, records.len())?;
    for record in records {
        destination.extend_from_slice(&record.header);
        append_country_c_string(destination, &record.name);
        append_country_c_string(destination, &record.script);
    }
    Ok(())
}

fn append_area_section(
    destination: &mut Vec<u8>,
    section: &'static str,
    records: &[WorldCountryWarArea],
) -> Result<(), WorldCountryWarRegionSerializationBlock> {
    append_count(destination, section, records.len())?;
    for record in records {
        destination.extend_from_slice(&record.header);
    }
    Ok(())
}

fn append_count(
    destination: &mut Vec<u8>,
    section: &'static str,
    count: usize,
) -> Result<(), WorldCountryWarRegionSerializationBlock> {
    let count = i32::try_from(count)
        .map_err(|_| WorldCountryWarRegionSerializationBlock::TooManyEntries { section, count })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn write_i32<const SIZE: usize>(destination: &mut [u8; SIZE], index: usize, value: i32) {
    let offset = index * 4;
    destination[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
}

fn append_country_c_string(destination: &mut Vec<u8>, value: &[u8]) {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    destination.extend_from_slice(&value[..end]);
    destination.push(0);
}

const GATE_FIELDS: [&str; 11] = [
    "tagGate.field_00",
    "tagGate.field_04",
    "tagGate.field_08",
    "tagGate.field_0C",
    "tagGate.field_10",
    "tagGate.field_14",
    "tagGate.field_18",
    "tagGate.field_1C",
    "tagGate.field_20",
    "tagGate.field_24",
    "tagGate.field_28",
];

const FLAG_FIELDS: [&str; 10] = [
    "tagFlag.field_00",
    "tagFlag.field_04",
    "tagFlag.field_08",
    "tagFlag.field_0C",
    "tagFlag.field_10",
    "tagFlag.field_14",
    "tagFlag.field_18",
    "tagFlag.field_1C",
    "tagFlag.field_20",
    "tagFlag.field_24",
];

const AREA_FIELDS: [&str; 5] = [
    "tagArea.field_00",
    "tagArea.field_04",
    "tagArea.field_08",
    "tagArea.field_0C",
    "tagArea.field_10",
];

struct CountryTokens<'a> {
    values: Vec<&'a [u8]>,
    next: usize,
}

impl<'a> CountryTokens<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self {
            values: bytes
                .split(|byte| byte.is_ascii_whitespace())
                .filter(|token| !token.is_empty())
                .collect(),
            next: 0,
        }
    }

    fn next_optional(&mut self) -> Option<&'a [u8]> {
        let value = self.values.get(self.next).copied()?;
        self.next += 1;
        Some(value)
    }

    fn next(
        &mut self,
        field: &'static str,
    ) -> Result<&'a [u8], WorldCountryWarRegionTextLoadError> {
        self.next_optional()
            .ok_or(WorldCountryWarRegionTextLoadError::MissingValue { field })
    }

    fn next_i32(&mut self, field: &'static str) -> Result<i32, WorldCountryWarRegionTextLoadError> {
        let value = self.next(field)?;
        std::str::from_utf8(value)
            .ok()
            .and_then(|value| value.parse().ok())
            .ok_or(WorldCountryWarRegionTextLoadError::InvalidValue { field })
    }
}
