//! Тонкий путь к снаряду базовой атаки боевой феи `CBFBaseAttackPhalanx` в
//! Zone.
//!
//! Источник: `gameserver.exe` `4F5C98E0…` + `GameServer.pdb` (RSDS match),
//! исходный владелец `appserver/skills/battlefairybasemagicphalanx.cpp`.
//! Форма, тики, клиентский снимок и формула перенесены буквально в
//! `nebokrai_zone::skills::battlefairybasemagicphalanx` порцией №6b (exact
//! точки x87/FISTP `0x005E2698..0x005E26DC` и `0x005E27BC..0x005E27E6` — в
//! шапке Zone-файла). Здесь реэкспорт прежних имён; потребители (summonshape,
//! serverregion, game/tick) не меняются.

pub(crate) use nebokrai_zone::skills::{
    BattleFairyPhalanxTick, CBattleFairyBaseMagicPhalanx,
    calculate_owned_battle_fairy_base_magic_attack,
};
