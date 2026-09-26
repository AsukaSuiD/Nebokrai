//! Honor Ranks DB-типы (records/snapshots/sink/outcome) исходного
//! мирового `CRsPlayer`.
//!
//! Источник — точная пара `Nworldserver.exe` + `WorldServer.pdb`.
//! Layout `HonorRankDbEntry` 0x24 байт сохранён `repr(C)` и const-asserts.

use std::error::Error;
use std::fmt;
use std::mem::{offset_of, size_of};

use chrono::{Datelike, Days, NaiveDate};

pub const HONOR_RANK_CATEGORY_COUNT: usize = 4;
pub const HONOR_RANK_TYPE_COUNT: usize = 4;
pub const HONOR_RANK_ENTRY_SIZE: usize = 0x24;
pub const HONOR_RANK_BLOB_HEADER_SIZE: usize = HONOR_RANK_CATEGORY_COUNT * size_of::<u32>();

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(C)]
pub struct HonorRankDbEntry {
    pub player_id: i32,
    pub level: u8,
    pub name: [u8; 20],
    pub occupation_id: u8,
    pub legacy_padding: [u8; 2],
    pub appellation_id: u32,
    pub eliminate_num: u32,
}

const _: [(); HONOR_RANK_ENTRY_SIZE] = [(); size_of::<HonorRankDbEntry>()];
const _: [(); 0x00] = [(); offset_of!(HonorRankDbEntry, player_id)];
const _: [(); 0x04] = [(); offset_of!(HonorRankDbEntry, level)];
const _: [(); 0x05] = [(); offset_of!(HonorRankDbEntry, name)];
const _: [(); 0x19] = [(); offset_of!(HonorRankDbEntry, occupation_id)];
const _: [(); 0x1a] = [(); offset_of!(HonorRankDbEntry, legacy_padding)];
const _: [(); 0x1c] = [(); offset_of!(HonorRankDbEntry, appellation_id)];
const _: [(); 0x20] = [(); offset_of!(HonorRankDbEntry, eliminate_num)];

impl HonorRankDbEntry {
    pub fn legacy_bytes(self) -> [u8; HONOR_RANK_ENTRY_SIZE] {
        let mut bytes = [0; HONOR_RANK_ENTRY_SIZE];
        bytes[0x00..0x04].copy_from_slice(&self.player_id.to_le_bytes());
        bytes[0x04] = self.level;
        bytes[0x05..0x19].copy_from_slice(&self.name);
        bytes[0x19] = self.occupation_id;
        bytes[0x1a..0x1c].copy_from_slice(&self.legacy_padding);
        bytes[0x1c..0x20].copy_from_slice(&self.appellation_id.to_le_bytes());
        bytes[0x20..0x24].copy_from_slice(&self.eliminate_num.to_le_bytes());
        bytes
    }

    pub fn from_legacy_bytes(bytes: &[u8; HONOR_RANK_ENTRY_SIZE]) -> Self {
        Self {
            player_id: i32::from_le_bytes(bytes[0x00..0x04].try_into().expect("fixed entry")),
            level: bytes[0x04],
            name: bytes[0x05..0x19].try_into().expect("fixed entry"),
            occupation_id: bytes[0x19],
            legacy_padding: bytes[0x1a..0x1c].try_into().expect("fixed entry"),
            appellation_id: u32::from_le_bytes(
                bytes[0x1c..0x20].try_into().expect("fixed entry"),
            ),
            eliminate_num: u32::from_le_bytes(
                bytes[0x20..0x24].try_into().expect("fixed entry"),
            ),
        }
    }
}

pub type HonorRankDbLists =
    [[Vec<HonorRankDbEntry>; HONOR_RANK_CATEGORY_COUNT]; HONOR_RANK_TYPE_COUNT];

#[derive(Clone, Copy, Debug)]
#[allow(
    dead_code,
    reason = "поля hour/minute/second/milliseconds переносятся как часть legacy SYSTEMTIME layout; SQL-путь читает только дату через legacy_sql_date/next_day, как в оригинале"
)]
pub struct HonorRanksCopyTimeSnapshot {
    year: u16,
    month: u16,
    day_of_week: u16,
    day: u16,
    hour: u16,
    minute: u16,
    second: u16,
    milliseconds: u16,
}

pub struct HonorRanksDbDataSnapshot {
    copy_time: HonorRanksCopyTimeSnapshot,
    history: HonorRankDbLists,
    current: HonorRankDbLists,
}

