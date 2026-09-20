//! Владелец `CChuckStone` (`0x19D`).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный
//! `appserver/skills/chuckstone.cpp`. Begin, Check, AI, visual и удар общие
//! со SkeletonArchery в `directprojectile`; отличие ChuckStone — повторный
//! GetTargetPath с принудительной длиной MAX и `prepared` после visual1.
//! OnChangeRegion завершает только неоконченный экземпляр через End(0), что
//! обслуживает общий зарегистрированный lifecycle.

pub(crate) const CHUCK_STONE_SKILL_ID: u32 = 0x19d;
