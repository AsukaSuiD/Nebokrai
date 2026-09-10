//! Безопасная Rust-проекция `CStateFactory::Unserialize` GameServer.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/statefactory.cpp`, RVA `0x001D7D00`. Native factory читает
//! ID, создаёт concrete state и поручает ему потребить запись. Rust хранит
//! concrete состояния у `CMoveShape`; этот owner сохраняет switch ID→layout и
//! последовательное продвижение по GameSave wire. Неизвестный ID, переменная
//! запись без терминатора и усечённый payload останавливают типизацию до
//! спорной записи: исходный хвост остаётся в `LegacyStateCodec`.
//! Для Cure, защитных щитов и расходуемого восстановления та же ветвь factory
//! задаёт concrete decoder: один проход CMoveShape переносит эти записи в общую
//! арену в wire-порядке. Остальные семейства пока материализуются старым путём
//! у CMoveShape; отсутствие decoder здесь не означает пустой игровой End.
//!
//! CRT allocation, RTTI, vtable и exception plumbing не воспроизводятся.

use crate::gameserver::appserver::chbystate::CHANGE_BODY_STATE_ID;
use crate::gameserver::appserver::exstate::{EX_STATE_ID, EX_STATE_NEW_ID};
use crate::gameserver::appserver::moveshape::StateData;
use crate::gameserver::appserver::restorestate::ConsumableRestoreState;
use crate::gameserver::appserver::particularstate::{PARTICULAR_STATE_BYTES, PARTICULAR_STATE_ID};
use crate::gameserver::appserver::ridestate::RIDE_STATE_ID;
use crate::gameserver::appserver::restorehpstate::{RESTORE_HP_STATE_BYTES, RESTORE_HP_STATE_ID};
use crate::gameserver::appserver::restorempstate::{RESTORE_MP_STATE_BYTES, RESTORE_MP_STATE_ID};
use crate::gameserver::appserver::scriptstate::ScriptMoveState;
use crate::gameserver::appserver::states::automaticrestore::{
    AUTOMATIC_RESTORE_STATE_BYTES, is_automatic_restore_state_id,
};
use crate::gameserver::appserver::teamstate::{CTeamState, TEAM_STATE_ID};

