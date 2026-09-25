//! Владельцы внутреннего каталога исторического WorldServer.

pub(crate) mod game;
pub(crate) mod honorranks;
pub(crate) mod jjcmaintenanceworker;
pub(crate) mod leitingresetworker;
pub(crate) mod loginreconnectworker;
pub(crate) mod playerranks;
pub(crate) mod playerloadworker;
pub(crate) mod savedb;
#[allow(
    dead_code,
    reason = "process-global World owners подключаются к полному lifecycle по мере сборки runtime"
)]
pub(crate) mod runtime;
pub(crate) mod worldserver;
pub(crate) mod writelogworker;
