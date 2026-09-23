//! Наложенные состояния Zone; живой владелец фигуры подключается переходным Game.

mod automaticrestore;
mod time;
mod visualeffect;

pub use automaticrestore::{
    AUTOMATIC_RESTORE_HP_FIGHT_STATE_ID, AUTOMATIC_RESTORE_HP_PEACE_STATE_ID,
    AUTOMATIC_RESTORE_MP_FIGHT_STATE_ID, AUTOMATIC_RESTORE_MP_PEACE_STATE_ID,
    AUTOMATIC_RESTORE_STATE_BYTES, AutomaticRestoreKind, AutomaticRestoreMutation,
    AutomaticRestoreProperties, AutomaticRestoreState, is_automatic_restore_state_id,
};
pub use time::timed_client_state_time;
pub use visualeffect::CVisualEffect;
