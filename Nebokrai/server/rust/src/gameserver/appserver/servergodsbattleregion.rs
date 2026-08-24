//! Статус корпуса: MIXED (`CGodsBattleMgr` startup snapshot реализован,
//! остальной owner сохранён как RAW pseudocode).
//! Декомпилятор: Ghidra 12.1.2
//! Сырой C++ ниже после typed owner-а является комментарием, а не
//! Rust-реализацией.
//!
//! Startup snapshot сохраняет exact wire, section-local clear, намеренное
//! append-поведение faction rules и обе внутренние audit-записи. Region-set
//! хранит ordered unique ID, а concrete startup region делегирует
//! подтверждённому `CServerWarRegion` wire-owner-у.
//! Безразмерный pointer и 256-байтный временный C-string buffer заменены
//! bounded slice/cursor и owned bytes; обрыв возвращает typed error после уже
//! завершённого prefix-а вместо неназначаемого legacy UB. Gameplay lifecycle
//! и остальные методы manager-а пока остаются RAW ниже. Top-ten SZL exchange
//! хранит единственный overwrite-able requester, exact World request и
//! terminal-marker decoder; client publication выполняет dispatcher-owner.
//! XYD round-trip использует configuration-owned slots, ordered region set и
//! faction player sets; полные NPC/contend ветви `AddObject/RemoveObject`
//! остаются RAW, а их player membership tail исполняет `CGame`.
//! SZL owner дополнительно материализует inclusive tier lookup и обе
//! victim-tier gain/loss формулы; death/team/client ordering остаётся у `CGame`.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.h

use crate::setup::godsbattleconf::{
    CGodsBattleConf, GodsBattleDecodeError, GodsBattleDecodeReport, GodsBattleFactionXydUpdate,
    GodsBattleSzlCalculation,
};
use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

use super::serverregion::ServerRegionDecodeError;
use super::serverwarregion::{CServerWarRegion, WarRegionDecodeContext, WarRegionDecodeError};

// Точные GBK payload из GameServer .rdata VA `0x00651870` и `0x00651850`.
const REVISE_MONEY_CONFIGURATION_ERROR: &[u8] =
    b"\xC9\xF1\xD6\xAE\xC1\xA6\xD0\xDE\xD5\xFD\xD6\xB5\xC5\xE4\xD6\xC3\xB4\xED\xCE\xF3\xA3\xA1";
const EMPTY_DIE_BACK_CONFIGURATION: &[u8] =
    b"\xA1\xBE\xD6\xEE\xC9\xF1\xD6\xAE\xD5\xBD\xA1\xBF\xCB\xC0\xCD\xF6\xBB\xD8\xB3\xC7\xB5\xC4\xC5\xE4\xD6\xC3\xCE\xAA\xBF\xD5";

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CGodsBattleMgr {
    configuration: CGodsBattleConf,
    region_set: BTreeSet<i32>,
    pending_top_ten_player_id: i32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GodsBattleTopTenEntry {
    pub(crate) faction: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) szl: u32,
    pub(crate) level: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum GodsBattleTopTenDecodeError {
    UnexpectedEnd { offset: usize, field: &'static str },
    MissingNameTerminator { offset: usize },
    NameOutsideLegacyBuffer { offset: usize, length: usize },
}

impl fmt::Display for GodsBattleTopTenDecodeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnexpectedEnd { offset, field } => {
                write!(
                    formatter,
                    "GodsBattle top-ten обрывается на {offset} в поле {field}"
                )
            }
            Self::MissingNameTerminator { offset } => write!(
                formatter,
                "GodsBattle top-ten name с {offset} не имеет NUL-терминатора"
            ),
            Self::NameOutsideLegacyBuffer { offset, length } => write!(
                formatter,
                "GodsBattle top-ten name с {offset} длиной {length} не помещается в 260 байт"
            ),
        }
    }
}

impl Error for GodsBattleTopTenDecodeError {}

impl CGodsBattleMgr {
    pub(crate) const fn configuration(&self) -> &CGodsBattleConf {
        &self.configuration
    }

    pub(crate) fn add_region_set(&mut self, region_id: i32) -> bool {
        self.region_set.insert(region_id)
    }

    pub(crate) fn contains_region(&self, region_id: i32) -> bool {
        self.region_set.contains(&region_id)
    }

