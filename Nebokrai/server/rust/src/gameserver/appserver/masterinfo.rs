//! Typed `tagMasterInfo` GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/masterinfo.cpp`. Десять последовательных DWORD сохраняются
//! буквально; Rust assignment заменяет native копирующий цикл.

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

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\masterinfo.cpp

// ============================================================================
// FUNCTION: Catch@004e8206
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\masterinfo.cpp
// RVA: 0x000E8206
// ADDRESS: 004e8206
// PROTOTYPE: undefined Catch@004e8206()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagMasterInfo::tagMasterInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\masterinfo.cpp:11
// RVA: 0x0010A610
// ADDRESS: 0050a610
// PROTOTYPE: undefined __thiscall tagMasterInfo(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: tagMasterInfo::operator=
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\masterinfo.cpp:16
// RVA: 0x0010A640
// ADDRESS: 0050a640
// PROTOTYPE: tagMasterInfo * __thiscall operator=(tagMasterInfo * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
