//! Общие форматы ресурсов; выбор корня и публикация остаются у владельца роли.

mod catalog;
mod filesinfo;
mod package;
mod path;
mod quest;
mod quest_text;
mod quest_wire;
mod rfile;
mod source;
mod stringtable;
mod stringtable_wire;

pub use catalog::{ResourceCatalog, ResourceLoadError, ResourceLoadReport, ResourcePackageLoad};
pub use filesinfo::{FileInfo, FilesInfo, FilesInfoParseError, PackFileInfo};
pub use package::{PackageArchive, PackageFileIndex, PackageReadError};
pub use path::{normalize_resource_path, resolve_resource_path};
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
