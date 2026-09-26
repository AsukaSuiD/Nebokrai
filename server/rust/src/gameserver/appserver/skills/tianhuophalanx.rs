//! Тонкий путь к живой области CTianhuoPhalanx в Zone.
//!
//! Источник: gameserver.exe `4F5C98E0…` + GameServer.pdb (RSDS match),
//! appserver/skills/tianhuophalanx.cpp. Живое тело области (форма, тики,
//! wire и calc) перенесено буквально в
//! `nebokrai_zone::skills::tianhuophalanx` порцией T2 «BF-облака области»
//! (истинные RVA-якоря и статусы MATCH/PARTIAL — в шапке Zone-файла).
//! Здесь — реэкспорт прежних имён; потребители (game run-flow, serverregion,
//! summonshape, tianhuo) не меняются.

pub(crate) use nebokrai_zone::skills::{
    CTianhuoPhalanx, TianhuoPhalanxTick, calculate_owned_tianhuo_attack,
};
