//! GameServer-владелец `CCountryParam`, подтверждённый точными
//! `gameserver.exe + GameServer.pdb`; исходники
//! `server/gameserver/appserver/country/countryparam.cpp/.h`.
//!
//! Game decoder позиционно присваивает 39 signed scalar-параметров, затем
//! очищает пять передаваемых maps и читает main return points, technology
//! levels и exile rects. Start maps Game decoder не получает. Полный scalar
//! prefix сохраняет частичное присваивание; maps очищаются только после него,
//! а malformed collection оставляет полный опубликованный префикс.
//!
//! Wire technology record World имеет порядок `level, country_power,
//! country_tech_exp`, но exact Game EXE потребляет только младший byte
//! четырёхбайтного level и присваивает первый value в `_country_tech_exp`, а
//! второй в `_country_power`. Эта несовместимая странность сохранена явно.
//! Main maps и technology используют `operator[]` (last-wins), exile —
//! `map::insert` (first-wins). Process singleton технически заменён owned-полем
//! `CGame`; неизвестные до snapshot constructor-scalars выражены `None`, а
//! `BTreeMap` сохраняет ordered-map lookup.
//! Один и тот же четырёх-DWORD exile wire World использует как rect только для
//! проверки наличия, а Game `OnCountryMessage(0x7FF0E)` читает как
//! `(region_id, x, y, legacy_fourth)`; эта межсерверная асимметрия выражена
//! отдельным accessor-ом без изменения формата snapshot-а.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

const COUNTRY_PARAMETER_COUNT: usize = 39;
const MAX_COUNTRY_POWER: usize = 2;
const MAX_COUNTRY_TREASURY: usize = 4;
const MAX_KING_MATERIAL_POINT: usize = 26;
const SILENCE_TIME: usize = 30;
const EXILE_TIME: usize = 32;
const MAX_EXPLOIT: usize = 36;
const INC_EXPLOIT: usize = 38;

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
pub(crate) struct CountryMainRect {
    pub(crate) left: i32,
    pub(crate) top: i32,
    pub(crate) right: i32,
    pub(crate) bottom: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryExilePoint {
    pub(crate) region_id: i32,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) legacy_fourth: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryTechLevel {
    pub(crate) country_tech_exp: i32,
    pub(crate) country_power: i32,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CountryMainReturnPoint {
    pub(crate) region_id: i32,
    pub(crate) rect: CountryMainRect,
    pub(crate) direction: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryParamDecodeReport {
    pub(crate) main_regions: usize,
    pub(crate) main_rects: usize,
    pub(crate) main_directions: usize,
    pub(crate) technology_levels: usize,
    pub(crate) exile_rects: usize,
}

/// Старый decoder не получал размер buffer-а; safe Rust останавливается в
/// точной достигнутой позиции вместо чтения за границей.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryParamInputBlock {
    pub(crate) field: &'static str,
    pub(crate) offset: usize,
    pub(crate) required: usize,
    pub(crate) available: usize,
}

impl fmt::Display for CountryParamInputBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "CountryParam snapshot обрывается на {} в {}: нужно {}, доступно {}",
            self.field, self.offset, self.required, self.available
        )
    }
}

impl Error for CountryParamInputBlock {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CCountryParam {
    parameters: [Option<i32>; COUNTRY_PARAMETER_COUNT],
    main_regions: BTreeMap<u8, i32>,
    main_rects: BTreeMap<u8, CountryMainRect>,
    main_directions: BTreeMap<u8, i32>,
    country_tech_levels: BTreeMap<i32, CountryTechLevel>,
    exile_rects: BTreeMap<u8, CountryMainRect>,
}

impl Default for CCountryParam {
    fn default() -> Self {
        Self {
            parameters: [None; COUNTRY_PARAMETER_COUNT],
            main_regions: BTreeMap::new(),
            main_rects: BTreeMap::new(),
            main_directions: BTreeMap::new(),
            country_tech_levels: BTreeMap::new(),
            exile_rects: BTreeMap::new(),
        }
    }
}

impl CCountryParam {
    /// Материализует полный Game decoder с исходным порядком side effects.
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<CountryParamDecodeReport, CountryParamInputBlock> {
        for (index, field) in COUNTRY_PARAMETER_FIELDS.into_iter().enumerate() {
            self.parameters[index] = Some(read_country_i32(source, cursor, field)?);
        }

        self.main_regions.clear();
        self.main_rects.clear();
        self.main_directions.clear();
        self.country_tech_levels.clear();
        self.exile_rects.clear();

        let region_count = read_country_i32(source, cursor, "m_mpMainRegions count")?;
        for _ in 0..region_count.max(0) {
            let country = read_country_u8(source, cursor, "m_mpMainRegions country")?;
            let region_id = read_country_i32(source, cursor, "m_mpMainRegions region")?;
            self.main_regions.insert(country, region_id);
        }

        let rect_count = read_country_i32(source, cursor, "m_mpMainRect count")?;
        for _ in 0..rect_count.max(0) {
            let country = read_country_u8(source, cursor, "m_mpMainRect country")?;
            let rect = read_country_rect(source, cursor, "m_mpMainRect")?;
            self.main_rects.insert(country, rect);
        }

        let direction_count = read_country_i32(source, cursor, "m_mpMainDir count")?;
        for _ in 0..direction_count.max(0) {
            let country = read_country_u8(source, cursor, "m_mpMainDir country")?;
            let direction = read_country_i32(source, cursor, "m_mpMainDir direction")?;
            self.main_directions.insert(country, direction);
        }

        let technology_count = read_country_i32(source, cursor, "_country_tech_lels count")?;
        for _ in 0..technology_count.max(0) {
            let wire_level = read_country_i32(source, cursor, "technology level")?;
            let level = i32::from(wire_level as u8);
            let country_tech_exp =
                read_country_i32(source, cursor, "Game technology exp / World country power")?;
            let country_power =
                read_country_i32(source, cursor, "Game country power / World technology exp")?;
            self.country_tech_levels.insert(
                level,
                CountryTechLevel {
                    country_tech_exp,
                    country_power,
                },
            );
        }

        let exile_count = read_country_i32(source, cursor, "m_mapExile count")?;
        for _ in 0..exile_count.max(0) {
            let country = read_country_u8(source, cursor, "m_mapExile country")?;
            let rect = read_country_rect(source, cursor, "m_mapExile")?;
            self.exile_rects.entry(country).or_insert(rect);
        }

        Ok(CountryParamDecodeReport {
            main_regions: self.main_regions.len(),
            main_rects: self.main_rects.len(),
            main_directions: self.main_directions.len(),
            technology_levels: self.country_tech_levels.len(),
            exile_rects: self.exile_rects.len(),
        })
    }

