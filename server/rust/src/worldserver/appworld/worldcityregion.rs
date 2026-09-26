//! Переиздание Realm `regions/worldcityregion.rs` для предков старого
//! `appworld/worldcityregion.rs` на время миграции.

#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::regions::worldcityregion::*;
