//! Тонкий путь к живой области CLeimingPhalanx2 в Zone.
//!
//! Источник: gameserver.exe `4F5C98E0…` + GameServer.pdb (RSDS match),
//! appserver/skills/thunder2phalanx.cpp. Живое тело области (форма, тики,
//! wire и calc) перенесено буквально в
//! `nebokrai_zone::skills::thunder2phalanx` порцией T2 «BF-облака области»
//! (истинные RVA-якоря и статусы MATCH/PARTIAL — в шапке Zone-файла).
//! Здесь — реэкспорт прежних имён; потребители (game run-flow, serverregion,
//! summonshape, thunder2) не меняются.

pub(crate) use nebokrai_zone::skills::{
    CLeimingPhalanx2, Leiming2PhalanxTick, calculate_owned_leiming2_attack,
};
