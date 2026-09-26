//! Данные владельца атаки Zone (`tagMasterInfo` GameServer).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/masterinfo.cpp`; идентификаторы точной пары —
//! docs/reconstruction/gameserver-npc-and-regions.md#идентификаторы-сборки.
//! Машинный код конструктора (VA `0x0050A610`) подтверждает: конструктор
//! обнуляет десять последовательных DWORD, а `operator=` (VA `0x0050A640`)
//! копирует ровно те же десять DWORD и возвращает destination.
//! `Default`, `Copy` и обычное присваивание Rust сохраняют этот
//! контракт без ручного цикла. Единственный соседний raw catch был служебным
//! cleanup MSVC vector и не принадлежал наблюдаемой семантике `tagMasterInfo`.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct MasterInfo {
    pub master_type: i32,
    pub master_id: i32,
    pub master_guild_id: i32,
    pub master_team_id: i32,
    pub master_union_id: i32,
    pub master_country_id: i32,
    pub permitted_to_kill_player: i32,
    pub permitted_to_kill_teammate: i32,
    pub permitted_to_kill_guild_member: i32,
    pub permitted_to_kill_criminal: i32,
}
