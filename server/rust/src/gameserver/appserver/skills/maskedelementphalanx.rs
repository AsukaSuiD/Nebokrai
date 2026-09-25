//! Путь к живой форме неподвижных масочных областей огненной стены и инь-ян
//! в Zone.
//! Источник: gameserver.exe/GameServer.pdb,
//! appserver/skills/firewallphalanx.cpp и yinyangphalanx{,2}.cpp. Композит
//! CShape + снимок атаки + маска и клиентский снимок перенесены буквально в
//! `nebokrai_zone::skills::masked_area` (основание и статусы см. там).

pub(crate) use nebokrai_zone::skills::{MaskedAreaPulse, MaskedElementPhalanx};
