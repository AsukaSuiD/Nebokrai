//! Блоки country-war надстройки мирового региона `CWorldCountryWarRegion`,
//! вынесенные заранее: сама надстройка остаётся в старом
//! `appworld/worldcountrywarregion.rs` до шага переноса области.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.

use crate::regions::worldregion::{WorldRegionLoadError, WorldRegionSerializationBlock};

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
