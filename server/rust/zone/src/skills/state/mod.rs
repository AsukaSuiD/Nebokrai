//! Состояния Zone: арена экземпляров мобильного носителя (`storage`,
//! бывший `moveshape/state_storage.rs` переходного Game) и клиентские
//! контракты (`catalog` — проекция живых записей для
//! `CMoveShape::AddToByteArray_ForClient` и runtime-план visual после Begin).

mod catalog;
mod storage;

pub use catalog::{StateClientRecord, registered_runtime_state_visual, state_client_record};
pub use storage::{
    AppliedState, AppliedStateEntries, CanonicalStateStorage, LegacyStateCodec, StateBatch,
    StateData, StateKey, StateSerialization,
};
