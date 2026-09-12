//! CManaShield, gameserver.exe + GameServer.pdb, appserver/skills/manashield.cpp.
//! Check/AI и lifecycle общие с MachineShield в selfshield/stateskill;
//! этот owner добавляет физическую и стихийную защиту к payload мана-щита.

use super::manashieldstate::ManaShieldState;
use super::selfshield::SelfShieldOwner;
use super::shieldstate::DefenseShieldState;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;

pub(crate) const MANA_SHIELD_SKILL_ID: u32 = 321;

pub(crate) struct ManaShieldOwner;

impl SelfShieldOwner for ManaShieldOwner {
    const SKILL_ID: u32 = MANA_SHIELD_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::ManaShield;

    fn create_state(properties: &CSkillBaseProperties) -> DefenseShieldState {
        let mp_factor = properties.query_property(20_025) as u16;
        let hp_factor = properties.query_property(20_024) as u16;
        let element_defense = properties.query_property(10_012) as i32;
        let physical_defense = properties.query_property(10_011) as i32;
        let life = properties.query_property(10_010) as i32;
        let keep = properties.query_property(10_002);
        DefenseShieldState::Mana(ManaShieldState::new(
            0, keep, life, physical_defense, element_defense, hp_factor, mp_factor,
        ))
    }
}
