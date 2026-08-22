//! Владельцы внутреннего каталога исторического WorldServer.

pub(crate) mod game;
pub(crate) mod honorranks;
#[allow(
    dead_code,
    reason = "PlayerRanks serializer подключён к initial-config до DB/timer lifecycle"
)]
pub(crate) mod playerranks;
pub(crate) mod savedb;
pub(crate) mod worldserver;
#[allow(
    dead_code,
    reason = "typed DB/batch consumer готов; внешний thread/reconnect lifecycle ещё RAW"
)]
pub(crate) mod writelogworker;
