//! Состояние принятого TCP-соединения CServerClient перенесено в Shared network.
//! Здесь реэкспорт для переходных потребителей всех направлений.

pub(crate) use nebokrai_shared::network::{
    AddSendDataOutcome, CServerClient, ServerClientMessageContext, ServerClientSizeError,
    ServerSendBatch, ServerSendCompletion, DEFAULT_PERMITTED_SEND_BYTES,
};
