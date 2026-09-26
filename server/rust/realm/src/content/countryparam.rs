//! Параметры `CCountryParam` из `countryparam.cpp/.h`.
//!
//! `Load` сначала очищает шесть start/main maps, сохраняя technology и exile,
//! затем позиционно читает 39 scalars и записи `*`, `#`, `+`. Повторный ключ
//! заменяет значение; ошибка ресурса оставляет scalars/technology/exile прежними
//! и общий legacy-успех, но шесть maps уже пусты.
//!
//! Wire содержит 39 DWORD, main region/rect/direction, technology и exile maps
//! со signed counts; start maps не передаются. Technology record имеет порядок
//! `level, country_power, country_tech_exp`, отличный от порядка загрузки.
//! Непрочитанные scalars остаются `None`; `BTreeMap` заменяет MSVC tree.

use std::collections::BTreeMap;

use nebokrai_shared::resources::read_to_marker as read_to;

const COUNTRY_PARAMETER_COUNT: usize = 39;
const MAX_COUNTRY_POWER: usize = 2;
const MAX_COUNTRY_TREASURY: usize = 4;
const DAILY_COUNTRY_TREASURY: usize = 5;
const KING_NEED_LEVEL: usize = 7;
const KING_NEED_CREDIT: usize = 8;
const DEFAULT_KING_CONTROL_POINT: usize = 9;
const MAX_KING_CONTROL_POINT: usize = 10;
const MIN_KING_CONTROL_POINT: usize = 11;
const DEC_KING_CONTROL_POINT_INTERVAL: usize = 12;
const DEC_KING_CONTROL_POINT_TIME: usize = 16;
const DEC_KING_CONTROL_POINT_DEMISE: usize = 17;
const DEC_KING_CONTROL_POINT_SILENCE: usize = 18;
const DEC_KING_CONTROL_POINT_EXILE: usize = 20;
const DEC_KING_CONTROL_POINT_ABSOLVE: usize = 21;
const DEC_KING_CONTROL_POINT_APPOINT: usize = 22;
const DEFAULT_KING_CONTROL_POINT_DEMISE_NEED: usize = 24;
const MAX_KING_MATERIAL_POINT: usize = 26;
const MAX_KING_WAR_POINT: usize = 28;
const MAX_SILENCE_NUM: usize = 29;
const SILENCE_TIME: usize = 30;
const EXILE_TIME: usize = 32;
const MAX_EXILE_NUM: usize = 31;
const MAX_EXILE_PK: usize = 33;
const MAX_ABSOLVE_NUM: usize = 35;

