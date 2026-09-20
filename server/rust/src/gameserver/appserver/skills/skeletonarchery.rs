//! Владелец `CSkeletonArchery` (`0x1A1`).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный
//! `appserver/skills/skeletonarchery.cpp`. Begin, Check, AI, visual и удар
//! объединены с ChuckStone в `directprojectile`. Этот владелец перед выпуском
//! строит обычный GetTargetPath и не отмечает prepared. `m_bAutoRestart`
//! конструируется ложным, не меняется, а его унаследованный Restart пуст.

pub(crate) const SKELETON_ARCHERY_SKILL_ID: u32 = 0x1a1;
