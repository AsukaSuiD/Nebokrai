//! CMachineShield, gameserver.exe + GameServer.pdb, appserver/skills/machineshield.cpp.
//! Check/AI и lifecycle общие с ManaShield в selfshield/stateskill;
//! payload содержит срок, прочность и два WORD-фактора без добавочных защит.

use super::machineshieldstate::MachineShieldState;
use super::selfshield::SelfShieldOwner;
use super::shieldstate::DefenseShieldState;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;

pub(crate) const MACHINE_SHIELD_SKILL_ID: u32 = 222;

pub(crate) struct MachineShieldOwner;

impl SelfShieldOwner for MachineShieldOwner {
    const SKILL_ID: u32 = MACHINE_SHIELD_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::MachineShield;

    fn create_state(properties: &CSkillBaseProperties) -> DefenseShieldState {
        let mp_factor = properties.query_property(20_025) as u16;
        let hp_factor = properties.query_property(20_024) as u16;
        let life = properties.query_property(10_010) as i32;
        let keep = properties.query_property(10_002);
        DefenseShieldState::Machine(MachineShieldState::new(0, keep, life, hp_factor, mp_factor))
    }
}
