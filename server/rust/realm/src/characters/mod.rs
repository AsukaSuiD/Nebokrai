//! Мировая запись персонажа и его жизненный цикл Realm.

pub mod monster; // монстр CMonster.
pub mod npc; // NPC CNpc.
pub mod player; // игрок CPlayer: identity, region/session и контейнеры.
pub mod playerdataqueue; // FIFO загруженных игроков.
pub mod playerexploit; // снимок обновления exploit игрока.
pub mod playerloadqueue; // FIFO запросов загрузки игроков.
pub mod playerloadworker; // контракт и пул фоновых DB worker-ов загрузки игроков.
pub mod worldplayers; // мировой реестр игроков и присутствие (login/online/offline/creation/restore/deletion): primary state и typed-операции владельца.