use super::agilitystate::PERSISTENT_AGILITY_FAMILY_STATE_BYTES;
use super::agilitystate2::AGILITY_STATE_2_BYTES;
use super::battlefairyattributestate::BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES;
use super::blindstate::{BLIND_STATE_BYTES, BLIND_STATE_ID};
use super::bloodlossstate::BLOOD_LOSS_STATE_BYTES;
use super::boalockstate::{BOA_LOCK_STATE_BYTES, BOA_LOCK_STATE_ID};
use super::bossbluefurystate::{BOSS_BLUE_FURY_STATE_BYTES, BOSS_BLUE_FURY_STATE_ID};
use super::bossbluequakestate::{BOSS_BLUE_QUAKE_STATE_BYTES, BOSS_BLUE_QUAKE_STATE_ID};
use super::callosity::{CALLOSITY_2_SKILL_ID, CALLOSITY_SKILL_ID};
use super::callositystate::CALLOSITY_STATE_BYTES;
use super::curestate::{CURE_STATE_BYTES, CURE_STATE_SKILL_ID, CureState};
use super::daubpoisonstate::{DAUB_POISON_STATE_BYTES, DAUB_POISON_STATE_ID};
use super::enlargefullmiss::ENLARGE_FULL_MISS_SKILL_ID;
use super::enlargefullmissstate::ENLARGE_FULL_MISS_STATE_BYTES;
use super::enlargemaxhp::ENLARGE_MAX_HP_SKILL_ID;
use super::enlargemaxhpstate::ENLARGE_MAX_HP_STATE_BYTES;
use super::enlargemaxmp::ENLARGE_MAX_MP_SKILL_ID;
use super::enlargemaxmpstate::ENLARGE_MAX_MP_STATE_BYTES;
use super::energyholdingstate::{ENERGY_HOLDING_STATE_BYTES, ENERGY_HOLDING_STATE_ID};
use super::furystate::{FURY_STATE_BYTES, FURY_STATE_SKILL_ID};
use super::godblessstate::{GOD_BLESS_STATE_BYTES, GOD_BLESS_STATE_ID};
use super::godblessstate2::GOD_BLESS_STATE_2_ID;
use super::hearten::HEARTEN_SKILL_ID;
use super::heartenstate::HEARTEN_STATE_BYTES;
use super::heal::HEAL_SKILL_ID;
use super::heal2::HEAL_2_SKILL_ID;
use super::healstate::HEAL_STATE_BYTES;
use super::kerosenestate::{KEROSENE_STATE_BYTES, KEROSENE_STATE_ID};
use super::knightcutstate::{KNIGHT_CUT_STATE_BYTES, KNIGHT_CUT_STATE_ID};
use super::knockoutstate::{KNOCK_OUT_STATE_BYTES, KNOCK_OUT_STATE_ID};
use super::leafcutstate::{LEAF_CUT_STATE_BYTES, LEAF_CUT_STATE_ID};
use super::leafcutstate2::{LEAF_CUT_2_STATE_BYTES, LEAF_CUT_2_STATE_ID};
use super::leafcutstate3::{LEAF_CUT_3_STATE_BYTES, LEAF_CUT_3_STATE_ID};
use super::lifeshield::LIFE_SHIELD_SKILL_ID;
use super::lifeshieldstate::{LIFE_SHIELD_STATE_BYTES, LifeShieldState};
use super::machineshield::MACHINE_SHIELD_SKILL_ID;
use super::machineshieldstate::{MACHINE_SHIELD_STATE_BYTES, MachineShieldState};
use super::manashield::MANA_SHIELD_SKILL_ID;
use super::manashieldstate::{MANA_SHIELD_STATE_BYTES, ManaShieldState};
use super::meteorarrowstate::{METEOR_ARROW_MASS_SKILL_ID, METEOR_ARROW_STATE_BYTES};
use super::origin::ORIGIN_SKILL_ID;
use super::originstate::ORIGIN_STATE_BYTES;
use super::pillarstate::{PILLAR_STATE_BYTES, PILLAR_STATE_ID};
use super::poisonarrow::POISON_ARROW_SKILL_ID;
use super::poisonarrowstate::POISON_ARROW_STATE_BYTES;
use super::poisonfogstate::{POISON_FOG_STATE_BYTES, POISON_FOG_STATE_ID};
use super::promotion::PROMOTION_SKILL_ID;
use super::promotionstate::{PROMOTION_STATE_BYTES, PromotionState};
use super::shieldstate::DefenseShieldState;
use super::ragebreakstate::{RAGE_BREAK_STATE_BYTES, RAGE_BREAK_STATE_ID};
use super::roarstate::{ROAR_STATE_BYTES, ROAR_STATE_ID};
use super::rushstate::{RUSH_STATE_BYTES, RUSH_STATE_ID};
use super::rushstate2::{RUSH_2_STATE_BYTES, RUSH_2_STATE_ID};
use super::sealstate::{SEAL_STATE_BYTES, SEAL_STATE_ID};
use super::soulcollectstate::{SOUL_COLLECT_STATE_BYTES, SOUL_COLLECT_STATE_ID};
use super::spiderpoison::SPIDER_POISON_SKILL_ID;
use super::spiderpoisonstate::SPIDER_POISON_STATE_BYTES;
use super::spiderweb::SPIDER_WEB_SKILL_ID;
use super::spiderwebstate::SPIDER_WEB_STATE_BYTES;
use super::spriteburn::SPRITE_BURN_SKILL_ID;
use super::spriteburnstate::SPRITE_BURN_STATE_BYTES;
use super::strikestate::{STRIKE_STATE_BYTES, STRIKE_STATE_ID};
use super::superheal::SUPER_HEAL_SKILL_ID;
use super::superheal2::SUPER_HEAL_2_SKILL_ID;
use super::swordship::{
    SWORDSHIP_2_SKILL_ID, SWORDSHIP_3_SKILL_ID, SWORDSHIP_4_SKILL_ID,
    SWORDSHIP_SKILL_ID,
};
use super::swordshipstate::SWORDSHIP_STATE_BYTES;
use super::taiji::TAIJI_SKILL_ID;
use super::taijistate::TAIJI_STATE_BYTES;
use super::tianshenxiafanstate::{TIAN_SHEN_XIA_FAN_STATE_BYTES, TIAN_SHEN_XIA_FAN_STATE_ID};
use super::wangshengstate::{WANGSHENG_STATE_BYTES, WANGSHENG_STATE_ID};
use super::weakstate::{WEAK_STATE_BYTES, WEAK_STATE_ID};
use super::wuxingstate::WUXING_STATE_BYTES;

const UNDEAD_STATE_ID: u32 = 0x38;
fn read_u32(payload: &[u8], offset: usize) -> Option<u32> {
    let bytes = payload.get(offset..offset.checked_add(4)?)?;
    Some(u32::from_le_bytes(bytes.try_into().ok()?))
}