    pub(crate) fn region_ids(&self) -> Vec<i32> {
        self.region_set.iter().copied().collect()
    }

    pub(crate) const fn pending_top_ten_player_id(&self) -> i32 {
        self.pending_top_ten_player_id
    }

    /// Exact post-send assignment `GetTopTenSZL`: concurrent request
    /// перезаписывает единственный legacy requester без sequence ID.
    pub(crate) const fn record_top_ten_request(&mut self, player_id: i32) {
        self.pending_top_ten_player_id = player_id;
    }

    pub(crate) fn faction_for_country(&self, country: u8) -> Option<i32> {
        self.configuration
            .faction_for_country(country)
            .map(|faction| faction as i32)
    }

    pub(crate) fn set_xyd(
        &mut self,
        faction_a: u32,
        faction_b: u32,
    ) -> [GodsBattleFactionXydUpdate; 2] {
        [
            self.configuration.set_faction_xyd(1, faction_a),
            self.configuration.set_faction_xyd(2, faction_b),
        ]
    }

    pub(crate) fn calculate_szl_gain(
        &self,
        killer_level: u8,
        killer_szl: u32,
        victim_level: u8,
        victim_szl: u32,
    ) -> GodsBattleSzlCalculation {
        self.configuration
            .calculate_szl_gain(killer_level, killer_szl, victim_level, victim_szl)
    }

    pub(crate) fn calculate_szl_loss(
        &self,
        killer_level: u8,
        killer_szl: u32,
        victim_level: u8,
        victim_szl: u32,
    ) -> GodsBattleSzlCalculation {
        self.configuration
            .calculate_szl_loss(killer_level, killer_szl, victim_level, victim_szl)
    }

    pub(crate) fn szl_level(&self, szl: u32) -> Option<u32> {
        self.configuration.szl_level(szl)
    }

    pub(crate) fn decode_top_ten(
        &self,
        source: &[u8],
        cursor: &mut usize,
    ) -> Result<Vec<GodsBattleTopTenEntry>, GodsBattleTopTenDecodeError> {
        let mut entries = Vec::new();
        loop {
            let marker = read_top_ten_i32(source, cursor, "marker")?;
            if marker == 0 {
                return Ok(entries);
            }
            let faction = read_top_ten_i32(source, cursor, "faction")?;
            let name_offset = *cursor;
            let remaining =
                source
                    .get(name_offset..)
                    .ok_or(GodsBattleTopTenDecodeError::UnexpectedEnd {
                        offset: name_offset,
                        field: "name",
                    })?;
            let length = remaining.iter().position(|byte| *byte == 0).ok_or(
                GodsBattleTopTenDecodeError::MissingNameTerminator {
                    offset: name_offset,
                },
            )?;
            if length >= 260 {
                return Err(GodsBattleTopTenDecodeError::NameOutsideLegacyBuffer {
                    offset: name_offset,
                    length,
                });
            }
            let name = remaining[..length].to_vec();
            *cursor = cursor.wrapping_add(length + 1);
            let szl = read_top_ten_i32(source, cursor, "szl")? as u32;
            let level = read_top_ten_i32(source, cursor, "level")? as u32;
            entries.push(GodsBattleTopTenEntry {
                faction,
                name,
                szl,
                level,
            });
        }
    }

    /// Воспроизводит `CGodsBattleMgr::DecordFromByteArray`, включая оба
    /// внутренних audit side effect-а в исходных позициях.
    pub(crate) fn decord_from_byte_array<AddLogText, PutStringToFile>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        add_log_text: &mut AddLogText,
        put_string_to_file: &mut PutStringToFile,
    ) -> Result<GodsBattleDecodeReport, GodsBattleDecodeError>
    where
        AddLogText: FnMut(&[u8]),
        PutStringToFile: FnMut(&str, &[u8]),
    {
        self.configuration.decord_from_byte_array(
            source,
            cursor,
            || add_log_text(REVISE_MONEY_CONFIGURATION_ERROR),
            || put_string_to_file("godsbattleLog", EMPTY_DIE_BACK_CONFIGURATION),
        )
    }
}

