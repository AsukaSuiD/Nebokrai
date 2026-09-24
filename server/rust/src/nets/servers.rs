//! Владелец входящих TCP-соединений CServer перенесён в Shared network.
//! Здесь реэкспорт для переходных потребителей всех направлений.

pub(crate) use nebokrai_shared::network::{
    AcceptStart, AdmissionOutcome, ServerCommandHandle, ServerHostError, ServerIoAction,
    ServerIoCompletion, ServerSnapshotError, ACCEPT_AT_CAPACITY_DELAY, ACCEPT_THREAD_DELAY,
    NET_THREAD_DELAY,
};
