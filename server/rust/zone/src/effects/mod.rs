//! Наложенные состояния Zone; живой владелец фигуры подключается переходным Game.

mod agility;
mod automaticrestore;
mod bloodloss;
mod element;
mod fullmiss;
mod hearten;
mod leafcut;
mod maxresource;
mod meteorarrow;
mod periodicattack;
mod poison;
mod poisonfog;
mod swordship;
mod time;
mod visualeffect;
mod wuxing;

pub use agility::{
    AGILITY_2_SKILL_ID, AGILITY_SKILL_ID, AGILITY_STATE_2_BYTES, AgilityState2, NATURAL_SKILL_ID,
    PERSISTENT_AGILITY_FAMILY_STATE_BYTES, PersistentAgilityFamilyState,
    PersistentAgilityProperties, RAPTURE_SKILL_ID,
};
pub use automaticrestore::{
    AUTOMATIC_RESTORE_HP_FIGHT_STATE_ID, AUTOMATIC_RESTORE_HP_PEACE_STATE_ID,
    AUTOMATIC_RESTORE_MP_FIGHT_STATE_ID, AUTOMATIC_RESTORE_MP_PEACE_STATE_ID,
    AUTOMATIC_RESTORE_STATE_BYTES, AutomaticRestoreKind, AutomaticRestoreMutation,
    AutomaticRestoreProperties, AutomaticRestoreState, is_automatic_restore_state_id,
};
pub use bloodloss::{
    BLOOD_LOSS_STATE_BYTES, BLOOD_LOSS_STATE_ID, BloodLossAttackSeed, BloodLossState,
};
pub use element::{
    ELEMENT_STATE_BYTES, ElementState, ORIGIN_STATE_ID, OriginState, TAIJI_STATE_ID, TaiJiState,
};
pub use fullmiss::{
    ENLARGE_FULL_MISS_STATE_BYTES, ENLARGE_FULL_MISS_STATE_ID, EnlargeFullMissState,
};
pub use hearten::{HEARTEN_STATE_BYTES, HEARTEN_STATE_ID, HeartenState};
pub use leafcut::{
    LEAF_CUT_2_STATE_ID, LEAF_CUT_3_STATE_ID, LEAF_CUT_STATE_BYTES, LEAF_CUT_STATE_ID,
    LeafCutAttackSeed, LeafCutState,
};
pub use maxresource::{
    ENLARGE_MAX_HP_STATE_ID, ENLARGE_MAX_MP_STATE_ID, EnlargeMaxHpState, EnlargeMaxMpState,
    MAX_RESOURCE_STATE_BYTES, MaxResourceState,
};
pub use meteorarrow::{METEOR_ARROW_MASS_SKILL_ID, METEOR_ARROW_STATE_BYTES, MeteorArrowState};
pub use periodicattack::{
    PeriodicAttackCore, PeriodicAttackRecord, encode_periodic_state,
    encode_periodic_state_for_install,
};
pub use poison::{POISON_STATE_BYTES, PoisonState};
pub use poisonfog::{POISON_FOG_STATE_BYTES, POISON_FOG_STATE_ID, PoisonFogState};
pub use swordship::{
    SWORDSHIP_2_STATE_ID, SWORDSHIP_3_STATE_ID, SWORDSHIP_4_STATE_ID, SWORDSHIP_STATE_BYTES,
    SWORDSHIP_STATE_ID, SwordshipState, is_swordship_state_id,
};
pub use time::timed_client_state_time;
pub use visualeffect::CVisualEffect;
pub use wuxing::{
    WUXING_EARTH_STATE_ID, WUXING_FIRE_STATE_ID, WUXING_METAL_STATE_ID, WUXING_STATE_BYTES,
    WUXING_WATER_STATE_ID, WUXING_WOOD_STATE_ID, WuXingCoefficients, WuXingKind, WuXingProperties,
    WuXingState, WuXingStateParameters, apply_wuxing_to_properties, kind_for_skill_id,
};
