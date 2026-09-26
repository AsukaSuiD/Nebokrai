//! Координация сохранения и DB-инфраструктура World Realm.

pub mod dbmisc; // DB-владелец аукционных очередей World.
pub mod largess; // CLargess: DB-владелец подарков, его трейт, Tiberius-реализация и worker.
pub mod row; // чтение именованных полей SQL-строки через Tiberius.
pub mod rsgenvar; // CRsGenVar: DB-владелец общих переменных мира.
pub mod rsplayer; // CRsPlayer: DB-владелец игрока, его трейт, Tiberius-реализация и адаптер загрузки.
pub mod rssetup; // инициализация World DB.
pub mod savedata; // tagDBData: data/handle-типы save-batch одного DoSaveData.
pub mod savedb; // оркестрация сохранения мира.
pub mod saveworker; // worker-вход SaveThreadFunc и RAII/trigger seam сохранения.
pub mod writelog; // payload-контракт FIFO журнала World.
pub mod writelogqueue; // FIFO команд World write-log.
pub mod writelogworker; // worker журнала с явным владением.