fn read_top_ten_i32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<i32, GodsBattleTopTenDecodeError> {
    let offset = *cursor;
    let bytes = source
        .get(offset..offset.saturating_add(4))
        .ok_or(GodsBattleTopTenDecodeError::UnexpectedEnd { offset, field })?;
    *cursor = cursor.wrapping_add(4);
    Ok(i32::from_le_bytes(
        bytes
            .try_into()
            .expect("проверенный четырёхбайтовый GodsBattle scalar"),
    ))
}

/// Startup-часть concrete GodsBattle region. Constructor подтверждает
/// наследование `CServerWarRegion`; player faction membership уже связан с
/// Add/Remove tail, NPC/contend gameplay коллекции сохраняют owned defaults.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CServerGodsBattleRegion {
    pub(crate) war: CServerWarRegion,
    faction_players: [BTreeSet<i32>; 3],
    faction_npcs: [BTreeSet<i32>; 3],
}

impl CServerGodsBattleRegion {
    pub(crate) fn decord_from_byte_array<Context: WarRegionDecodeContext>(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        context: &mut Context,
    ) -> Result<bool, WarRegionDecodeError<ServerRegionDecodeError<Context::RuntimeError>>> {
        self.war
            .decord_from_byte_array(source, cursor, include_child, context)
    }

    pub(crate) fn add_faction_player(&mut self, player_id: i32, faction: i32) -> bool {
        match faction {
            5 => self.faction_players[1].insert(player_id),
            6 => self.faction_players[2].insert(player_id),
            _ => false,
        }
    }

    pub(crate) fn remove_faction_player(&mut self, player_id: i32) -> bool {
        self.faction_players
            .iter_mut()
            .fold(false, |removed, players| {
                players.remove(&player_id) || removed
            })
    }

