//! Переиздание Realm `regions/worldregion.rs` для предков старого
//! `appworld/worldregion.rs` на время миграции. Resource-контекст живёт
//! рядом с reload-владельцем в Realm app.

pub(crate) use nebokrai_realm::app::worldserver::WorldRegionResourceContext;
#[allow(unused_imports, reason = "потребитель перенесён в Realm волной C5-C; shim умирает с пакетом в C5-D")]
pub(crate) use nebokrai_realm::regions::worldregion::*;