    pub(crate) fn parameter(&self, index: usize) -> Option<i32> {
        self.parameters.get(index).copied().flatten()
    }

    pub(crate) const fn max_country_treasury(&self) -> Option<i32> {
        self.parameters[MAX_COUNTRY_TREASURY]
    }

    pub(crate) const fn max_country_power(&self) -> Option<i32> {
        self.parameters[MAX_COUNTRY_POWER]
    }

    pub(crate) const fn max_king_material_point(&self) -> Option<i32> {
        self.parameters[MAX_KING_MATERIAL_POINT]
    }

    pub(crate) const fn silence_time(&self) -> Option<i32> {
        self.parameters[SILENCE_TIME]
    }

    pub(crate) const fn exile_time(&self) -> Option<i32> {
        self.parameters[EXILE_TIME]
    }

    pub(crate) const fn max_exploit(&self) -> Option<i32> {
        self.parameters[MAX_EXPLOIT]
    }

    pub(crate) const fn exploit_increment(&self) -> Option<i32> {
        self.parameters[INC_EXPLOIT]
    }

    pub(crate) fn country_tech_level(&self, level: i32) -> Option<&CountryTechLevel> {
        self.country_tech_levels.get(&level)
    }

    pub(crate) fn country_tech_level_count(&self) -> i32 {
        self.country_tech_levels.len() as i32
    }

    pub(crate) fn exile_rect(&self, country: u8) -> Option<CountryMainRect> {
        self.exile_rects.get(&country).copied()
    }

    pub(crate) fn exile_point(&self, country: u8) -> Option<CountryExilePoint> {
        self.exile_rect(country).map(|wire| CountryExilePoint {
            region_id: wire.left,
            x: wire.top,
            y: wire.right,
            legacy_fourth: wire.bottom,
        })
    }

    pub(crate) fn exile_rects(&self) -> &BTreeMap<u8, CountryMainRect> {
        &self.exile_rects
    }

    /// Сохраняет три mutating `operator[]` lookup-а оригинала.
    pub(crate) fn main_return_point(&mut self, country: u8) -> CountryMainReturnPoint {
        let region_id = *self.main_regions.entry(country).or_insert(0);
        let rect = *self.main_rects.entry(country).or_default();
        let direction = *self.main_directions.entry(country).or_insert(0);
        CountryMainReturnPoint {
            region_id,
            rect,
            direction,
        }
    }
}

fn read_country_rect(
    source: &[u8],
    cursor: &mut usize,
    collection: &'static str,
) -> Result<CountryMainRect, CountryParamInputBlock> {
    let field = match collection {
        "m_mpMainRect" => [
            "m_mpMainRect.left",
            "m_mpMainRect.top",
            "m_mpMainRect.right",
            "m_mpMainRect.bottom",
        ],
        _ => [
            "m_mapExile.left",
            "m_mapExile.top",
            "m_mapExile.right",
            "m_mapExile.bottom",
        ],
    };
    Ok(CountryMainRect {
        left: read_country_i32(source, cursor, field[0])?,
        top: read_country_i32(source, cursor, field[1])?,
        right: read_country_i32(source, cursor, field[2])?,
        bottom: read_country_i32(source, cursor, field[3])?,
    })
}

fn read_country_u8(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u8, CountryParamInputBlock> {
    Ok(take_country_bytes(source, cursor, 1, field)?[0])
}

fn read_country_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, CountryParamInputBlock> {
    let bytes = take_country_bytes(source, cursor, 4, field)?;
    Ok(i32::from_le_bytes(
        bytes.try_into().expect("проверенный 4-байтовый срез"),
    ))
}

fn take_country_bytes<'a>(
    source: &'a [u8],
    cursor: &mut usize,
    count: usize,
    field: &'static str,
) -> Result<&'a [u8], CountryParamInputBlock> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(count) else {
        return Err(CountryParamInputBlock {
            field,
            offset,
            required: count,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        return Err(CountryParamInputBlock {
            field,
            offset,
            required: count,
            available,
        });
    };
    *cursor = end;
    Ok(bytes)
}
