//! Безопасная Rust-проекция `CStateFactory::Unserialize` GameServer.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/statefactory.cpp`, RVA `0x001D7D00`. Native factory читает
//! ID, создаёт concrete state и поручает ему потребить запись. Rust хранит
//! concrete состояния у `CMoveShape`; этот owner сохраняет switch ID→layout и
//! последовательное продвижение по GameSave wire. Неизвестный ID, переменная
//! запись без терминатора и усечённый payload останавливают типизацию до
//! спорной записи: исходный хвост остаётся в `LegacyStateCodec`.
//! Каждая достигнутая ветвь одновременно задаёт layout и concrete decoder:
//! один проход CMoveShape переносит записи в общую арену в wire-порядке,
//! включая повторные ID. Отдельных decoder-проходов по семействам нет.
//!
//! CRT allocation, RTTI, vtable и exception plumbing не воспроизводятся.

use crate::gameserver::appserver::chbystate::CHANGE_BODY_STATE_ID;
use crate::gameserver::appserver::exstate::{EX_STATE_ID, EX_STATE_NEW_ID};
use crate::gameserver::appserver::moveshape::StateData;
use crate::gameserver::appserver::shape::ShapeIdentity;
use super::skillfactory::CSkillFactory;
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
use super::curestate::{CURE_STATE_BYTES, CURE_STATE_SKILL_ID};
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

type StateDecoder = fn(&[u8], usize, ShapeIdentity, &CSkillFactory) -> Option<StateData>;

struct StateRecordLayout {
    bytes: usize,
    decode: StateDecoder,
}

impl StateRecordLayout {
    fn typed(bytes: usize, decode: StateDecoder) -> Self {
        Self { bytes, decode }
    }
}

