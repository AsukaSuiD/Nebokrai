//! Подготовка содержимого Realm; исполнение сценариев принадлежит Zone.

mod clientresource;
pub mod goods;
mod quests;
mod scriptfiles;
mod scripts;
pub mod skill;
mod timetoreturn;

pub use clientresource::{
    DefaultClientResourceOwner, DefaultClientResourceReplacement, LOAD_SERVER_RESOURCE_SUCCESS_LOG,
};
pub use quests::{QuestCatalog, QUEST_EX_PATH, QUEST_PATH};
pub use timetoreturn::{
    TimeToReturn, TimeToReturnCallbacks, TimeToReturnContext, TimeToReturnFireDisposition,
    TimeToReturnFireReport, TimeToReturnLoadError, TimeToReturnLoadReport, TimeToReturnParam,
};

pub use scriptfiles::{find_script_files, ScriptFileScan, ScriptFileScanError};
pub use scripts::{
    normalize_script_path, ScriptListSource, ScriptLoadContext, ScriptLoadReport,
    ScriptReleaseState, ScriptRequiredFile, ScriptResources,
};
