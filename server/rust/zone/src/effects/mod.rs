//! Наложенные состояния Zone; живой владелец фигуры подключается переходным Game.

mod automaticrestore;
mod bloodloss;
mod leafcut;
mod meteorarrow;
mod periodicattack;
mod poison;
mod time;
mod visualeffect;

pub use automaticrestore::{
    AUTOMATIC_RESTORE_HP_FIGHT_STATE_ID, AUTOMATIC_RESTORE_HP_PEACE_STATE_ID,
    AUTOMATIC_RESTORE_MP_FIGHT_STATE_ID, AUTOMATIC_RESTORE_MP_PEACE_STATE_ID,
    AUTOMATIC_RESTORE_STATE_BYTES, AutomaticRestoreKind, AutomaticRestoreMutation,
    AutomaticRestoreProperties, AutomaticRestoreState, is_automatic_restore_state_id,
};
pub use bloodloss::{
    BLOOD_LOSS_STATE_BYTES, BLOOD_LOSS_STATE_ID, BloodLossAttackSeed, BloodLossState,
};
pub use leafcut::{
    LEAF_CUT_2_STATE_ID, LEAF_CUT_3_STATE_ID, LEAF_CUT_STATE_BYTES, LEAF_CUT_STATE_ID,
    LeafCutAttackSeed, LeafCutState,
};
pub use meteorarrow::{METEOR_ARROW_MASS_SKILL_ID, METEOR_ARROW_STATE_BYTES, MeteorArrowState};
pub use periodicattack::{
    PeriodicAttackCore, PeriodicAttackRecord, encode_periodic_state,
    encode_periodic_state_for_install,
};
pub use poison::{POISON_STATE_BYTES, PoisonState};
pub use time::timed_client_state_time;
pub use visualeffect::CVisualEffect;