fn record_layout(payload: &[u8], cursor: usize, state_id: u32) -> Option<StateRecordLayout> {
    Some(match state_id {
        CHANGE_BODY_STATE_ID => StateRecordLayout::typed(
            124, |payload, offset, _owner, _factory| {
                crate::gameserver::appserver::chbystate::ChangeBodyState::decode_at(payload, offset, 0).map(StateData::ChangeBody)
            },
        ),
        EX_STATE_ID => StateRecordLayout::typed(
            44, |payload, offset, _owner, _factory| {
                crate::gameserver::appserver::exstate::ExtendedState::decode_at(payload, offset, 0).map(StateData::Extended)
            },
        ),
        EX_STATE_NEW_ID => StateRecordLayout::typed(
            56, |payload, offset, _owner, _factory| {
                crate::gameserver::appserver::exstate::ExtendedState::decode_at(payload, offset, 0).map(StateData::Extended)
            },
        ),
        UNDEAD_STATE_ID => StateRecordLayout::typed(
            76, |payload, offset, _owner, _factory| {
                crate::gameserver::appserver::moveshape::UndeadState::decode_at(payload, offset, 0).map(StateData::Undead)
            },
        ),
        LEAF_CUT_STATE_ID => StateRecordLayout::typed(
            LEAF_CUT_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::leafcutstate::LeafCutState::decode(payload, offset, 0).ok().map(StateData::LeafCut)
            },
        ),
        LEAF_CUT_2_STATE_ID => StateRecordLayout::typed(
            LEAF_CUT_2_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::leafcutstate2::LeafCutState2::decode(payload, offset, 0).ok().map(StateData::LeafCut2)
            },
        ),
        LEAF_CUT_3_STATE_ID => StateRecordLayout::typed(
            LEAF_CUT_3_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::leafcutstate3::LeafCutState3::decode(payload, offset, 0).ok().map(StateData::LeafCut3)
            },
        ),
        KEROSENE_STATE_ID => StateRecordLayout::typed(
            KEROSENE_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::kerosenestate::KeroseneState::decode(payload, offset, 0).ok().map(StateData::Kerosene)
            },
        ),
        SWORDSHIP_SKILL_ID | SWORDSHIP_2_SKILL_ID | SWORDSHIP_3_SKILL_ID | SWORDSHIP_4_SKILL_ID => StateRecordLayout::typed(
            SWORDSHIP_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::swordshipstate::SwordshipState::decode(payload, offset).ok().map(StateData::Swordship)
            },
        ),
        STRIKE_STATE_ID => StateRecordLayout::typed(
            STRIKE_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::strikestate::StrikeState::decode(payload, offset).ok().map(StateData::Strike)
            },
        ),
        0x353..=0x357 => StateRecordLayout::typed(
            WUXING_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::wuxingstate::WuXingState::decode(payload, offset).ok().map(StateData::WuXing)
            },
        ),
        POISON_FOG_STATE_ID => StateRecordLayout::typed(
            POISON_FOG_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::poisonfogstate::PoisonFogState::decode(payload, offset, 0).ok().map(StateData::PoisonFog)
            },
        ),
        METEOR_ARROW_MASS_SKILL_ID => StateRecordLayout::typed(
            METEOR_ARROW_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::meteorarrowstate::MeteorArrowState::decode(payload, offset).ok().map(StateData::MeteorArrow)
            },
        ),
        BLIND_STATE_ID => StateRecordLayout::typed(
            BLIND_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::blindstate::BlindState::decode(payload, offset).ok().map(StateData::Blind)
            },
        ),
        KNOCK_OUT_STATE_ID => StateRecordLayout::typed(
            KNOCK_OUT_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::knockoutstate::KnockOutState::decode(payload, offset).ok().map(StateData::KnockOut)
            },
        ),
        SPIDER_WEB_SKILL_ID => StateRecordLayout::typed(
            SPIDER_WEB_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::spiderwebstate::SpiderWebState::decode(payload, offset).ok().map(StateData::SpiderWeb)
            },
        ),
        SEAL_STATE_ID => StateRecordLayout::typed(
            SEAL_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::sealstate::SealState::decode(payload, offset).ok().map(StateData::Seal)
            },
        ),
        GOD_BLESS_STATE_ID | GOD_BLESS_STATE_2_ID => StateRecordLayout::typed(
            GOD_BLESS_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::godblessstate::GodBlessState::decode(payload, offset).ok().map(StateData::GodBless)
            },
        ),
        WEAK_STATE_ID => StateRecordLayout::typed(
            WEAK_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::weakstate::WeakState::decode(payload, offset).ok().map(StateData::Weak)
            },
        ),
        SOUL_COLLECT_STATE_ID => StateRecordLayout::typed(
            SOUL_COLLECT_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::soulcollectstate::SoulCollectState::decode(payload, offset).ok().map(StateData::SoulCollect)
            },
        ),
        SPRITE_BURN_SKILL_ID => StateRecordLayout::typed(
            SPRITE_BURN_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::spriteburnstate::SpriteBurnState::decode(payload, offset, 0).ok().map(StateData::SpriteBurn)
            },
        ),
        SPIDER_POISON_SKILL_ID => StateRecordLayout::typed(
            SPIDER_POISON_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::spiderpoisonstate::SpiderPoisonState::decode(payload, offset, 0).ok().map(StateData::SpiderPoison)
            },
        ),
        DAUB_POISON_STATE_ID => StateRecordLayout::typed(
            DAUB_POISON_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::daubpoisonstate::DaubPoisonState::decode(payload, offset).ok().map(StateData::DaubPoison)
            },
        ),
        BOSS_BLUE_QUAKE_STATE_ID => StateRecordLayout::typed(
            BOSS_BLUE_QUAKE_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::bossbluequakestate::BossBlueQuakeState::decode(payload, offset).ok().map(StateData::BossBlueQuake)
            },
        ),
        KNIGHT_CUT_STATE_ID => StateRecordLayout::typed(
            KNIGHT_CUT_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::knightcutstate::KnightCutState::decode(payload, offset).ok().map(StateData::KnightCut)
            },
        ),
        BOA_LOCK_STATE_ID => StateRecordLayout::typed(
            BOA_LOCK_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::boalockstate::BoaLockState::decode(payload, offset).ok().map(StateData::BoaLock)
            },
        ),
        RUSH_STATE_ID => StateRecordLayout::typed(
            RUSH_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::rushstate::RushState::decode(payload, offset).ok().map(StateData::Rush)
            },
        ),
        RUSH_2_STATE_ID => StateRecordLayout::typed(
            RUSH_2_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::rushstate2::Rush2State::decode(payload, offset).ok().map(StateData::Rush2)
            },
        ),
        ROAR_STATE_ID => StateRecordLayout::typed(
            ROAR_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::roarstate::RoarState::decode(payload, offset).ok().map(StateData::Roar)
            },
        ),
        PILLAR_STATE_ID => StateRecordLayout::typed(
            PILLAR_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::pillarstate::PillarState::decode(payload, offset).ok().map(StateData::Pillar)
            },
        ),
        RAGE_BREAK_STATE_ID => StateRecordLayout::typed(
            RAGE_BREAK_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::ragebreakstate::RageBreakState::decode(payload, offset).ok().map(StateData::RageBreak)
            },
        ),
        FURY_STATE_SKILL_ID => StateRecordLayout::typed(
            FURY_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::furystate::FuryState::decode(payload, offset).ok().map(StateData::Fury)
            },
        ),
        HEAL_SKILL_ID | HEAL_2_SKILL_ID | SUPER_HEAL_SKILL_ID | SUPER_HEAL_2_SKILL_ID => StateRecordLayout::typed(
            HEAL_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::healstate::HealState::decode(payload, offset, _owner).ok().map(StateData::Heal)
            },
        ),
        CURE_STATE_SKILL_ID => StateRecordLayout::typed(
            CURE_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::curestate::CureState::decode(payload, offset).ok().map(StateData::Cure)
            },
        ),
        ENLARGE_FULL_MISS_SKILL_ID => StateRecordLayout::typed(
            ENLARGE_FULL_MISS_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::enlargefullmissstate::EnlargeFullMissState::decode(payload, offset).ok().map(StateData::EnlargeFullMiss)
            },
        ),
        TAIJI_SKILL_ID => StateRecordLayout::typed(
            TAIJI_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::taijistate::TaiJiState::decode(payload, offset).ok().map(StateData::TaiJi)
            },
        ),
        ENLARGE_MAX_HP_SKILL_ID => StateRecordLayout::typed(
            ENLARGE_MAX_HP_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::enlargemaxhpstate::EnlargeMaxHpState::decode(payload, offset).ok().map(StateData::EnlargeMaxHp)
            },
        ),
        ENLARGE_MAX_MP_SKILL_ID => StateRecordLayout::typed(
            ENLARGE_MAX_MP_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::enlargemaxmpstate::EnlargeMaxMpState::decode(payload, offset).ok().map(StateData::EnlargeMaxMp)
            },
        ),
        ORIGIN_SKILL_ID => StateRecordLayout::typed(
            ORIGIN_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::originstate::OriginState::decode(payload, offset).ok().map(StateData::Origin)
            },
        ),
        HEARTEN_SKILL_ID => StateRecordLayout::typed(
            HEARTEN_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::heartenstate::HeartenState::decode(payload, offset, 0).ok().map(StateData::Hearten)
            },
        ),
        super::agility::AGILITY_SKILL_ID | super::natural::NATURAL_SKILL_ID | super::rapture::RAPTURE_SKILL_ID => StateRecordLayout::typed(
            PERSISTENT_AGILITY_FAMILY_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::agilitystate::PersistentAgilityFamilyState::decode(payload, offset).ok().map(StateData::PersistentAgility)
            },
        ),
        super::agility2::AGILITY_2_SKILL_ID => StateRecordLayout::typed(
            AGILITY_STATE_2_BYTES, |payload, offset, _owner, _factory| {
                super::agilitystate2::AgilityState2::decode(payload, offset, 0).ok().map(StateData::Agility2)
            },
        ),
        CALLOSITY_SKILL_ID | CALLOSITY_2_SKILL_ID => StateRecordLayout::typed(
            CALLOSITY_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::callositystate::CallosityFamilyState::decode(payload, offset).ok().map(StateData::Callosity)
            },
        ),
        super::bloodloss::BLOOD_LOSS_SKILL_ID => StateRecordLayout::typed(
            BLOOD_LOSS_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::bloodlossstate::BloodLossState::decode(payload, offset, 0).ok().map(StateData::BloodLoss)
            },
        ),
        BOSS_BLUE_FURY_STATE_ID => StateRecordLayout::typed(
            BOSS_BLUE_FURY_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::bossbluefurystate::BossBlueFuryState::decode(payload, offset, 0).ok().map(StateData::BossBlueFury)
            },
        ),
        0x212..=0x219 => StateRecordLayout::typed(
            BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::battlefairyattributestate::BattleFairyAttributeState::decode(payload, offset).ok().map(StateData::BattleFairyAttribute)
            },
        ),
        TIAN_SHEN_XIA_FAN_STATE_ID => StateRecordLayout::typed(
            TIAN_SHEN_XIA_FAN_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::tianshenxiafanstate::TianShenXiaFanState::decode(payload, offset).ok().map(StateData::TianShenXiaFan)
            },
        ),
        WANGSHENG_STATE_ID => StateRecordLayout::typed(
            WANGSHENG_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::wangshengstate::WangshengState::decode(payload, offset).ok().map(StateData::Wangsheng)
            },
        ),
        POISON_ARROW_SKILL_ID => StateRecordLayout::typed(
            POISON_ARROW_STATE_BYTES, |payload, offset, _owner, _factory| {
                super::poisonarrowstate::PoisonArrowState::decode(payload, offset, 0).ok().map(StateData::PoisonArrow)
            },
        ),
        state_id if state_id == RESTORE_HP_STATE_ID as u32 => StateRecordLayout::typed(
            RESTORE_HP_STATE_BYTES, |payload, offset, _owner, _factory| {
                ConsumableRestoreState::decode(payload, offset).map(StateData::ConsumableRestore)
            },
        ),
        state_id if state_id == RESTORE_MP_STATE_ID as u32 => StateRecordLayout::typed(
            RESTORE_MP_STATE_BYTES, |payload, offset, _owner, _factory| {
                ConsumableRestoreState::decode(payload, offset).map(StateData::ConsumableRestore)
            },
        ),
        state_id if is_automatic_restore_state_id(state_id) => StateRecordLayout::typed(
            AUTOMATIC_RESTORE_STATE_BYTES, |payload, offset, _owner, _factory| {
                crate::gameserver::appserver::states::automaticrestore::AutomaticRestoreState::decode(payload, offset).map(StateData::AutomaticRestore)
            },
        ),
        PARTICULAR_STATE_ID => StateRecordLayout::typed(
            PARTICULAR_STATE_BYTES, |payload, offset, _owner, _factory| {
                crate::gameserver::appserver::particularstate::ParticularState::decode(payload, offset).ok().map(StateData::Particular)
            },
        ),
        state_id if state_id == TEAM_STATE_ID as u32 => StateRecordLayout::typed(
            CTeamState::serialized_size(payload, cursor)?, |payload, offset, _owner, _factory| {
                CTeamState::decode(payload, offset).ok().map(StateData::Team)
            },
        ),
        state_id if ScriptMoveState::serialized_size(state_id as i32).is_some() => StateRecordLayout::typed(
            ScriptMoveState::serialized_size(state_id as i32)?, |payload, offset, _owner, _factory| {
                ScriptMoveState::decode(payload, offset).ok().map(StateData::Script)
            },
        ),
        RIDE_STATE_ID => StateRecordLayout::typed(
            17 + payload.get(cursor.checked_add(16)?..)?.iter().take(256).position(|byte| *byte == 0)?, |payload, offset, _owner, _factory| {
                crate::gameserver::appserver::ridestate::RideState::decode_at(payload, offset).map(StateData::Ride)
            },
        ),
        MACHINE_SHIELD_SKILL_ID => StateRecordLayout::typed(
            MACHINE_SHIELD_STATE_BYTES, |payload, offset, _owner, _factory| {
                MachineShieldState::decode(payload, offset, 0).ok().map(|state| StateData::DefenseShield(DefenseShieldState::Machine(state)))
            },
        ),
        MANA_SHIELD_SKILL_ID => StateRecordLayout::typed(
            MANA_SHIELD_STATE_BYTES, |payload, offset, _owner, _factory| {
                ManaShieldState::decode(payload, offset, 0).ok().map(|state| StateData::DefenseShield(DefenseShieldState::Mana(state)))
            },
        ),
        LIFE_SHIELD_SKILL_ID => StateRecordLayout::typed(
            LIFE_SHIELD_STATE_BYTES, |payload, offset, _owner, _factory| {
                LifeShieldState::decode(payload, offset, 0).ok().map(|state| StateData::DefenseShield(DefenseShieldState::Life(state)))
            },
        ),
        PROMOTION_SKILL_ID => StateRecordLayout::typed(
            PROMOTION_STATE_BYTES, |payload, offset, _owner, _factory| {
                PromotionState::decode(payload, offset, 0).ok().map(|state| StateData::DefenseShield(DefenseShieldState::Promotion(state)))
            },
        ),
        ENERGY_HOLDING_STATE_ID => StateRecordLayout::typed(
            ENERGY_HOLDING_STATE_BYTES, |payload, offset, _owner, _factory| {
                {
                let level = read_u32(payload, offset + 4)?;
                let percent = _factory.query_skill_base_properties(ENERGY_HOLDING_STATE_ID, level as i32)?
                    .query_property(super::energyholding::PARAMETER_PERCENT);
                super::energyholdingstate::EnergyHoldingState::decode(payload, offset, percent).ok()
                    .map(StateData::EnergyHolding)
            }
            },
        ),
        _ => return None,
    })
}

