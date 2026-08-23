//! Ordered owner `CCountryHandler` GameServer, подтверждённый точными
//! `gameserver.exe + GameServer.pdb`; исходники
//! `server/gameserver/appserver/country/countryhandler.cpp/.h`.
//!
//! Decoder сначала освобождает все прежние `CCountry`, очищает map, читает
//! signed count и публикует каждый полностью декодированный country по его
//! unsigned ID. Duplicate ID заменяет прежний pointer через `operator[]`;
//! Rust освобождает заменённый owner вместо исторической внутренней утечки.
//! Malformed record сохраняет уже опубликованный префикс. Process singleton и
//! manual deleting destructors заменены owned-полем `CGame` и обычным `Drop`.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use super::country::{CCountry, CountryDecodeError};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CCountryHandler {
    countries: BTreeMap<u8, CCountry>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CountryHandlerDecodeReport {
    pub(crate) declared: i32,
    pub(crate) decoded: usize,
    pub(crate) retained: usize,
    pub(crate) replaced_duplicates: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CountryHandlerDecodeError {
    Count {
        offset: usize,
        available: usize,
    },
    Country {
        record_index: usize,
        source: CountryDecodeError,
    },
}

impl fmt::Display for CountryHandlerDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Count { offset, available } => write!(
                formatter,
                "CountryHandler snapshot обрывается на count в {offset}: нужно 4, доступно {available}"
            ),
            Self::Country {
                record_index,
                source,
            } => write!(
                formatter,
                "CountryHandler country record {record_index} не декодирован: {source}"
            ),
        }
    }
}

impl Error for CountryHandlerDecodeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Country { source, .. } => Some(source),
            Self::Count { .. } => None,
        }
    }
}

impl CCountryHandler {
    pub(crate) fn decord_from_byte_array(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<CountryHandlerDecodeReport, CountryHandlerDecodeError> {
        self.countries.clear();
        let offset = *cursor;
        let available = source.len().saturating_sub(offset);
        let Some(bytes) = source.get(offset..offset.saturating_add(4)) else {
            return Err(CountryHandlerDecodeError::Count { offset, available });
        };
        *cursor += 4;
        let declared =
            i32::from_le_bytes(bytes.try_into().expect("CountryHandler count уже проверен"));

        let mut decoded = 0;
        let mut replaced_duplicates = 0;
        for record_index in 0..declared.max(0) as usize {
            let mut country = CCountry::default();
            country
                .decord_from_byte_array(source, cursor)
                .map_err(|source| CountryHandlerDecodeError::Country {
                    record_index,
                    source,
                })?;
            if self
                .countries
                .insert(country.country_id(), country)
                .is_some()
            {
                replaced_duplicates += 1;
            }
            decoded += 1;
        }

        Ok(CountryHandlerDecodeReport {
            declared,
            decoded,
            retained: self.countries.len(),
            replaced_duplicates,
        })
    }

    pub(crate) fn country(&self, country_id: u8) -> Option<&CCountry> {
        self.countries.get(&country_id)
    }

    pub(crate) fn country_mut(&mut self, country_id: u8) -> Option<&mut CCountry> {
        self.countries.get_mut(&country_id)
    }

    pub(crate) fn countries(&self) -> &BTreeMap<u8, CCountry> {
        &self.countries
    }
}
