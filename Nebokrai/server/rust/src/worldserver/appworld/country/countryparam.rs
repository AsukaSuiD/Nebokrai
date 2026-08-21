//! Владелец параметров стран исторического `WorldServer`.
//!
//! Owner `CCountryParam`; constructor `0x000445B0`, `Load` `0x00043C70`,
//! `Initialize` `0x00044750` и `AddToByteArray` `0x00041F70` имеют статус
//! `IMPLEMENTED`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countryparam.h`
//! и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\countryparam.cpp:19,27,37,152`.
//!
//! Старый размер класса `0xFC`: первые `0x9C` bytes — 39 signed параметров,
//! которым constructor не назначал значения, затем восемь `std::map` по
//! offsets `0x9C..0xF0`. Rust хранит неизвестные scalars как `Option<i32>`, а
//! ordered map — как `BTreeMap`; process-global lazy singleton заменён явным
//! owner-ом без изменения его данных или порядка обхода.
//!
//! `Load` сначала очищает только шесть start/main maps, но сохраняет прежние
//! technology/exile entries, затем читает `data/CountryParam.ini`: 39 пар
//! label/value, records `*` для start/main, `#` для technology и `+` для exile.
//! Повторяющиеся ключи перезаписываются как `operator[]`; country ID сужается
//! до `u8`. Поставочный loose fixture длиной 2339 bytes имеет SHA-256
//! `F627CBC56F66B021C72A9688D4340618200A97C3BDCEC4C2C762E70B759EE898` и даёт
//! четыре country, три technology и четыре exile записи. Отсутствующий ресурс
//! оставляет scalars/technology/exile как были, но уже очищает шесть maps и
//! сообщает caller-у необходимость старого log.
//!
//! `Initialize` в exact EXE является прямым jump на `Load`; общий epilogue
//! `0x0044458D` всегда возвращает `true`, в том числе после missing resource.
//! Это `VERIFIED_DISASSEMBLY`. Malformed numeric input оставлен локальным
//! `BLOCKED_MISSING_FACT`: formatted extraction мог продолжить с прежними либо
//! неинициализированными locals, поэтому safe Rust сохраняет уже выполненные
//! мутации и останавливает только эту границу.
//!
//! Wire содержит 39 DWORD, затем только main-region/main-rect/main-dir,
//! technology и exile maps с signed DWORD counts. Technology entry имеет
//! наблюдаемый порядок `level, country_power, country_tech_exp`, отличный от
//! порядка полей при чтении. Start maps в wire не входят. STL tree/allocation,
//! SEH, `Unwind@...` и попавшие в этот файл шаблоны соседнего CountryWar
//! классифицированы как library/compiler noise и после реализации удалены.

use std::collections::BTreeMap;

use crate::public::readwrite::read_to;

const COUNTRY_PARAMETER_COUNT: usize = 39;
const MAX_COUNTRY_POWER: usize = 2;
const MAX_COUNTRY_TREASURY: usize = 4;
const MAX_KING_CONTROL_POINT: usize = 10;
const MIN_KING_CONTROL_POINT: usize = 11;
const DEC_KING_CONTROL_POINT_SILENCE: usize = 18;
const DEC_KING_CONTROL_POINT_EXILE: usize = 20;
const DEC_KING_CONTROL_POINT_ABSOLVE: usize = 21;
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

/// Точные четыре signed поля старого `tagRECT`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryRect {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CountryTechLevel {
    country_tech_exp: i32,
    country_power: i32,
}

/// Согласованный результат трёх `operator[]`, читаемых точкой возврата.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryReturnPoint {
    pub(crate) region_id: i32,
    pub(crate) rect: CountryRect,
    pub(crate) direction: i32,
}

/// Наблюдаемый результат старого resource-open и неизменный legacy bool.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryParamLoadReport {
    pub(crate) resource_found: bool,
    pub(crate) legacy_result: bool,
}

/// Safe-граница formatted extraction старого text-loader-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryParamLoadError {
    MissingValue { field: &'static str },
    InvalidValue { field: &'static str },
}

/// Граница byte-array для constructor-неизвестных полей и 32-битных counts.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryParamSerializationBlock {
    UninitializedParameter {
        field: &'static str,
    },
    TooManyEntries {
        collection: &'static str,
        count: usize,
    },
}

