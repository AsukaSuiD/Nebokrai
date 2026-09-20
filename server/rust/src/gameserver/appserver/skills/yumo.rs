//! Усиление стихийной атаки `CYumo` (`0x218`).
use super::battlefairyattribute::BattleFairyAttributeSkill;
use super::battlefairyattributestate::BattleFairyAttributeKind;
pub(crate) const SKILL_ID: u32 = 0x218;
pub(crate) const DEFINITION: BattleFairyAttributeSkill = BattleFairyAttributeSkill { value_usage: 0x73, kind: BattleFairyAttributeKind::ElementModifyGain };
