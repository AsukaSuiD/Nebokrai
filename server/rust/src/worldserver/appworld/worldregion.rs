//! Переиздание Realm `regions/worldregion.rs` для предков старого
//! `appworld/worldregion.rs` на время миграции. Resource-контекст живёт
//! рядом с reload-владельцем в Realm app.

pub(crate) use nebokrai_realm::app::worldserver::WorldRegionResourceContext;
pub(crate) use nebokrai_realm::regions::worldregion::*;
