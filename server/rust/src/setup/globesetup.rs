//! Общий глобальный setup `CGlobeSetup` перенесён в Shared resources.
//! Здесь реэкспорт для переходных потребителей обеих ролей.
//!
//! World-адаптер JJC (`GlobeSetupJjcWorldConfig`) переехал в Realm
//! `activities::jjcsystem` волной C5-C вместе с его потребителями.

pub(crate) use nebokrai_shared::resources::{
    GlobePlayerPropertyCoefficients, GlobeSetupDecodeError, GlobeSetupSnapshot,
    GlobeStiffenSetup,
};
