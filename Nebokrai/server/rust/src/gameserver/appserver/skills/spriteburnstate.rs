//! Периодическое состояние SpriteBurn (0x1A6).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/spriteburnstate.cpp.
//! Несмотря на название, урон имеет тип Poison и передаёт signed HP без clamp.
//! Срок проверяется после возможного удара: просроченное состояние ещё может
//! ударить, но отсутствие или смерть sufferer завершает его до чтения часов.
//! Codec56 и lifecycle используют общий owner яда и единую арену состояний.
//! Ниже сохранены две пока недостигнутые перегрузки Begin.

use super::spriteburn::SPRITE_BURN_SKILL_ID;
pub(crate) use crate::gameserver::appserver::states::poison::POISON_STATE_BYTES
    as SPRITE_BURN_STATE_BYTES;
pub(crate) type SpriteBurnState =
    crate::gameserver::appserver::states::poison::PoisonState<SPRITE_BURN_SKILL_ID>;

// ============================================================================
// FUNCTION: CSpriteBurnState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// RVA: 0x00206350
// ADDRESS: 00606350
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpriteBurnState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// RVA: 0x002063F0
// ADDRESS: 006063f0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