const COUNTRY_PARAMETER_FIELDS: [&str; COUNTRY_PARAMETER_COUNT] = [
    "m_lMaxCountyrs",
    "_def_country_power",
    "_max_country_power",
    "_def_country_treasury",
    "_max_country_treasury",
    "_daily_country_treasury",
    "_def_country_tecklel",
    "_king_need_level",
    "_king_need_credit",
    "_def_king_control_point",
    "_max_king_control_point",
    "_min_king_control_point",
    "_dec_king_control_point_interval",
    "_inc_king_control_point_win",
    "_inc_king_control_point_fail",
    "_inc_king_control_point_draw",
    "_dec_king_control_point_time",
    "_dec_king_control_point_demise",
    "_dec_king_control_point_silence",
    "_dec_king_control_point_police",
    "_dec_king_control_point_exile",
    "_dec_king_control_point_absolve",
    "_dec_king_control_point_appoint",
    "_dec_king_control_point_convene",
    "_def_king_control_point_demise_need",
    "_def_king_material_point",
    "_max_king_material_point",
    "_def_king_war_point",
    "_max_king_war_point",
    "_max_silence_num",
    "_silence_time",
    "_max_exile_num",
    "_exile_time",
    "_max_exile_pk",
    "_max_pk_num",
    "_max_absolve_num",
    "_max_exploit",
    "_max_kudos",
    "_inc_exploit",
];

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CountryRect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CountryTechLevel {
    country_tech_exp: i32,
    country_power: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountryTechLevelLookup {
    pub level: i32,
    pub inserted: bool,
    pub country_tech_exp: i32,
    pub country_power: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountryReturnPoint {
    pub region_id: i32,
    pub rect: CountryRect,
    pub direction: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountryParamLoadReport {
    pub resource_found: bool,
    pub legacy_result: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryParamLoadError {
    MissingValue { field: &'static str },
    InvalidValue { field: &'static str },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryParamSerializationBlock {
    UninitializedParameter {
        field: &'static str,
    },
    TooManyEntries {
        collection: &'static str,
        count: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountryParameterUnavailable {
    pub field: &'static str,
}

pub struct CCountryParam {
    parameters: [Option<i32>; COUNTRY_PARAMETER_COUNT],
    start_regions: BTreeMap<u8, i32>,
    start_rects: BTreeMap<u8, CountryRect>,
    start_directions: BTreeMap<u8, i32>,
    main_regions: BTreeMap<u8, i32>,
    main_rects: BTreeMap<u8, CountryRect>,
    main_directions: BTreeMap<u8, i32>,
    country_tech_levels: BTreeMap<i32, CountryTechLevel>,
    exile_rects: BTreeMap<u8, CountryRect>,
}

impl CCountryParam {
    pub fn technology_level_or_insert(&mut self, level: i32) -> CountryTechLevelLookup {
        let inserted = !self.country_tech_levels.contains_key(&level);
        let technology = self.country_tech_levels.entry(level).or_default();
        CountryTechLevelLookup {
            level,
            inserted,
            country_tech_exp: technology.country_tech_exp,
            country_power: technology.country_power,
        }
    }

    pub const fn new() -> Self {
        Self {
            parameters: [None; COUNTRY_PARAMETER_COUNT],
            start_regions: BTreeMap::new(),
            start_rects: BTreeMap::new(),
            start_directions: BTreeMap::new(),
            main_regions: BTreeMap::new(),
            main_rects: BTreeMap::new(),
            main_directions: BTreeMap::new(),
            country_tech_levels: BTreeMap::new(),
            exile_rects: BTreeMap::new(),
        }
    }

    pub fn initialize(
        &mut self,
        source: Option<&[u8]>,
    ) -> Result<CountryParamLoadReport, CountryParamLoadError> {
        self.load(source)
    }

    pub fn load(
        &mut self,
        source: Option<&[u8]>,
    ) -> Result<CountryParamLoadReport, CountryParamLoadError> {
        self.start_regions.clear();
        self.start_rects.clear();
        self.start_directions.clear();
        self.main_regions.clear();
        self.main_rects.clear();
        self.main_directions.clear();

        let Some(source) = source else {
            return Ok(CountryParamLoadReport {
                resource_found: false,
                legacy_result: true,
            });
        };
        let mut tokens = source
            .split(u8::is_ascii_whitespace)
            .filter(|token| !token.is_empty());

        for (index, field) in COUNTRY_PARAMETER_FIELDS.into_iter().enumerate() {
            let _label = next_country_token(&mut tokens, field)?;
            self.parameters[index] = Some(next_country_i32(&mut tokens, field)?);
        }

        while read_to(&mut tokens, b"*") {
            let country = next_country_i32(&mut tokens, "country record ID")? as u8;
            let start_region = next_country_i32(&mut tokens, "start region ID")?;
            let start_rect = read_country_rect(&mut tokens, "start rect")?;
            let start_direction = next_country_i32(&mut tokens, "start direction")?;
            let main_region = next_country_i32(&mut tokens, "main region ID")?;
            let main_rect = read_country_rect(&mut tokens, "main rect")?;
            let main_direction = next_country_i32(&mut tokens, "main direction")?;

            self.start_regions.insert(country, start_region);
            self.start_rects.insert(country, start_rect);
            self.start_directions.insert(country, start_direction);
            self.main_regions.insert(country, main_region);
            self.main_rects.insert(country, main_rect);
            self.main_directions.insert(country, main_direction);
        }

        while read_to(&mut tokens, b"#") {
            let level = next_country_i32(&mut tokens, "technology level")?;
            let country_tech_exp = next_country_i32(&mut tokens, "country technology exp")?;
            let country_power = next_country_i32(&mut tokens, "technology country power")?;
            self.country_tech_levels.insert(
                level,
                CountryTechLevel {
                    country_tech_exp,
                    country_power,
                },
            );
        }

        while read_to(&mut tokens, b"+") {
            let country = next_country_i32(&mut tokens, "exile country ID")? as u8;
            let rect = read_country_rect(&mut tokens, "exile rect")?;
            self.exile_rects.insert(country, rect);
        }

        Ok(CountryParamLoadReport {
            resource_found: true,
            legacy_result: true,
        })
    }

    pub fn main_return_point(&mut self, country: u8) -> CountryReturnPoint {
        let region_id = *self.main_regions.entry(country).or_insert(0);
        let rect = *self.main_rects.entry(country).or_default();
        let direction = *self.main_directions.entry(country).or_insert(0);
        CountryReturnPoint {
            region_id,
            rect,
            direction,
        }
    }

    pub fn start_region_or_insert(&mut self, country: u8) -> i32 {
        *self.start_regions.entry(country).or_insert(0)
    }

    /// Возвращает rect, который create-role читает только после успешного
    /// lookup живого region-owner-а и до выбора случайной клетки.
    pub fn start_rect_or_insert(&mut self, country: u8) -> CountryRect {
        *self.start_rects.entry(country).or_default()
    }

    pub fn start_direction_or_insert(&mut self, country: u8) -> i32 {
        *self.start_directions.entry(country).or_insert(0)
    }

    pub const fn max_country_power(&self) -> Option<i32> {
        self.parameters[MAX_COUNTRY_POWER]
    }

    pub const fn max_country_treasury(&self) -> Option<i32> {
        self.parameters[MAX_COUNTRY_TREASURY]
    }

    pub const fn daily_country_treasury(&self) -> Option<i32> {
        self.parameters[DAILY_COUNTRY_TREASURY]
    }

    pub const fn max_king_control_point(&self) -> Option<i32> {
        self.parameters[MAX_KING_CONTROL_POINT]
    }

    pub const fn min_king_control_point(&self) -> Option<i32> {
        self.parameters[MIN_KING_CONTROL_POINT]
    }

    pub const fn king_control_point_decay_interval(&self) -> Option<i32> {
        self.parameters[DEC_KING_CONTROL_POINT_INTERVAL]
    }

    pub const fn king_control_point_decay(&self) -> Option<i32> {
        self.parameters[DEC_KING_CONTROL_POINT_TIME]
    }

    pub const fn king_need_level(&self) -> Option<i32> {
        self.parameters[KING_NEED_LEVEL]
    }

    pub const fn king_need_credit(&self) -> Option<i32> {
        self.parameters[KING_NEED_CREDIT]
    }

    pub const fn default_king_control_point(&self) -> Option<i32> {
        self.parameters[DEFAULT_KING_CONTROL_POINT]
    }

    pub const fn demise_required_control_point(&self) -> Option<i32> {
        self.parameters[DEFAULT_KING_CONTROL_POINT_DEMISE_NEED]
    }

    pub const fn silence_control_point_cost(&self) -> Option<i32> {
        self.parameters[DEC_KING_CONTROL_POINT_SILENCE]
    }

    pub const fn demise_control_point_cost(&self) -> Option<i32> {
        self.parameters[DEC_KING_CONTROL_POINT_DEMISE]
    }

    pub const fn appoint_control_point_cost(&self) -> Option<i32> {
        self.parameters[DEC_KING_CONTROL_POINT_APPOINT]
    }

    pub const fn max_silence_count(&self) -> Option<i32> {
        self.parameters[MAX_SILENCE_NUM]
    }

    pub const fn silence_time(&self) -> Option<i32> {
        self.parameters[SILENCE_TIME]
    }

    pub const fn exile_control_point_cost(&self) -> Option<i32> {
        self.parameters[DEC_KING_CONTROL_POINT_EXILE]
    }

    pub const fn absolve_control_point_cost(&self) -> Option<i32> {
        self.parameters[DEC_KING_CONTROL_POINT_ABSOLVE]
    }

    pub const fn max_absolve_count(&self) -> Option<i32> {
        self.parameters[MAX_ABSOLVE_NUM]
    }

    pub const fn max_king_material_point(&self) -> Option<i32> {
        self.parameters[MAX_KING_MATERIAL_POINT]
    }

    pub const fn max_king_war_point(&self) -> Option<i32> {
        self.parameters[MAX_KING_WAR_POINT]
    }

    pub const fn exile_time_ms(&self) -> Option<i32> {
        self.parameters[EXILE_TIME]
    }

    pub const fn max_exile_count(&self) -> Option<i32> {
        self.parameters[MAX_EXILE_NUM]
    }

    pub const fn max_exile_pk(&self) -> Option<i32> {
        self.parameters[MAX_EXILE_PK]
    }

    pub fn has_exile_rect(&self, country: u8) -> bool {
        self.exile_rects.contains_key(&country)
    }

    pub fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<bool, CountryParamSerializationBlock> {
        for (index, field) in COUNTRY_PARAMETER_FIELDS.into_iter().enumerate() {
            let value = self.parameters[index]
                .ok_or(CountryParamSerializationBlock::UninitializedParameter { field })?;
            destination.extend_from_slice(&value.to_le_bytes());
        }

        append_country_count(destination, "m_mpMainRegions", self.main_regions.len())?;
        for (&country, &region_id) in &self.main_regions {
            destination.push(country);
            destination.extend_from_slice(&region_id.to_le_bytes());
        }

        append_country_count(destination, "m_mpMainRect", self.main_rects.len())?;
        for (&country, &rect) in &self.main_rects {
            destination.push(country);
            append_country_rect(destination, rect);
        }

        append_country_count(destination, "m_mpMainDir", self.main_directions.len())?;
        for (&country, &direction) in &self.main_directions {
            destination.push(country);
            destination.extend_from_slice(&direction.to_le_bytes());
        }

        append_country_count(
            destination,
            "_country_tech_lels",
            self.country_tech_levels.len(),
        )?;
        for (&level, tech) in &self.country_tech_levels {
            destination.extend_from_slice(&level.to_le_bytes());
            destination.extend_from_slice(&tech.country_power.to_le_bytes());
            destination.extend_from_slice(&tech.country_tech_exp.to_le_bytes());
        }

        append_country_count(destination, "m_mapExile", self.exile_rects.len())?;
        for (&country, &rect) in &self.exile_rects {
            destination.push(country);
            append_country_rect(destination, rect);
        }
        Ok(true)
    }
}

fn read_country_rect<'a, Tokens>(
    tokens: &mut Tokens,
    field: &'static str,
) -> Result<CountryRect, CountryParamLoadError>
where
    Tokens: Iterator<Item = &'a [u8]>,
{
    Ok(CountryRect {
        left: next_country_i32(tokens, field)?,
        top: next_country_i32(tokens, field)?,
        right: next_country_i32(tokens, field)?,
        bottom: next_country_i32(tokens, field)?,
    })
}

fn next_country_i32<'a, Tokens>(
    tokens: &mut Tokens,
    field: &'static str,
) -> Result<i32, CountryParamLoadError>
where
    Tokens: Iterator<Item = &'a [u8]>,
{
    let token = next_country_token(tokens, field)?;
    std::str::from_utf8(token)
        .ok()
        .and_then(|value| value.parse().ok())
        .ok_or(CountryParamLoadError::InvalidValue { field })
}

fn next_country_token<'a, Tokens>(
    tokens: &mut Tokens,
    field: &'static str,
) -> Result<&'a [u8], CountryParamLoadError>
where
    Tokens: Iterator<Item = &'a [u8]>,
{
    tokens
        .next()
        .ok_or(CountryParamLoadError::MissingValue { field })
}

fn append_country_count(
    destination: &mut Vec<u8>,
    collection: &'static str,
    count: usize,
) -> Result<(), CountryParamSerializationBlock> {
    let count = i32::try_from(count)
        .map_err(|_| CountryParamSerializationBlock::TooManyEntries { collection, count })?;
    destination.extend_from_slice(&count.to_le_bytes());
    Ok(())
}

fn append_country_rect(destination: &mut Vec<u8>, rect: CountryRect) {
    destination.extend_from_slice(&rect.left.to_le_bytes());
    destination.extend_from_slice(&rect.top.to_le_bytes());
    destination.extend_from_slice(&rect.right.to_le_bytes());
    destination.extend_from_slice(&rect.bottom.to_le_bytes());
}
