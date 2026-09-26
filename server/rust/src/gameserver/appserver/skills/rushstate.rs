//! Оглушение первым рывком: 0x73, общий timed payload и lifecycle Blind.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/rushstate.cpp.
//! ID 0x73, 8-байтная запись и тип состояния перенесены в Zone
//! `skills/rush.rs` (порция №5 «player melee»); OnAction здесь пустой,
//! поэтому Defense не снимает состояние, в отличие от собственно Blind.
//! Начало состояния остаётся общим lifecycle Blind семейства
//! (`blindstate.rs`); его alias и безпотребительные реэкспорты ID/BYTES
//! со старого пути сняты порцией №5 (живые типы адресуются из Zone напрямую,
//! `RUSH_2_*` — в `rushstate2.rs` для blind/strike).

pub(crate) use nebokrai_zone::skills::rush::RushState;
