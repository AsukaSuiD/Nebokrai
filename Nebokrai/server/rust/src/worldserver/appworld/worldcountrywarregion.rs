//! Владелец `WorldCountryWarRegion` WorldServer из точной пары EXE/PDB и
//! исходного owner-а `worldcountrywarregion.cpp`.
//!
//! Owner содержит один прямой `CWorldRegion` и шесть ordered list-ов. `Load`
//! сначала полностью выполняет base Load, затем читает `regions/{id}.country`.
//! Успешный open очищает и заполняет секции в порядке defend/attack gates,
//! defend/attack flags, defend/attack areas; missing resource сохраняет прежние
//! списки и возвращает `0`; успешный разбор возвращает `1`.
//!
//! Gate wire состоит из `0x2C` scalar bytes и двух C-строк, flag — из `0x28`
//! bytes и двух C-строк, area — из `0x14` bytes. Serializer сначала вызывает
//! base с тем же `include_child`, затем пишет шесть signed count-ов и записи в
//! том же порядке. Независимый GameServer decoder
//! `ServerCountryRegion::DecordFromByteArray` подтверждает
//! размеры и порядок. Имена и scripts сохраняются byte-: COUNTRY loader,
//! в отличие от CITY, не обращается к StringTable.
//! PDB дополнительно задаёт имена всех scalar-полей: gate содержит
//! `lID/lPicID/lDir/lAction/lMaxHP/lDef/lER/lTitleX/lTitleY/lWidthInc/`
//! `lHeightInc`, flag — ту же последовательность без `lAction`, area — `lID`
//! и четыре координаты `tagRECT`. Rust хранит их именованно; `[i32; N]` в
//! wire helper-е сохраняет x86 little-endian последовательность без старого
//! MSVC layout.
//!
//! `std::list`, stream, allocation, destructors и compiler cleanup заменены
//! `Vec`, заимствованными resource bytes и обычным `Drop`; Rust layout не
//! объявляется старым ABI.

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

/// Профиль country-war ворот из точного PDB `tagGate`.
#[derive(Clone, Debug, Eq, PartialEq)]
struct WorldCountryWarGate {
    id: i32,
    picture_id: i32,
    direction: i32,
    action: i32,
    maximum_hp: i32,
    defence: i32,
    element_resistant: i32,
    title_x: i32,
    title_y: i32,
    width_increment: i32,
    height_increment: i32,
    name: Vec<u8>,
    script: Vec<u8>,
}

impl WorldCountryWarGate {
    const fn wire_scalars(&self) -> [i32; 11] {
        [
            self.id,
            self.picture_id,
            self.direction,
            self.action,
            self.maximum_hp,
            self.defence,
            self.element_resistant,
            self.title_x,
            self.title_y,
            self.width_increment,
            self.height_increment,
        ]
    }
}

/// Профиль country-war флага из точного PDB `tagFlag`.
#[derive(Clone, Debug, Eq, PartialEq)]
struct WorldCountryWarFlag {
    id: i32,
    picture_id: i32,
    direction: i32,
    maximum_hp: i32,
    defence: i32,
    element_resistant: i32,
    title_x: i32,
    title_y: i32,
    width_increment: i32,
    height_increment: i32,
    name: Vec<u8>,
    script: Vec<u8>,
}

impl WorldCountryWarFlag {
    const fn wire_scalars(&self) -> [i32; 10] {
        [
            self.id,
            self.picture_id,
            self.direction,
            self.maximum_hp,
            self.defence,
            self.element_resistant,
            self.title_x,
            self.title_y,
            self.width_increment,
            self.height_increment,
        ]
    }
}

/// Зона country-war из PDB `tagArea`: ID и legacy `RECT`.
#[derive(Clone, Debug, Eq, PartialEq)]
struct WorldCountryWarArea {
    id: i32,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
}

impl WorldCountryWarArea {
    const fn wire_scalars(&self) -> [i32; 5] {
        [self.id, self.left, self.top, self.right, self.bottom]
    }
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

    pub(crate) fn load_from_context<Context, ResolveName>(
        &mut self,
        context: &mut Context,
        resolve_name: &mut ResolveName,
    ) -> Result<WorldCountryWarRegionLoadOutcome, WorldCountryWarRegionLoadError>
    where
        Context: WorldRegionResourceContext + ?Sized,
        ResolveName: FnMut(&[u8]) -> Vec<u8> + ?Sized,
    {
        let counts = self
            .base
            .load_from_context(context, resolve_name)
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
        let id = tokens.next_i32("tagGate.lID")?;
        let name = tokens.next("tagGate.strName")?.to_vec();
        let picture_id = tokens.next_i32("tagGate.lPicID")?;
        let direction = tokens.next_i32("tagGate.lDir")?;
        let action = tokens.next_i32("tagGate.lAction")?;
        let maximum_hp = tokens.next_i32("tagGate.lMaxHP")?;
        let defence = tokens.next_i32("tagGate.lDef")?;
        let element_resistant = tokens.next_i32("tagGate.lER")?;
        let title_x = tokens.next_i32("tagGate.lTitleX")?;
        let title_y = tokens.next_i32("tagGate.lTitleY")?;
        let width_increment = tokens.next_i32("tagGate.lWidthInc")?;
        let height_increment = tokens.next_i32("tagGate.lHeightInc")?;
        let script = tokens.next("tagGate.strScript")?.to_vec();
        records.push(WorldCountryWarGate {
            id,
            picture_id,
            direction,
            action,
            maximum_hp,
            defence,
            element_resistant,
            title_x,
            title_y,
            width_increment,
            height_increment,
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
        let id = tokens.next_i32("tagFlag.lID")?;
        let name = tokens.next("tagFlag.strName")?.to_vec();
        let picture_id = tokens.next_i32("tagFlag.lPicID")?;
        let direction = tokens.next_i32("tagFlag.lDir")?;
        let maximum_hp = tokens.next_i32("tagFlag.lMaxHP")?;
        let defence = tokens.next_i32("tagFlag.lDef")?;
        let element_resistant = tokens.next_i32("tagFlag.lER")?;
        let title_x = tokens.next_i32("tagFlag.lTitleX")?;
        let title_y = tokens.next_i32("tagFlag.lTitleY")?;
        let width_increment = tokens.next_i32("tagFlag.lWidthInc")?;
        let height_increment = tokens.next_i32("tagFlag.lHeightInc")?;
        let script = tokens.next("tagFlag.strScript")?.to_vec();
        records.push(WorldCountryWarFlag {
            id,
            picture_id,
            direction,
            maximum_hp,
            defence,
            element_resistant,
            title_x,
            title_y,
            width_increment,
            height_increment,
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
        records.push(WorldCountryWarArea {
            id: tokens.next_i32("tagArea.lID")?,
            left: tokens.next_i32("tagArea.rPoint.left")?,
            top: tokens.next_i32("tagArea.rPoint.top")?,
            right: tokens.next_i32("tagArea.rPoint.right")?,
            bottom: tokens.next_i32("tagArea.rPoint.bottom")?,
        });
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
        for scalar in record.wire_scalars() {
            destination.extend_from_slice(&scalar.to_le_bytes());
        }
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
        for scalar in record.wire_scalars() {
            destination.extend_from_slice(&scalar.to_le_bytes());
        }
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
        for scalar in record.wire_scalars() {
            destination.extend_from_slice(&scalar.to_le_bytes());
        }
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

fn append_country_c_string(destination: &mut Vec<u8>, value: &[u8]) {
    let end = value
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(value.len());
    destination.extend_from_slice(&value[..end]);
    destination.push(0);
}

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