pub(crate) fn decode_state_record(
    payload: &[u8],
    offset: usize,
    owner: ShapeIdentity,
    factory: &CSkillFactory,
) -> Option<StateData> {
    let layout = record_layout(payload, offset, read_u32(payload, offset)?)?;
    let _ = payload.get(offset..offset.checked_add(layout.bytes)?)?;
    (layout.decode)(payload, offset, owner, factory)
}

/// Возвращает только доказанные начала записей в исходном порядке.
pub(crate) fn known_state_record_offsets(payload: &[u8]) -> Vec<usize> {
    known_state_record_spans(payload).into_iter().map(|(offset, _)| offset).collect()
}

/// Границы тех же достигнутых записей, включая точный конец последней.
pub(crate) fn known_state_record_spans(payload: &[u8]) -> Vec<(usize, usize)> {
    let Some(declared_count) = read_u32(payload, 0) else {
        return Vec::new();
    };
    let mut offsets = Vec::new();
    let mut cursor = 4usize;
    for _ in 0..declared_count {
        let Some(state_id) = read_u32(payload, cursor) else { break };
        let Some(layout) = record_layout(payload, cursor, state_id) else { break };
        let Some(end) = cursor.checked_add(layout.bytes).filter(|end| *end <= payload.len()) else { break };
        offsets.push((cursor, layout.bytes));
        cursor = end;
    }
    offsets
}
