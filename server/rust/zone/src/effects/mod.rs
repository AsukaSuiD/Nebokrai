//! Наложенные состояния Zone; живой владелец фигуры подключается переходным Game.

mod agility;
mod attackgain;
mod automaticrestore;
mod battlefairy;
mod bloodloss;
mod blind;
mod bossbluefury;
mod callosity;
mod cure;
mod daubpoison;
mod element;
mod energyholding;
mod fullmiss;
mod godbless;
mod heal;
mod hearten;
mod leafcut;
mod lifeshield;
mod machineshield;
mod manashield;
mod maxresource;
mod meteorarrow;
mod periodicattack;
mod pillar;
mod poison;
mod poisonfog;
mod promotion;
mod roar;
mod shieldabsorption;
mod soulcollect;
mod swordship;
mod tianshenxiafan;
mod time;
mod visualeffect;
mod wangsheng;
mod weak;
mod wuxing;

pub use agility::{
    AGILITY_2_SKILL_ID, AGILITY_SKILL_ID, AGILITY_STATE_2_BYTES, AgilityState2, NATURAL_SKILL_ID,
    PERSISTENT_AGILITY_FAMILY_STATE_BYTES, PersistentAgilityFamilyState,
    PersistentAgilityProperties, RAPTURE_SKILL_ID,
};
pub use attackgain::{
    ATTACK_GAIN_STATE_BYTES, AttackGainState, FURY_STATE_SKILL_ID, FuryState,
    RAGE_BREAK_STATE_ID, RageBreakState,
};
pub use automaticrestore::{
    AUTOMATIC_RESTORE_HP_FIGHT_STATE_ID, AUTOMATIC_RESTORE_HP_PEACE_STATE_ID,
    AUTOMATIC_RESTORE_MP_FIGHT_STATE_ID, AUTOMATIC_RESTORE_MP_PEACE_STATE_ID,
    AUTOMATIC_RESTORE_STATE_BYTES, AutomaticRestoreKind, AutomaticRestoreMutation,
    AutomaticRestoreProperties, AutomaticRestoreState, is_automatic_restore_state_id,
};
pub use battlefairy::{
    BATTLE_FAIRY_ATTRIBUTE_STATE_BYTES, BattleFairyAttributeKind,
    BattleFairyAttributePlayerView, BattleFairyAttributeState, battle_fairy_attribute_kind,
};
pub use bloodloss::{
    BLOOD_LOSS_STATE_BYTES, BLOOD_LOSS_STATE_ID, BloodLossAttackSeed, BloodLossState,
};
pub use bossbluefury::{
    BOSS_BLUE_FURY_STATE_BYTES, BOSS_BLUE_FURY_STATE_ID, BossBlueFuryState,
};
pub use blind::{
    BLIND_STATE_BYTES, BLIND_STATE_ID, BOA_LOCK_STATE_BYTES, BOA_LOCK_STATE_ID,
    BOSS_BLUE_QUAKE_STATE_BYTES, BOSS_BLUE_QUAKE_STATE_ID, BlindState, BoaLockState,
    BossBlueQuakeState, KNIGHT_CUT_STATE_BYTES, KNIGHT_CUT_STATE_ID, KNOCK_OUT_STATE_BYTES,
    KNOCK_OUT_STATE_ID, KnightCutState, KnockOutState, SPIDER_WEB_STATE_BYTES,
    SPIDER_WEB_STATE_ID, SpiderWebState,
};
pub use energyholding::{ENERGY_HOLDING_STATE_BYTES, ENERGY_HOLDING_STATE_ID, EnergyHoldingState};
pub use callosity::{
    CALLOSITY_2_SKILL_ID, CALLOSITY_SKILL_ID, CALLOSITY_STATE_BYTES, CallosityFamilyState,
};
pub use cure::{CURE_STATE_BYTES, CURE_STATE_SKILL_ID, CureState};
pub use daubpoison::{DAUB_POISON_STATE_BYTES, DAUB_POISON_STATE_ID, DaubPoisonState};
pub use element::{
    ELEMENT_STATE_BYTES, ElementState, ORIGIN_STATE_ID, OriginState, TAIJI_STATE_ID, TaiJiState,
};
pub use fullmiss::{
    ENLARGE_FULL_MISS_STATE_BYTES, ENLARGE_FULL_MISS_STATE_ID, EnlargeFullMissState,
};
pub use godbless::{GOD_BLESS_STATE_2_ID, GOD_BLESS_STATE_BYTES, GOD_BLESS_STATE_ID, GodBlessState};
pub use heal::{HEAL_STATE_BYTES, HealState};
pub use hearten::{HEARTEN_STATE_BYTES, HEARTEN_STATE_ID, HeartenState};
pub use leafcut::{
    LEAF_CUT_2_STATE_ID, LEAF_CUT_3_STATE_ID, LEAF_CUT_STATE_BYTES, LEAF_CUT_STATE_ID,
    LeafCutAttackSeed, LeafCutState,
};
pub use lifeshield::{LIFE_SHIELD_SKILL_ID, LIFE_SHIELD_STATE_BYTES, LifeShieldState};
pub use machineshield::{MACHINE_SHIELD_SKILL_ID, MACHINE_SHIELD_STATE_BYTES, MachineShieldState};
pub use manashield::{MANA_SHIELD_SKILL_ID, MANA_SHIELD_STATE_BYTES, ManaShieldState};
pub use maxresource::{
    ENLARGE_MAX_HP_STATE_ID, ENLARGE_MAX_MP_STATE_ID, EnlargeMaxHpState, EnlargeMaxMpState,
    MAX_RESOURCE_STATE_BYTES, MaxResourceState,
};
pub use meteorarrow::{METEOR_ARROW_MASS_SKILL_ID, METEOR_ARROW_STATE_BYTES, MeteorArrowState};
pub use periodicattack::{
    PeriodicAttackCore, PeriodicAttackRecord, encode_periodic_state,
    encode_periodic_state_for_install,
};
pub use pillar::{PILLAR_STATE_BYTES, PILLAR_STATE_ID, PillarState};
pub use poison::{POISON_STATE_BYTES, PoisonState};
pub use poisonfog::{POISON_FOG_STATE_BYTES, POISON_FOG_STATE_ID, PoisonFogState};
pub use promotion::{PROMOTION_STATE_BYTES, PROMOTION_STATE_ID, PromotionState};
pub use roar::{ROAR_STATE_BYTES, ROAR_STATE_ID, RoarState};
pub use soulcollect::{SOUL_COLLECT_STATE_BYTES, SOUL_COLLECT_STATE_ID, SoulCollectState};
pub use swordship::{
    SWORDSHIP_2_STATE_ID, SWORDSHIP_3_STATE_ID, SWORDSHIP_4_STATE_ID, SWORDSHIP_STATE_BYTES,
    SWORDSHIP_STATE_ID, SwordshipState, is_swordship_state_id,
};
pub use tianshenxiafan::{
    TIAN_SHEN_XIA_FAN_STATE_BYTES, TIAN_SHEN_XIA_FAN_STATE_ID, TianShenXiaFanPlayerView,
    TianShenXiaFanState,
};
pub use time::timed_client_state_time;
pub use visualeffect::CVisualEffect;
pub use wangsheng::{WANGSHENG_STATE_BYTES, WANGSHENG_STATE_ID, WangshengState};
pub use weak::{WEAK_STATE_BYTES, WEAK_STATE_ID, WeakState};
pub use wuxing::{
    WUXING_EARTH_STATE_ID, WUXING_FIRE_STATE_ID, WUXING_METAL_STATE_ID, WUXING_STATE_BYTES,
    WUXING_WATER_STATE_ID, WUXING_WOOD_STATE_ID, WuXingCoefficients, WuXingKind, WuXingProperties,
    WuXingState, WuXingStateParameters, apply_wuxing_to_properties, kind_for_skill_id,
};
