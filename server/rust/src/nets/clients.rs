//! Исходящий TCP-клиент направлений перенесён в Shared network.
//! Здесь реэкспорт для переходных потребителей всех направлений.

pub(crate) use nebokrai_shared::network::{
    connect, ClientConnectError, ClientSendError, ClientSendQueue, FlushOutcome,
    INITIAL_RECEIVE_CAPACITY,
};
