//! Ослабление уклонения `CPojia` (`0x212`).
//!
//! Полная достигнутая вертикаль применения, AI и состояния использует общего
//! владельца семейства, а этот модуль сохраняет конкретный идентификатор и код
//! свойства формулы.
use super::battlefairyattribute::BattleFairyAttributeSkill;
use super::battlefairyattributestate::BattleFairyAttributeKind;
pub(crate) const SKILL_ID: u32 = 0x212;
pub(crate) const DEFINITION: BattleFairyAttributeSkill = BattleFairyAttributeSkill { value_usage: 0xe4, kind: BattleFairyAttributeKind::AttackAvoidLoss };