    pub(crate) fn faction_player_ids(&self, faction: i32) -> Option<Vec<i32>> {
        let index = match faction {
            5 => 1,
            6 => 2,
            _ => return None,
        };
        Some(self.faction_players[index].iter().copied().collect())
    }
}

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetFactionXYD
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:932
// RVA: 0x000A5B50
// ADDRESS: 004a5b50
// PROTOTYPE: ulong __thiscall GetFactionXYD(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::IsPlayerContendSymbol
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:438
// RVA: 0x000A5E00
// ADDRESS: 004a5e00
// PROTOTYPE: bool __thiscall IsPlayerContendSymbol(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::IsGodsBattleRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1216
// RVA: 0x000A6200
// ADDRESS: 004a6200
// PROTOTYPE: bool __thiscall IsGodsBattleRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetReturnPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1340
// RVA: 0x000A6230
// ADDRESS: 004a6230
// PROTOTYPE: bool __thiscall GetReturnPoint(long param_1, ulong param_2, long * param_3, long * param_4, long * param_5, long * param_6, long * param_7, long * param_8)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::DelObj
// STATUS: PARTIALLY_IMPLEMENTED_PLAYER_MEMBERSHIP
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:183
// RVA: 0x000A6640
// ADDRESS: 004a6640
// PROTOTYPE: void __thiscall DelObj(int param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::OnEnterContend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:410
// RVA: 0x000A66E0
// ADDRESS: 004a66e0
// PROTOTYPE: void __thiscall OnEnterContend(CPlayer * param_1, long param_2, char * param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetNpcNameByMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1220
// RVA: 0x000A67B0
// ADDRESS: 004a67b0
// PROTOTYPE: bool __thiscall GetNpcNameByMonster(CMonster * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetAlreadyDieCount
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1320
// RVA: 0x000A6850
// ADDRESS: 004a6850
// PROTOTYPE: ulong __thiscall GetAlreadyDieCount(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::RemoveObject
// STATUS: PARTIALLY_IMPLEMENTED_PLAYER_MEMBERSHIP
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:138
// RVA: 0x000A7270
// ADDRESS: 004a7270
// PROTOTYPE: void __thiscall RemoveObject(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::CancelContendByPlayerID
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:452
// RVA: 0x000A8010
// ADDRESS: 004a8010
// PROTOTYPE: bool __thiscall CancelContendByPlayerID(CPlayer * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::~CServerGodsBattleRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:19
// RVA: 0x000A86B0
// ADDRESS: 004a86b0
// PROTOTYPE: void __thiscall ~CServerGodsBattleRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::CServerGodsBattleRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:14
// RVA: 0x000A90F0
// ADDRESS: 004a90f0
// PROTOTYPE: undefined __thiscall CServerGodsBattleRegion(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::GetObjFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:312
// RVA: 0x000A91B0
// ADDRESS: 004a91b0
// PROTOTYPE: Fation __thiscall GetObjFaction(int param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::AddContend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:361
// RVA: 0x000A9270
// ADDRESS: 004a9270
// PROTOTYPE: void __thiscall AddContend(CPlayer * param_1, long param_2, char * param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::CancelContendBySymbol
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:568
// RVA: 0x000A9590
// ADDRESS: 004a9590
// PROTOTYPE: void __thiscall CancelContendBySymbol(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:592
// RVA: 0x000A9660
// ADDRESS: 004a9660
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004a9779
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:639
// RVA: 0x000A9779
// ADDRESS: 004a9779
// PROTOTYPE: undefined Catch@004a9779()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::StrSplit
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1161
// RVA: 0x000A9C70
// ADDRESS: 004a9c70
// PROTOTYPE: bool __thiscall StrSplit(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, vector<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>,std::allocator<std::basic_string<char,std::char_traits<char>,std::allocator<char>_>_>_> * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetMonsterCountByNpcName
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1237
// RVA: 0x000A9DB0
// ADDRESS: 004a9db0
// PROTOTYPE: ulong __thiscall GetMonsterCountByNpcName(char * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::OnNpcMonsterDie
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1253
// RVA: 0x000A9FC0
// ADDRESS: 004a9fc0
// PROTOTYPE: void __thiscall OnNpcMonsterDie(CMonster * param_1, CBaseObject * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::OnEnterContend
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1078
// RVA: 0x000AA5D0
// ADDRESS: 004aa5d0
// PROTOTYPE: bool __thiscall OnEnterContend(CPlayer * param_1, CNpc * param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::OnNpcUpdate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1186
// RVA: 0x000AA8A0
// ADDRESS: 004aa8a0
// PROTOTYPE: void __thiscall OnNpcUpdate(CNpc * param_1, ulong param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::CGodsBattleMgr
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:893
// RVA: 0x000AAAF0
// ADDRESS: 004aaaf0
// PROTOTYPE: undefined __thiscall CGodsBattleMgr(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:912
// RVA: 0x000AABF0
// ADDRESS: 004aabf0
// PROTOTYPE: CGodsBattleMgr * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::OnNpcSetFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:648
// RVA: 0x000AAC60
// ADDRESS: 004aac60
// PROTOTYPE: void __thiscall OnNpcSetFaction(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::OnMonsterDie
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:698
// RVA: 0x000AB150
// ADDRESS: 004ab150
// PROTOTYPE: void __thiscall OnMonsterDie(CMonster * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::GetReturnPoint
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:715
// RVA: 0x000AB1C0
// ADDRESS: 004ab1c0
// PROTOTYPE: void __thiscall GetReturnPoint(CPlayer * param_1, long * param_2, long * param_3, long * param_4, long * param_5, long * param_6, long * param_7)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CGodsBattleMgr::RefreshMonsterForNpc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:1150
// RVA: 0x000AB740
// ADDRESS: 004ab740
// PROTOTYPE: bool __thiscall RefreshMonsterForNpc(basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_1, ulong param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::AddObject
// STATUS: PARTIALLY_IMPLEMENTED_PLAYER_MEMBERSHIP
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:24
// RVA: 0x000AB7C0
// ADDRESS: 004ab7c0
// PROTOTYPE: void __thiscall AddObject(CBaseObject * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::OnChangeFaction
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:226
// RVA: 0x000ABC30
// ADDRESS: 004abc30
// PROTOTYPE: bool __thiscall OnChangeFaction(int param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CServerGodsBattleRegion::OnContendTimeOver
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp:476
// RVA: 0x000ABDF0
// ADDRESS: 004abdf0
// PROTOTYPE: void __thiscall OnContendTimeOver(tagContend * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004f3739
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp
// RVA: 0x000F3739
// ADDRESS: 004f3739
// PROTOTYPE: undefined Catch@004f3739()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004f38a6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\servergodsbattleregion.cpp
// RVA: 0x000F38A6
// ADDRESS: 004f38a6
// PROTOTYPE: undefined Catch@004f38a6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
