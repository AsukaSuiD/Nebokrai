//! Переиздание Realm `app/worldothermessage.rs` для предков старого
//! `appworld/message/othermessage.rs` на время миграции.

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::app::worldothermessage::*;
