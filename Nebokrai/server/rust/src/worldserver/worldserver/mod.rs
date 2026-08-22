//! Владельцы внутреннего каталога исторического WorldServer.

pub(crate) mod game;
pub(crate) mod honorranks;
pub(crate) mod jjcmaintenanceworker;
#[allow(
    dead_code,
    reason = "LeiTing DB worker готов для concrete LeiTingContext runtime-owner"
)]
pub(crate) mod leitingresetworker;
#[allow(
    dead_code,
    reason = "reconnect-worker готов для runtime-owner полного CGame lifecycle"
)]
pub(crate) mod loginreconnectworker;
#[allow(
    dead_code,
    reason = "PlayerRanks serializer подключён к initial-config до DB/timer lifecycle"
)]
pub(crate) mod playerranks;
#[allow(
    dead_code,
    reason = "concrete player-load thread pool готов для runtime-owner полного CGame lifecycle"
)]
pub(crate) mod playerloadworker;
pub(crate) mod savedb;
pub(crate) mod worldserver;
#[allow(
    dead_code,
    reason = "concrete write-log worker подключён к CGame Init/Release context-границам"
)]
pub(crate) mod writelogworker;
