//! Подготовка содержимого Realm; исполнение сценариев принадлежит Zone.

mod clientresource;
mod quests;
mod scriptfiles;
mod scripts;

pub use clientresource::{
    DefaultClientResourceOwner, DefaultClientResourceReplacement, LOAD_SERVER_RESOURCE_SUCCESS_LOG,
};
pub use quests::{QuestCatalog, QUEST_EX_PATH, QUEST_PATH};

pub use scriptfiles::{find_script_files, ScriptFileScan, ScriptFileScanError};
pub use scripts::{
    normalize_script_path, ScriptListSource, ScriptLoadContext, ScriptLoadReport,
    ScriptReleaseState, ScriptRequiredFile, ScriptResources,
};
