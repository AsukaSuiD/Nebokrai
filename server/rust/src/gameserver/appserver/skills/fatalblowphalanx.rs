//! Тонкий путь к снаряду смертельного удара `CFatalBlowPhalanx` в Zone.
//!
//! Источник: `gameserver.exe` `4F5C98E0…` + `GameServer.pdb` (RSDS match),
//! исходный владелец `appserver/skills/fatalblowphalanx.cpp`. Форма, тики,
//! клиентский снимок и формула перенесены буквально в
//! `nebokrai_zone::skills::fatalblowphalanx` порцией №6b (RAW-метаданные
//! недостигнутого `DecordFromByteArray` — там же). Здесь реэкспорт прежних
//! имён; потребители (summonshape, serverregion, game/tick) не меняются.

pub(crate) use nebokrai_zone::skills::{
    CFatalBlowPhalanx, FatalBlowPhalanxTick, calculate_owned_fatal_blow_attack,
};
