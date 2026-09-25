//! Мировая запись персонажа и его жизненный цикл Realm.

pub mod honordb; // DB-типы почётных рангов.
pub mod honorranks; // почётные ранги CHonorRanks и генератор их DB-копий.
pub mod monster; // монстр CMonster.
pub mod npc; // NPC CNpc.
pub mod player; // игрок CPlayer: identity, region/session и контейнеры.
pub mod playerdataqueue; // FIFO загруженных игроков.
pub mod playerexploit; // снимок обновления exploit игрока.
pub mod playerloadqueue; // FIFO запросов загрузки игроков.
pub mod playerloadworker; // контракт и пул фоновых DB worker-ов загрузки игроков.
pub mod playerranks; // рейтинг игроков CPlayerRanks.
