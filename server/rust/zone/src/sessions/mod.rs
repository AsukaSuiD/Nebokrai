//! Состояние и правила игровых клиентских сессий Zone.

pub mod cplug;
pub mod csession;
pub mod cteam;
pub mod cteamate;

mod sequence;
mod validation;

pub use sequence::{
    CSequenceRegistry, CSequenceString, SequenceRegistryInitializationError, SequenceSerializeError,
};
pub use validation::{
    LoginValidationRelease, LoginValidationState, PreparedSequence, SequencePreparationError,
};
