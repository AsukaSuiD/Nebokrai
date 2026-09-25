//! Клиентские контракты состояний Zone: проекция живых записей для
//! `CMoveShape::AddToByteArray_ForClient` и runtime-план visual после Begin.
//! Каталог и сварочный трейт — в `catalog`.

mod catalog;

pub use catalog::{
    StateClientPayload, StateClientRecord, StatePayloadView,
    registered_runtime_state_visual, state_client_record,
};
