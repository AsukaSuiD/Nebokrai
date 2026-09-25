//! Блоки war-надстройки мирового региона `CWorldWarRegion`, вынесенные
//! заранее: сама надстройка остаётся в старом `appworld/worldwarregion.rs`
//! до шага переноса области.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.

use crate::regions::worldregion::{WorldRegionLoadError, WorldRegionSerializationBlock};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldWarRegionLoadError {
    Base(WorldRegionLoadError),
    MissingValue { field: &'static str },
    InvalidValue { field: &'static str },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldWarRegionSerializationBlock {
    Base(WorldRegionSerializationBlock),
    UninitializedField { field: &'static str },
}
