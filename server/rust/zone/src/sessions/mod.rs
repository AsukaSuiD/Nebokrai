//! Состояние и правила игровых клиентских сессий Zone.

mod sequence;

pub use sequence::{
    CSequenceRegistry, CSequenceString, SequenceRegistryInitializationError, SequenceSerializeError,
};
