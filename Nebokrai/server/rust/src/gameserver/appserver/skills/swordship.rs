//! Постоянные прибавки четырёх навыков Swordship.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/swordship{,2,3,4}.cpp.
//! Общий зарегистрированный AI читает свойства и только U, без подстановки S.
//! MIN читается перед MAX; исходный ctor принимает MAX/MIN, а Rust хранит их
//! в именованных полях. Новый Begin(U,U) предшествует поиску первого того же
//! ID без фильтра RTTI/ended. Замена через общий установщик вызывает старый
//! End и destructor остатка, сохраняет позицию, затем пересчитывает свойства.
//! Четыре ID сосуществуют. Нет отдельного visual, OnChangeStates, MP, delay
//! или reuse; успешный и отказной AI заканчиваются End(0). Player и monster
//! используют один зарегистрированный экземпляр и общий lifecycle.

use super::skillbaseproperties::CSkillBaseProperties;
use super::swordshipstate::SwordshipState;

pub(crate) const SWORDSHIP_SKILL_ID: u32 = 0x6f;
pub(crate) const SWORDSHIP_2_SKILL_ID: u32 = 0xe0;
pub(crate) const SWORDSHIP_3_SKILL_ID: u32 = 0xe8;
pub(crate) const SWORDSHIP_4_SKILL_ID: u32 = 0xe9;

const SKILL_USAGE_TARGET_MIN_ATK_GAIN: u32 = 0x74;
const SKILL_USAGE_TARGET_MAX_ATK_GAIN: u32 = 0x75;

pub(crate) const fn is_swordship_skill(skill_id: u32) -> bool {
    matches!(
        skill_id,
        SWORDSHIP_SKILL_ID | SWORDSHIP_2_SKILL_ID | SWORDSHIP_3_SKILL_ID | SWORDSHIP_4_SKILL_ID
    )
}

pub(super) fn state_from_properties(
    skill_id: u32,
    properties: &CSkillBaseProperties,
) -> SwordshipState {
    let minimum = properties.query_property(SKILL_USAGE_TARGET_MIN_ATK_GAIN) as i32;
    let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_ATK_GAIN) as i32;
    SwordshipState::new(skill_id, minimum, maximum)
}
