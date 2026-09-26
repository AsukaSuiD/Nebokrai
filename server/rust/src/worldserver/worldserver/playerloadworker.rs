//! Жизненный цикл пула загрузки игроков перенесён в Realm characters.
//! Здесь его реэкспорт для переходных потребителей.

#[allow(unused_imports, reason = "потребитель (process owner) перенесён в Realm волной C5-D; shim умирает с пакетом")]
pub(crate) use nebokrai_realm::characters::playerloadworker::*;

// Generic spec получает конкретного игрока старого владельца: локальный alias
// перекрывает glob-реэкспорт этого имени, потребители видят готовый тип.
pub(crate) type WorldPlayerLoadWorkerSpec =
    nebokrai_realm::characters::playerloadworker::WorldPlayerLoadWorkerSpec<
        crate::worldserver::appworld::player::CPlayer,
    >;
