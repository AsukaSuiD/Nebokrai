//! Базовая защита GameServer (`SKILL_BASE_DEFENSE`) для обычной атаки.
//! Источник: gameserver.exe + GameServer.pdb,
//! appserver/skills/fightdefense.cpp/.h.
//!
//! Формулы защиты (hit-качество и полный промах, физическая/стихийная/душевная
//! защита, blast/critical-ветви, уклонение, коэффициенты PvP и Pillar) перенесены
//! буквально в `nebokrai_zone::combat::fightdefense`; машинное основание и
//! статусы см. там. Снимок `PlayerCombatProperties` переехал туда же и
//! переиздаётся из `appserver/player.rs`. Здесь остаются re-export прежних имён
//! для существующих потребителей (`truncate_original` — переходный путь к
//! модели усечений Zone `combat/rounding`; живые Begin/restart/AI/End щитов —
//! соседний `shieldstate.rs`).

pub(crate) use nebokrai_zone::combat::{
    defend_build_base_attack, defend_build_from_monster_base_attack, defend_monster_base_attack,
    defend_monster_from_monster_base_attack, defend_player_base_attack,
    defend_player_from_monster_base_attack, truncate_original,
};
