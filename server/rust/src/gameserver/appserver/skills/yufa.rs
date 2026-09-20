//! Усиление уклонения от стихий `CYufa` (`0x219`).
use super::battlefairyattribute::BattleFairyAttributeSkill;
use super::battlefairyattributestate::BattleFairyAttributeKind;
pub(crate) const SKILL_ID: u32 = 0x219;
pub(crate) const DEFINITION: BattleFairyAttributeSkill = BattleFairyAttributeSkill { value_usage: 0x81, kind: BattleFairyAttributeKind::ElementAvoidGain };