struct StateRecordLayout {
    bytes: usize,
    decode: Option<fn(&[u8], usize) -> Option<StateData>>,
}

impl StateRecordLayout {
    fn typed(bytes: usize, decode: fn(&[u8], usize) -> Option<StateData>) -> Self {
        Self { bytes, decode: Some(decode) }
    }
}

fn record_layout(payload: &[u8], cursor: usize, state_id: u32) -> Option<StateRecordLayout> {
    let bytes = match state_id {
        CHANGE_BODY_STATE_ID => 124,
        EX_STATE_ID => 44,
        EX_STATE_NEW_ID => 56,
        UNDEAD_STATE_ID => 76,
        LEAF_CUT_STATE_ID => LEAF_CUT_STATE_BYTES,
        LEAF_CUT_2_STATE_ID => LEAF_CUT_2_STATE_BYTES,
        LEAF_CUT_3_STATE_ID => LEAF_CUT_3_STATE_BYTES,
        KEROSENE_STATE_ID => KEROSENE_STATE_BYTES,
        SWORDSHIP_SKILL_ID | SWORDSHIP_2_SKILL_ID | SWORDSHIP_3_SKILL_ID
        | SWORDSHIP_4_SKILL_ID => SWORDSHIP_STATE_BYTES,
        STRIKE_STATE_ID => STRIKE_STATE_BYTES,
        0x353..=0x357 => WUXING_STATE_BYTES,
        POISON_FOG_STATE_ID => POISON_FOG_STATE_BYTES,
        METEOR_ARROW_MASS_SKILL_ID => METEOR_ARROW_STATE_BYTES,
        BLIND_STATE_ID => BLIND_STATE_BYTES,
        KNOCK_OUT_STATE_ID => KNOCK_OUT_STATE_BYTES,
        SPIDER_WEB_SKILL_ID => SPIDER_WEB_STATE_BYTES,
        SEAL_STATE_ID => SEAL_STATE_BYTES,
        GOD_BLESS_STATE_ID | GOD_BLESS_STATE_2_ID => GOD_BLESS_STATE_BYTES,
        WEAK_STATE_ID => WEAK_STATE_BYTES,
        SOUL_COLLECT_STATE_ID => SOUL_COLLECT_STATE_BYTES,
        SPRITE_BURN_SKILL_ID => SPRITE_BURN_STATE_BYTES,
        SPIDER_POISON_SKILL_ID => SPIDER_POISON_STATE_BYTES,
        DAUB_POISON_STATE_ID => DAUB_POISON_STATE_BYTES,
        BOSS_BLUE_QUAKE_STATE_ID => BOSS_BLUE_QUAKE_STATE_BYTES,
        KNIGHT_CUT_STATE_ID => KNIGHT_CUT_STATE_BYTES,
        BOA_LOCK_STATE_ID => BOA_LOCK_STATE_BYTES,
        RUSH_STATE_ID => RUSH_STATE_BYTES,
        RUSH_2_STATE_ID => RUSH_2_STATE_BYTES,
        ROAR_STATE_ID => ROAR_STATE_BYTES,
        PILLAR_STATE_ID => PILLAR_STATE_BYTES,
        RAGE_BREAK_STATE_ID => RAGE_BREAK_STATE_BYTES,
        FURY_STATE_SKILL_ID => FURY_STATE_BYTES,
        HEAL_SKILL_ID | HEAL_2_SKILL_ID | SUPER_HEAL_SKILL_ID | SUPER_HEAL_2_SKILL_ID => HEAL_STATE_BYTES,
        state_id if state_id == RESTORE_HP_STATE_ID as u32 => {
            return Some(StateRecordLayout::typed(RESTORE_HP_STATE_BYTES, |bytes, offset| {
                ConsumableRestoreState::decode(bytes, offset).map(StateData::ConsumableRestore)
            }));
        }
        state_id if state_id == RESTORE_MP_STATE_ID as u32 => {
            return Some(StateRecordLayout::typed(RESTORE_MP_STATE_BYTES, |bytes, offset| {
                ConsumableRestoreState::decode(bytes, offset).map(StateData::ConsumableRestore)
            }));
        }
        state_id if is_automatic_restore_state_id(state_id) => AUTOMATIC_RESTORE_STATE_BYTES,
        PARTICULAR_STATE_ID => PARTICULAR_STATE_BYTES,
        state_id if state_id == TEAM_STATE_ID as u32 => CTeamState::serialized_size(payload, cursor)?,
        CURE_STATE_SKILL_ID => {
            return Some(StateRecordLayout::typed(CURE_STATE_BYTES, |bytes, offset| {
                CureState::decode(bytes, offset).ok().map(StateData::Cure)
            }));
        }
        ENLARGE_FULL_MISS_SKILL_ID => ENLARGE_FULL_MISS_STATE_BYTES,
        TAIJI_SKILL_ID => TAIJI_STATE_BYTES,
        ENLARGE_MAX_HP_SKILL_ID => ENLARGE_MAX_HP_STATE_BYTES,
        ENLARGE_MAX_MP_SKILL_ID => ENLARGE_MAX_MP_STATE_BYTES,
        ORIGIN_SKILL_ID => ORIGIN_STATE_BYTES,
        MACHINE_SHIELD_SKILL_ID => {
            return Some(StateRecordLayout::typed(MACHINE_SHIELD_STATE_BYTES, |bytes, offset| {
                MachineShieldState::decode(bytes, offset, 0).ok()
                    .map(|state| StateData::DefenseShield(DefenseShieldState::Machine(state)))
            }));
        }
        MANA_SHIELD_SKILL_ID => {
            return Some(StateRecordLayout::typed(MANA_SHIELD_STATE_BYTES, |bytes, offset| {
                ManaShieldState::decode(bytes, offset, 0).ok()
                    .map(|state| StateData::DefenseShield(DefenseShieldState::Mana(state)))
            }));
        }
        LIFE_SHIELD_SKILL_ID => {
            return Some(StateRecordLayout::typed(LIFE_SHIELD_STATE_BYTES, |bytes, offset| {
                LifeShieldState::decode(bytes, offset, 0).ok()
                    .map(|state| StateData::DefenseShield(DefenseShieldState::Life(state)))
            }));
        }
        PROMOTION_SKILL_ID => {
            return Some(StateRecordLayout::typed(PROMOTION_STATE_BYTES, |bytes, offset| {
                PromotionState::decode(bytes, offset, 0).ok()
                    .map(|state| StateData::DefenseShield(DefenseShieldState::Promotion(state)))
            }));
        }
        HEARTEN_SKILL_ID => HEARTEN_STATE_BYTES,
        super::agility::AGILITY_SKILL_ID | super::natural::NATURAL_SKILL_ID
        | super::rapture::RAPTURE_SKILL_ID => PERSISTENT_AGILITY_FAMILY_STATE_BYTES,
        super::agility2::AGILITY_2_SKILL_ID => AGILITY_STATE_2_BYTES,
        CALLOSITY_SKILL_ID | CALLOSITY_2_SKILL_ID => CALLOSITY_STATE_BYTES,
        super::bloodloss::BLOOD_LOSS_SKILL_ID => BLOOD_LOSS_STATE_BYTES,
        ENERGY_HOLDING_STATE_ID => ENERGY_HOLDING_STATE_BYTES,
        BOSS_BLUE_FURY_STATE_ID => BOSS_BLUE_FURY_STATE_BYTES,
        0x212..=0x219 => BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES,
        TIAN_SHEN_XIA_FAN_STATE_ID => TIAN_SHEN_XIA_FAN_STATE_BYTES,
        WANGSHENG_STATE_ID => WANGSHENG_STATE_BYTES,
        POISON_ARROW_SKILL_ID => POISON_ARROW_STATE_BYTES,
        state_id if ScriptMoveState::serialized_size(state_id as i32).is_some() => {
            ScriptMoveState::serialized_size(state_id as i32)?
        }
        RIDE_STATE_ID => {
            let name = payload.get(cursor.checked_add(16)?..)?;
            16 + name.iter().take(256).position(|byte| *byte == 0)? + 1
        }
        _ => return None,
    };
    Some(StateRecordLayout { bytes, decode: None })
}

pub(crate) fn decode_state_record(payload: &[u8], offset: usize) -> Option<StateData> {
    let layout = record_layout(payload, offset, read_u32(payload, offset)?)?;
    let _ = payload.get(offset..offset.checked_add(layout.bytes)?)?;
    (layout.decode?)(payload, offset)
}

/// Возвращает только доказанные начала записей в исходном порядке.
pub(crate) fn known_state_record_offsets(payload: &[u8]) -> Vec<usize> {
    let Some(declared_count) = read_u32(payload, 0) else {
        return Vec::new();
    };
    let mut offsets = Vec::new();
    let mut cursor = 4usize;
    for _ in 0..declared_count {
        let Some(state_id) = read_u32(payload, cursor) else { break };
        let Some(layout) = record_layout(payload, cursor, state_id) else { break };
        let Some(end) = cursor.checked_add(layout.bytes).filter(|end| *end <= payload.len()) else { break };
        offsets.push(cursor);
        cursor = end;
    }
    offsets
}
