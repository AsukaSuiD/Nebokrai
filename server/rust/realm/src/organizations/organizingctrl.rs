//! Блоки и lookup-типы `COrganizingCtrl`, вынесенные сюда заранее: сам
//! контроллер остаётся в старом пакете до шага переноса
//! organizing-области.
//!
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.

/// Результат `COrganizingCtrl::is_free_player`: свободный игрок, член
/// фракции либо ячейка с null-указателем фракции.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FreePlayerLookup {
    NoFaction,
    Faction(i32),
    BlockedNullFaction { map_key: i32 },
}
