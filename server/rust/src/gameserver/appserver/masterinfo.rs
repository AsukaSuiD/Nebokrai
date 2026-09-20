//! Typed `tagMasterInfo` GameServer.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный owner
//! `appserver/masterinfo.cpp`. Конструктор обнуляет десять последовательных
//! DWORD, а `operator=` копирует ровно те же десять DWORD и возвращает
//! destination. `Default`, `Copy` и обычное присваивание Rust сохраняют этот
//! контракт без ручного цикла. Единственный соседний raw catch был служебным
//! cleanup MSVC vector и не принадлежал наблюдаемой семантике `tagMasterInfo`.

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct MasterInfo {
    pub(crate) master_type: i32,
    pub(crate) master_id: i32,
    pub(crate) master_guild_id: i32,
    pub(crate) master_team_id: i32,
    pub(crate) master_union_id: i32,
    pub(crate) master_country_id: i32,
    pub(crate) permitted_to_kill_player: i32,
    pub(crate) permitted_to_kill_teammate: i32,
    pub(crate) permitted_to_kill_guild_member: i32,
    pub(crate) permitted_to_kill_criminal: i32,
}
