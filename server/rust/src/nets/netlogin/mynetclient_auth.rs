//! Исходящее AuthServer-направление Login перенесено в Realm app.
//! Здесь реэкспорт для переходных потребителей.

pub(crate) use nebokrai_realm::app::login_auth_client::{
    AuthClientEvent, AuthClientEventPublisher, AuthClientIoError, AuthClientIoStep,
    CMyNetClientAuth,
};
