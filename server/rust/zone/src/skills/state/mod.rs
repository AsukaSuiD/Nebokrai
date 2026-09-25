//! Состояния Zone: арена экземпляров мобильного носителя (`storage`,
//! бывший `moveshape/state_storage.rs` переходного Game) и клиентские
//! контракты (`catalog` — проекция живых записей для
//! `CMoveShape::AddToByteArray_ForClient` и runtime-план visual после Begin).

mod catalog; // клиентская проекция живых состояний и runtime-план их visual.
mod storage; // арена экземпляров состояний мобильного носителя.

pub use catalog::{StateClientRecord, registered_runtime_state_visual, state_client_record}; // клиентская запись состояния и runtime-план visual.
pub use storage::{
    AppliedState, AppliedStateEntries, CanonicalStateStorage, LegacyStateCodec, StateBatch,
    StateData, StateKey, StateSerialization,
}; // хранилище состояний и его кодек сериализации.