impl HonorRanksDbDataSnapshot {
    pub fn from_legacy_copy(
        copy_time: HonorRanksCopyTimeSnapshot,
        history: HonorRankDbLists,
        current: HonorRankDbLists,
    ) -> Self {
        Self {
            copy_time,
            history,
            current,
        }
    }

    pub const fn copy_time(&self) -> HonorRanksCopyTimeSnapshot {
        self.copy_time
    }

    pub fn lists_mut(
        &mut self,
        period: HonorRanksSavePeriod,
        rank_type: HonorRanksType,
    ) -> &mut [Vec<HonorRankDbEntry>; HONOR_RANK_CATEGORY_COUNT] {
        let type_index = rank_type as usize;
        match period {
            HonorRanksSavePeriod::History => &mut self.history[type_index],
            HonorRanksSavePeriod::Current => &mut self.current[type_index],
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HonorRanksSavePeriod {
    Current,
    History,
}

pub trait HonorRanksLoadSink {
    fn clear_honor_ranks_period(&mut self, period: HonorRanksSavePeriod);
    fn replace_honor_ranks_type(
        &mut self,
        period: HonorRanksSavePeriod,
        rank_type: HonorRanksType,
        lists: [Vec<HonorRankDbEntry>; HONOR_RANK_CATEGORY_COUNT],
    );
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HonorRanksBlobDecodeBlock {
    pub rank_type: HonorRanksType,
    pub country: u8,
    pub offset: usize,
    pub required_bytes: usize,
    pub available_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HonorRanksLoadFailure {
    MissingConnection,
    Database { period: HonorRanksSavePeriod },
    MissingRow { period: HonorRanksSavePeriod },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HonorRanksLoadBlock {
    pub period: HonorRanksSavePeriod,
    pub source: HonorRanksBlobDecodeBlock,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HonorRanksLoadOutcome {
    ReturnedTrue,
    ReturnedFalse(HonorRanksLoadFailure),
    BlockedMissingFact(HonorRanksLoadBlock),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(usize)]
pub enum HonorRanksType {
    Day = 0,
    Week = 1,
    Month = 2,
    Total = 3,
}

impl HonorRanksType {
    pub const ALL: [Self; HONOR_RANK_TYPE_COUNT] = [Self::Day, Self::Week, Self::Month, Self::Total];

    pub const fn column_name(self) -> &'static str {
        match self {
            Self::Day => "DayHonnorRank",
            Self::Week => "WeekHonnorRank",
            Self::Month => "MonthHonnorRank",
            Self::Total => "TotalHonnorRank",
        }
    }
}

impl fmt::Display for HonorRanksBlobDecodeBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "honor blob {:?}, страна {}, offset {}: требуется {} байт, доступно {}",
            self.rank_type,
            self.country,
            self.offset,
            self.required_bytes,
            self.available_bytes
        )
    }
}

impl Error for HonorRanksBlobDecodeBlock {}

impl HonorRanksCopyTimeSnapshot {
    pub fn from_legacy_fields(fields: [u16; 8]) -> Option<Self> {
        let [
            year,
            month,
            day_of_week,
            day,
            hour,
            minute,
            second,
            milliseconds,
        ] = fields;
        NaiveDate::from_ymd_opt(i32::from(year), u32::from(month), u32::from(day))?;
        Some(Self {
            year,
            month,
            day_of_week,
            day,
            hour,
            minute,
            second,
            milliseconds,
        })
    }

    pub fn next_day(self) -> Self {
        if self.year == u16::MAX && self.month == 12 && self.day == 31 {
            return Self {
                year: 0,
                month: 1,
                day_of_week: self.day_of_week.wrapping_add(1) % 7,
                day: 1,
                ..self
            };
        }

        let date = NaiveDate::from_ymd_opt(
            i32::from(self.year),
            u32::from(self.month),
            u32::from(self.day),
        )
        .expect("HonorRanksCopyTimeSnapshot сохраняет валидную дату");
        let next = date
            .checked_add_days(Days::new(1))
            .expect("диапазон u16 года помещается в chrono::NaiveDate");
        Self {
            year: u16::try_from(next.year()).expect("переполнение u16 обработано отдельной веткой"),
            month: u16::try_from(next.month()).expect("месяц помещается в u16"),
            day_of_week: self.day_of_week.wrapping_add(1) % 7,
            day: u16::try_from(next.day()).expect("день помещается в u16"),
            ..self
        }
    }

    pub fn legacy_sql_date(self) -> String {
        format!("{}-{}-{}", self.year, self.month, self.day)
    }
}
