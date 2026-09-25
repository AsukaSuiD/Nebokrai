//! Полученные определения и ресурсы Zone; живые игровые экземпляры здесь не хранятся.

pub mod countryparam; // CCountryParam: GameServer-владелец параметров стран.
mod functions; // реестр имён сценарных команд (LoadFunction).
pub mod goods; // базовые свойства товара и их startup wire-decoder.
pub mod honorranks; // CHonorRanks: startup snapshot рангов чести.
mod quests; // полученный каталог заданий.
mod scripts; // function/variable/script ресурсы сценариев.
mod skills; // полученный реестр runtime-свойств навыков.

pub use functions::{FunctionRegistryLoadReport, ScriptFunctionRegistry}; // реестр функций и отчёт загрузки.
pub use quests::QuestCatalog; // каталог заданий.
pub use scripts::{ScriptResourcePublication, ScriptResources, ScriptResourcesReleased}; // хранилище ресурсов и его публикация.
pub use skills::{
    is_need_float, is_war_soul_skill, skill_failed_message_color, CSkillBaseProperties,
    SkillPropertiesCatalog, SkillPropertiesDecodeError, UNKNOWN_SKILL_TYPE,
}; // свойства навыков и кодек их каталога.
