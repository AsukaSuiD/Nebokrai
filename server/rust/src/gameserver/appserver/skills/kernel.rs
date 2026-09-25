//! Адаптеры конкретных исполнений игрока и боевого духа перенесены в Zone
//! `skills/execution` (порция 5 волны moveshape): typed payload, каталоги
//! `PlayerSkillExecution`/`BattleFairyExecution` и полная запись реестра.
//! База, стадии и единственный kernel принадлежат `zone/skills/lifecycle.rs`.
//! Здесь их реэкспорт для старого пакета.

pub(crate) use nebokrai_zone::skills::execution::*;
pub(crate) use nebokrai_zone::skills::{
    SkillExecutionKernel, SkillLifecycle, SkillStage, SkillTermination,
    battle_fairy_mana_text_cost, skill_is_restored,
};
