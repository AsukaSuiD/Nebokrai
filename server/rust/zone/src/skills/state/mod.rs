//! Состояния Zone: арена экземпляров мобильного носителя (`storage`,
//! бывший `moveshape/state_storage.rs` переходного Game), клиентские
//! контракты (`catalog` — проекция живых записей для
//! `CMoveShape::AddToByteArray_ForClient` и runtime-план visual после Begin)
//! и DB Save/Load (`serialization` — кодек проекции GameSave, бывшая
//! Save/Load-семья hub `appserver/moveshape.rs`).

mod catalog; // клиентская проекция живых состояний и runtime-план их visual.
mod serialization; // DB Save/Load-кодек состояний GameSave и сброс persisted snapshot.
mod storage; // арена экземпляров состояний мобильного носителя.

pub use catalog::{StateClientRecord, registered_runtime_state_visual, state_client_record}; // клиентская запись состояния и runtime-план visual.
pub use serialization::{
    clear_persisted_runtime_state, read_i16, read_i32, read_u16, read_u32, replace_ex_states,
    serialize_ex_states_for_save, serialized_ex_states, update_known_state_record,
    update_nth_known_state_record, write_i16, write_i32, write_u16, write_u32,
}; // DB-кодек проекции GameSave и бинарные примитивы записей состояний.
pub use storage::{
    AppliedState, AppliedStateEntries, CanonicalStateStorage, LegacyStateCodec, StateBatch,
    StateData, StateKey, StateSerialization,
}; // хранилище состояний и его кодек сериализации.
