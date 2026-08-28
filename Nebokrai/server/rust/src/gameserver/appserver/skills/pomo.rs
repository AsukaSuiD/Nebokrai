//! Ослабление стихийной атаки `CPomo` (`0x214`).
use super::battlefairyattribute::BattleFairyAttributeSkill;
use super::battlefairyattributestate::BattleFairyAttributeKind;
pub(crate) const SKILL_ID: u32 = 0x214;
pub(crate) const DEFINITION: BattleFairyAttributeSkill = BattleFairyAttributeSkill { value_usage: 0xd7, kind: BattleFairyAttributeKind::ElementModifyLoss };
