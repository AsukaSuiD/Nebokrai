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
#[allow(
    dead_code,
    reason = "goods-destroy serializer подключён к initial-config до text loader-а"
)]
pub(crate) mod goodsdestructionconfig;
#[allow(
    dead_code,
    reason = "GM-list serializer подключён к initial-config до точных ini loaders/accessors"
)]
pub(crate) mod gmlist;
#[allow(
    dead_code,
    reason = "globe snapshot подключён к initial-config до typed loaders/accessors"
)]
pub(crate) mod globesetup;
#[allow(
    dead_code,
    reason = "GodsBattle serializer подключён к initial-config до loaders/runtime owner-а"
)]
pub(crate) mod godsbattleconf;
pub(crate) mod hitlevelsetup;
#[allow(
    dead_code,
    reason = "honor-eliminate serializer подключён к initial-config до точного loader-а"
)]
pub(crate) mod honorelimilateconfig;
pub(crate) mod incrementshoplist;
#[allow(
    dead_code,
    reason = "LingBao serializer подключён к combined initial-config до точного text loader-а"
)]
pub(crate) mod lingbao;
pub(crate) mod leitingsetup;
#[allow(
    dead_code,
    reason = "log-system serializer подключён к initial-config до text loader-а"
)]
pub(crate) mod logsystem;
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
pub(crate) mod prisonconf;
#[allow(
    dead_code,
    reason = "quest serializer подключён к initial-config до точных ini loaders/runtime-а"
)]
pub(crate) mod questsystem;
#[allow(
    dead_code,
    reason = "region-setup serializer подключён к initial-config до точного loader-а"
)]
pub(crate) mod regionsetup;
#[allow(
    dead_code,
    reason = "region-router serializer подключён через globe initial-config до loader-а"
)]
pub(crate) mod regionrouter;
#[allow(
    dead_code,
    reason = "synthesis serializer подключён к initial-config до точного XML loader-а"
)]
pub(crate) mod synthesis;
pub(crate) mod tradelist;
