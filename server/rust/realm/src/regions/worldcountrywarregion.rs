//! Country-war регион `WorldCountryWarRegion` из
//! `worldcountrywarregion.cpp`, подтверждённый `worldserver.exe` и
//! `worldserver.pdb`, перенесённый в Realm `regions/`.
//!
//! После base Load owner читает `regions/{id}.country`. Успешный open очищает
//! и заполняет defend/attack gates, flags и areas; missing resource сохраняет
//! прежние списки.
//!
//! Wire пишет base, затем шесть signed counts и ordered records: gate 0x2C с
//! двумя C-строками, flag 0x28 с двумя C-строками, area 0x14. Имена/scripts
//! остаются byte-exact и не проходят StringTable. Именованные scalars
//! сериализуются little-endian без зависимости от Rust/MSVC layout.

use crate::app::worldserver::WorldRegionResourceContext;
use crate::regions::worldregion::{
    CWorldRegion, WorldRegionLoadError, WorldRegionLoadedCounts, WorldRegionSerializationBlock,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldCountryWarRegionTextLoadError {
    MissingValue { field: &'static str },
    InvalidValue { field: &'static str },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldCountryWarRegionLoadError {
    Base(WorldRegionLoadError),
    Country(WorldCountryWarRegionTextLoadError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldCountryWarRegionSerializationBlock {
    Base(WorldRegionSerializationBlock),
    TooManyEntries { section: &'static str, count: usize },
}

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
pub struct WorldCountryWarRegionLoadOutcome {
    pub counts: WorldRegionLoadedCounts,
    pub loaded: bool,
}

pub struct WorldCountryWarRegion {
    base: CWorldRegion,
    defend_gates: Vec<WorldCountryWarGate>,
    defend_flags: Vec<WorldCountryWarFlag>,
    defend_areas: Vec<WorldCountryWarArea>,
    attack_gates: Vec<WorldCountryWarGate>,
    attack_flags: Vec<WorldCountryWarFlag>,
    attack_areas: Vec<WorldCountryWarArea>,
}

impl WorldCountryWarRegion {
    pub const fn with_constructor_state() -> Self {
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

    pub const fn base(&self) -> &CWorldRegion {
        &self.base
    }

    pub const fn base_mut(&mut self) -> &mut CWorldRegion {
        &mut self.base
    }

    pub fn load_from_context<Context, ResolveName>(
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

    pub fn load_country_bytes(
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

    pub fn add_to_byte_array(
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
