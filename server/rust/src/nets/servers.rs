//! Владелец входящих TCP-соединений CServer перенесён в Shared network.
//! Здесь реэкспорт для переходных потребителей всех направлений.

pub(crate) use nebokrai_shared::network::{
    AcceptStart, AdmissionOutcome, CServer, ComponentReceiveErrorAction, ServerCommandHandle,
    ServerComponentCallbacks, ServerHostError, ServerIoAction, ServerIoCompletion, ServerSnapshot,
    ServerSnapshotError, ACCEPT_AT_CAPACITY_DELAY, ACCEPT_THREAD_DELAY, NET_THREAD_DELAY,
};