/// Safe-граница чтения constructor-неизвестного scalar-параметра.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryParameterUnavailable {
    pub(crate) field: &'static str,
}

/// Полный достигнутый state исходного `CCountryParam`.
pub(crate) struct CCountryParam {
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
    /// Создаёт точный constructor-state: восемь пустых maps и неизвестные scalars.
    pub(crate) const fn new() -> Self {
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

    /// Выполняет точный `Initialize`, являющийся jump на `Load`.
    pub(crate) fn initialize(
        &mut self,
        source: Option<&[u8]>,
    ) -> Result<CountryParamLoadReport, CountryParamLoadError> {
        self.load(source)
    }

    /// Загружает уже разрешённый `data/CountryParam.ini` либо missing resource.
    pub(crate) fn load(
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

    /// Возвращает main return-point и сохраняет zero-insertion трёх `operator[]`.
    pub(crate) fn main_return_point(&mut self, country: u8) -> CountryReturnPoint {
        let region_id = *self.main_regions.entry(country).or_insert(0);
        let rect = *self.main_rects.entry(country).or_default();
        let direction = *self.main_directions.entry(country).or_insert(0);
        CountryReturnPoint {
            region_id,
            rect,
            direction,
        }
    }

    pub(crate) const fn max_country_power(&self) -> Option<i32> {
        self.parameters[MAX_COUNTRY_POWER]
    }

    pub(crate) const fn max_country_treasury(&self) -> Option<i32> {
        self.parameters[MAX_COUNTRY_TREASURY]
    }

    /// Возвращает достигнутый максимум king control point без default-подстановки.
    pub(crate) const fn max_king_control_point(&self) -> Option<i32> {
        self.parameters[MAX_KING_CONTROL_POINT]
    }

    pub(crate) const fn min_king_control_point(&self) -> Option<i32> {
        self.parameters[MIN_KING_CONTROL_POINT]
    }

    /// Возвращает exact `_dec_king_control_point_silence` без default-подстановки.
    pub(crate) const fn silence_control_point_cost(&self) -> Option<i32> {
        self.parameters[DEC_KING_CONTROL_POINT_SILENCE]
    }

    pub(crate) const fn max_silence_count(&self) -> Option<i32> {
        self.parameters[MAX_SILENCE_NUM]
    }

    pub(crate) const fn silence_time(&self) -> Option<i32> {
        self.parameters[SILENCE_TIME]
    }

    /// Возвращает exact `_dec_king_control_point_exile` без default-подстановки.
    pub(crate) const fn exile_control_point_cost(&self) -> Option<i32> {
        self.parameters[DEC_KING_CONTROL_POINT_EXILE]
    }

    pub(crate) const fn absolve_control_point_cost(&self) -> Option<i32> {
        self.parameters[DEC_KING_CONTROL_POINT_ABSOLVE]
    }

    pub(crate) const fn max_absolve_count(&self) -> Option<i32> {
        self.parameters[MAX_ABSOLVE_NUM]
    }

    /// Возвращает достигнутый максимум king material point без default-подстановки.
    pub(crate) const fn max_king_material_point(&self) -> Option<i32> {
        self.parameters[MAX_KING_MATERIAL_POINT]
    }

    /// Возвращает достигнутый максимум king war point без default-подстановки.
    pub(crate) const fn max_king_war_point(&self) -> Option<i32> {
        self.parameters[MAX_KING_WAR_POINT]
    }

    pub(crate) const fn exile_time_ms(&self) -> Option<i32> {
        self.parameters[EXILE_TIME]
    }

    pub(crate) const fn max_exile_count(&self) -> Option<i32> {
        self.parameters[MAX_EXILE_NUM]
    }

    pub(crate) const fn max_exile_pk(&self) -> Option<i32> {
        self.parameters[MAX_EXILE_PK]
    }

    pub(crate) fn has_exile_rect(&self, country: u8) -> bool {
        self.exile_rects.contains_key(&country)
    }

    /// Дописывает полный country-parameter wire в исходном порядке.
    pub(crate) fn add_to_byte_array(
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
