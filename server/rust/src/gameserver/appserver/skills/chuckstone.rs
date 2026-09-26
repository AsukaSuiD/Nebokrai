//! Владелец `CChuckStone` (`0x19D`).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный
//! `appserver/skills/chuckstone.cpp`. Begin, Check, AI, visual и удар общие
//! со SkeletonArchery — перенесены буквально в Zone `skills/directprojectile.rs`
//! (кластер B полосы Monster 0x19x); делегация исполнения — в соседнем
//! `directprojectile.rs`. Отличие ChuckStone — повторный GetTargetPath с
//! принудительной длиной MAX и `prepared` после visual1. OnChangeRegion
//! завершает только неоконченный экземпляр через End(0), что обслуживает
//! общий зарегистрированный lifecycle. ID владельца живёт в Zone.

pub(crate) use nebokrai_zone::skills::directprojectile::CHUCK_STONE_SKILL_ID;
