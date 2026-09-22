//! Состояние и правила игровых клиентских сессий Zone.

mod sequence;
mod validation;

pub use sequence::{
    CSequenceRegistry, CSequenceString, SequenceRegistryInitializationError, SequenceSerializeError,
};
pub use validation::{
    LoginValidationRelease, LoginValidationState, PlayerLoginValidateTime, PreparedSequence,
    SequencePreparationError,
};
