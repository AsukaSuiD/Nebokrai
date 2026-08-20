//! Общие setup-owner-ы исторических серверов Miracle.

pub(crate) mod emotion;
pub(crate) mod hitlevelsetup;
#[allow(
    dead_code,
    reason = "increment-shop serializer подключён к initial-config до LoadItems"
)]
pub(crate) mod incrementshoplist;
pub(crate) mod monsterlist;
pub(crate) mod playerlist;
#[allow(
    dead_code,
    reason = "trade-list loader подключён к initial-config до общего setup lifecycle"
)]
pub(crate) mod tradelist;
