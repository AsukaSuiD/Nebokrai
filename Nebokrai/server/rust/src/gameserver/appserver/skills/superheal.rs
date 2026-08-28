//! Усиленное периодическое лечение `CSuperHeal` (`0xD9`).
//!
//! Общий достигнутый конвейер принадлежит модулю `heal`; этот владелец
//! фиксирует отличающийся идентификатор и замену обычного состояния `0xD3`.
//! Координатная перегрузка выбора цели пока не подключена и сохранена ниже.

pub(crate) const SUPER_HEAL_SKILL_ID: u32 = 0xd9;

// ============================================================================
// FUNCTION: CSuperHeal::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\superheal.cpp:170
// RVA: 0x00173C00
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
