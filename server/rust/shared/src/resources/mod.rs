//! Общие форматы ресурсов; выбор корня и публикация остаются у владельца роли.

mod catalog;
mod emotion;
mod filesinfo;
mod hitlevel;
mod globesetup;
mod marker;
mod monsterlist;
mod regionrouter;
mod package;
mod path;
mod playerlist;
mod quest;
mod quest_text;
mod quest_wire;
mod rfile;
mod source;
mod stringtable;
mod stringtable_wire;

pub use catalog::{ResourceCatalog, ResourceLoadError, ResourceLoadReport, ResourcePackageLoad};
pub use emotion::{
    CEmotion, EmotionDecodeError, EmotionFormatError, EmotionSerializeError,
};
pub use filesinfo::{FileInfo, FilesInfo, FilesInfoParseError, PackFileInfo};
pub use hitlevel::{
    CHitLevelSetup, HitLevelDecodeError, HitLevelEntry, HitLevelFormatError,
    HitLevelSerializeError,
};
pub use monsterlist::{
    MonsterDrop, MonsterDropList, MonsterDropRegistry, MonsterListDecodeError,
    MonsterListLoadError, MonsterListSerializeError, MonsterProperties, MonsterRegistry,
    MonsterSkill, decode_monster_list, get_monster_property_by_origin_index,
    get_monster_property_by_origin_name, get_monster_property_by_origin_name_mut,
    get_monster_property_by_picture_id, load_drop_goods_list, load_monster_list,
    serialize_monster_list,
};
pub use globesetup::{
    GLOBE_SETUP_BLOB_LENGTH, GlobePlayerPropertyCoefficients, GlobeRpGainPolicy,
    GlobeSetupDecodeError, GlobeSetupLoadError, GlobeSetupLoadReport, GlobeSetupSnapshot,
    GlobeStiffenSetup,
};
pub use marker::read_to_marker;
pub use package::{PackageArchive, PackageFileIndex, PackageReadError};
pub use path::{normalize_resource_path, resolve_resource_path};
pub use playerlist::{
    CPlayerList, PlayerBaseProperties, PlayerBasePropertiesMap, PlayerCreationPropertiesLookup,
    PlayerListDecodeError, PlayerListFormatError, PlayerListLoadReport, PlayerListSerializeError,
    PlayerOriginEquipment, PlayerPropertiesUpgrade, PlayerPropertiesUpgradeLoadReport,
    PlayerPropertiesUpgradeMap,
};
pub use regionrouter::{
    RegionNextNode, RegionRoutePoint, RegionRouteStep, RegionRouter,
    RegionRouterChangeOutcome, RegionRouterDecodeError, RegionRouterDecodeField,
    RegionRouterLoadError, RegionRouterLoadReport, RegionRouterNode,
    RegionRouterSerializeError,
};
pub use quest::{CQuestSystem, QuestEntry};
pub use quest_text::{
    QuestSystemLoadCompletion, QuestSystemLoadReport, QuestTextError, QuestTextErrorKind,
};
pub use quest_wire::{
    QuestStringField, QuestSystemDecodeError, QuestSystemDecodeOutcome, QuestSystemSerializationBlock,
    QuestWireField,
};
pub use rfile::CRFile;
pub use source::{ResourceOpenError, ResourceSource, open_resource};
pub use stringtable::{StringTable, StringTableParseError, StringTableParseErrorKind};
pub use stringtable_wire::{MyStringTable, MyStringTableDecodeError, MyStringTableDecodeOutcome};
