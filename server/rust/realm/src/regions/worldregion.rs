//! Блоки загрузки и сериализации мирового региона `CWorldRegion::Load`/
//! `Serialize`, вынесенные сюда заранее: сами ветки разбора и `CWorldRegion`
//! остаются в старом `appworld/worldregion.rs` до шага переноса области.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.

use crate::regions::region::RegionSerializationBlock;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldRegionTextLoadError {
    MissingValue {
        field: &'static str,
    },
    InvalidValue {
        field: &'static str,
    },
    MonsterNameRequiresLegacyHeapLayout {
        length: usize,
    },
    TooManyEntries {
        collection: &'static str,
        count: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WorldRegionSetupSerializationBlock {
    pub field: &'static str,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldRegionLoadError {
    Text {
        owner: &'static str,
        source: WorldRegionTextLoadError,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorldRegionSerializationBlock {
    Region(RegionSerializationBlock),
    Setup(WorldRegionSetupSerializationBlock),
    TooManyEntries {
        collection: &'static str,
        count: usize,
    },
    MonsterVariant {
        source: WorldRegionTextLoadError,
    },
}
