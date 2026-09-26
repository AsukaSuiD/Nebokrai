//! Мировые рейтинги Realm: рейтинг игроков, почётные ранги, их DB-типы и
//! honor-eliminator индекс.

pub mod honordb; // DB-типы почётных рангов.
pub mod honoreliminators; // honor-eliminator индекс мира: убитый игрок → его убийцы (ветви 0x5FD0C/0x5FD0D).
pub mod honorranks; // почётные ранги CHonorRanks и генератор их DB-копий.
pub mod playerranks; // рейтинг игроков CPlayerRanks.
