//! Данные владельца атаки Zone (`tagMasterInfo` GameServer).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/masterinfo.cpp`. Машинный код конструктора (VA `0x0050A610`)
//! проверен на паре EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB SHA-256 `B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016`:
//! конструктор обнуляет десять последовательных DWORD, а `operator=` (VA
//! `0x0050A640`) копирует ровно те же десять DWORD и возвращает destination.
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
