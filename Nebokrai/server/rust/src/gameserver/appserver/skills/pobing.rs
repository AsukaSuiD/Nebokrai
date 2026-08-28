//! Ослабление физической атаки `CPobing` (`0x213`).
use super::battlefairyattribute::BattleFairyAttributeSkill;
use super::battlefairyattributestate::BattleFairyAttributeKind;
pub(crate) const SKILL_ID: u32 = 0x213;
pub(crate) const DEFINITION: BattleFairyAttributeSkill = BattleFairyAttributeSkill { value_usage: 0xcd, kind: BattleFairyAttributeKind::AttackLoss };
