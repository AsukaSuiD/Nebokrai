//! Блоки city-надстройки мирового региона `CWorldCityRegion`, вынесенные
//! заранее: сама надстройка остаётся в старом `appworld/worldcityregion.rs`
//! до шага переноса области.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.

use crate::regions::worldwarregion::{WorldWarRegionLoadError, WorldWarRegionSerializationBlock};
use crate::regions::worldregion::WorldRegionSetupSerializationBlock;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldCityRegionTextLoadError {
    MissingValue { field: &'static str },
    InvalidValue { field: &'static str },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldCityRegionLoadError {
    War(WorldWarRegionLoadError),
    City(WorldCityRegionTextLoadError),
    BaseSetup(WorldRegionSetupSerializationBlock),
    UninitializedDefenceField { field: &'static str },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldCityRegionSerializationBlock {
    War(WorldWarRegionSerializationBlock),
    UninitializedDefenceField { field: &'static str },
    TooManyGates { count: usize },
}
