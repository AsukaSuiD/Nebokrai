//! Data-контракты карты государств `CCountryHandler` из
//! `countryhandler.cpp/.h`, подтверждённые точной парой `worldserver.exe` и
//! `worldserver.pdb`. Data-уровень перенесён в Realm `organizations/`.
//!
//! Run-, new-day- и initialize-отчёты перенесены вместе с data-семьями стран;
//! servermessage-волна добавила `CountryHandlerSerializeError` к уже
//! перенесённому `CountrySerializeError`. `CountryAppendDisposition` ссылается
//! на саму `CCountry` и переносится вместе с ней в волне владельца.

use std::error::Error;
use std::fmt;

use crate::app::world_message::CMessage;
use crate::organizations::country::{
    CountryAiBlock, CountryAiReport, CountrySerializeError, CountrySetNewDayReport,
};

/// Наблюдаемые блоки `CCountryHandler::Run` после устранения внутреннего
/// null-slot lifecycle-дефекта.
#[derive(Debug, Eq, PartialEq)]
pub enum CountryRunBlock {
    Ai {
        map_key: u8,
        source: CountryAiBlock,
    },
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryRunReport {
    pub expired_top_info_ids: Vec<i32>,
    pub ai_country_ids: Vec<u8>,
    pub ai_reports: Vec<(u8, CountryAiReport)>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryHandlerNewDayEntry {
    pub map_key: u8,
    pub report: CountrySetNewDayReport,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryHandlerNewDayReport {
    pub requested_day: i32,
    pub countries: Vec<CountryHandlerNewDayEntry>,
    pub skipped_null_country_keys: Vec<u8>,
}

#[derive(Debug, Eq, PartialEq)]
pub struct CountryHandlerInitializeReport {
    pub local_day: i32,
    pub database_loaded: bool,
    pub new_day: Option<CountryHandlerNewDayReport>,
    pub legacy_result: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CountryHandlerReleaseReport {
    pub released_countries: usize,
    pub released_top_infos: usize,
}

pub trait CountryInfoDeliveryContext {
    fn send_all(&mut self, message: &CMessage) -> i32;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CountryHandlerSerializeError {
    CountryCountOutOfRange { country_count: usize },
    NullCountry { map_key: u8 },
    Country {
        map_key: u8,
        source: CountrySerializeError,
    },
}

impl fmt::Display for CountryHandlerSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountryCountOutOfRange { country_count } => write!(
                formatter,
                "CCountryHandler содержит {country_count} стран вне signed 32-битного диапазона"
            ),
            Self::NullCountry { map_key } => {
                write!(formatter, "CCountryHandler содержит null country по ключу {map_key}")
            }
            Self::Country { map_key, source } => {
                write!(formatter, "страна по ключу {map_key} не сериализована: {source}")
            }
        }
    }
}

impl Error for CountryHandlerSerializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Country { source, .. } => Some(source),
            _ => None,
        }
    }
}
