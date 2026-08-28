//! Ослабление уклонения от стихий `CPofa` (`0x215`).
use super::battlefairyattribute::BattleFairyAttributeSkill;
use super::battlefairyattributestate::BattleFairyAttributeKind;
pub(crate) const SKILL_ID: u32 = 0x215;
pub(crate) const DEFINITION: BattleFairyAttributeSkill = BattleFairyAttributeSkill { value_usage: 0xe5, kind: BattleFairyAttributeKind::ElementAvoidLoss };
