//! FIFO загруженных игроков WorldServer перенесена в Realm characters.
//! Здесь её реэкспорт для переходных потребителей.

// Generic-типу нужен конкретный игрок старого владельца: единственная
// alias-строка, документированное расширение shim-паттерна.
pub(crate) type CPlayerDataQueue =
    nebokrai_realm::characters::playerdataqueue::CPlayerDataQueue<
        crate::worldserver::appworld::player::CPlayer,
    >;
