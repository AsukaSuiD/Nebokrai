//! Второе усиленное периодическое лечение `CSuperHeal2` (`0xE4`).
//!
//! Исходный `AI` хранит состояние у выбранной цели, но вызывает `Begin` с
//! заклинателем как источником и целью. Общий владелец `heal` сохраняет это
//! расхождение явно. Координатная перегрузка пока не подключена.

pub(crate) const SUPER_HEAL_2_SKILL_ID: u32 = 0xe4;

// ============================================================================
// FUNCTION: CSuperHeal2::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\superheal2.cpp:170
// RVA: 0x00151DD0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
