//! Общие setup-owner-ы исторических серверов Miracle.

#[allow(
    dead_code,
    reason = "fairy-exp serializer подключён к initial-config до точного XML loader-а"
)]
pub(crate) mod cbattlefairyexpconfig;
#[allow(
    dead_code,
    reason = "contribute setup подключён к initial-config до общего setup lifecycle"
)]
pub(crate) mod contributesetup;
pub(crate) mod emotion;
pub(crate) mod hitlevelsetup;
#[allow(
    dead_code,
    reason = "increment-shop serializer подключён к initial-config до LoadItems"
)]
pub(crate) mod incrementshoplist;
pub(crate) mod monsterlist;
#[allow(
    dead_code,
    reason = "new-skill-monster serializer подключён к initial-config до XML loader-а"
)]
pub(crate) mod newskillmonsterlist;
pub(crate) mod playerlist;
#[allow(
    dead_code,
    reason = "PreciousBox serializer подключён к initial-config до точного XML→range materializer-а"
)]
pub(crate) mod preciousboxconf;
#[allow(
    dead_code,
    reason = "PrisonConf подключён к initial-config до общего setup lifecycle"
)]
pub(crate) mod prisonconf;
#[allow(
    dead_code,
    reason = "synthesis serializer подключён к initial-config до точного XML loader-а"
)]
pub(crate) mod synthesis;
#[allow(
    dead_code,
    reason = "trade-list loader подключён к initial-config до общего setup lifecycle"
)]
pub(crate) mod tradelist;
