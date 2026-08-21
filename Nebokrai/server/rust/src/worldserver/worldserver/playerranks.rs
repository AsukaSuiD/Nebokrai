//! Общий рейтинг игроков исторического WorldServer.
//!
//! Статус World `CPlayerRanks::AddToByteArray` RVA `0x0001B760` и
//! `UpdateRanksToGameServer` RVA `0x0001B890`, `StatPlayerRanks` RVA
//! `0x0001C0D0` и `AddRank` RVA `0x0001C1A0`: `IMPLEMENTED`; timer/init
//! lifecycle ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный owner PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp:75`.
//!
//! Exact World serializer и Game decoder подтверждают wire: signed count и
//! insertion-order records `i32 player_id + C-string name + u16 occupation +
//! u16 level + C-string faction_name`. `Vec` заменяет `std::list`, owned bytes
//! — `std::string`; порядок и little-endian поля не меняются. Внутренний NUL
//! штатно завершает исходный C-string и поэтому обрезает только wire-поле.
//! Невозможный 32-битный count блокирует весь append до изменения destination.
//!
//! Публикация использует готовые `CMessage` и `ServerCommandHandle`: exact
//! `0x7F801 + subtype 0x17 + serialized ranks` сохраняется, а самописные
//! буфер, CRC-envelope и fan-out не дублируются. Исходный `SendAll` игнорировал
//! transport-result; Rust оставляет его в отчёте, не меняя порядок вызовов.
//! `StatPlayerRanks` связан в `CGame` с реальным `CRsPlayer`: список очищается
//! до start-log, tick снимается после него, DB добавляет live prefix, затем
//! всегда идут end-log и публикация даже после DB `false`. `AddRank` получает
//! organizing явно вместо singleton-а и сохраняет empty-name ветви отсутствия
//! faction; исходный null внутри map остаётся typed-блоком.

use std::error::Error;
use std::fmt;

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::nets::servers::ServerCommandHandle;
use crate::worldserver::appworld::organizingsystem::organizingctrl::{
    COrganizingCtrl, FreePlayerLookup,
};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PlayerRankEntry {
    pub(crate) player_id: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) occupation: u16,
    pub(crate) level: u16,
    pub(crate) faction_name: Vec<u8>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CPlayerRanks {
    maximum_count: Option<i32>,
    ranks: Vec<PlayerRankEntry>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct PlayerRanksGameServerUpdate {
    pub(crate) rank_count: usize,
    pub(crate) payload_length: usize,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

impl CPlayerRanks {
    pub(crate) const fn maximum_count(&self) -> Option<i32> {
        self.maximum_count
    }

    pub(crate) fn set_maximum_count(&mut self, maximum_count: i32) {
        self.maximum_count = Some(maximum_count);
    }

    pub(crate) fn clear(&mut self) {
        self.ranks.clear();
    }

    pub(crate) fn push(&mut self, rank: PlayerRankEntry) {
        self.ranks.push(rank);
    }

    /// Добавляет строку DB и вычисляет faction-name по live organizing map.
    pub(crate) fn add_rank(
        &mut self,
        organizing: &COrganizingCtrl,
        player_id: i32,
        name: Vec<u8>,
        occupation: u16,
        level: u16,
    ) -> Result<(), PlayerRankAddBlock> {
        let faction_name = match organizing.is_free_player(player_id) {
            FreePlayerLookup::NoFaction => Vec::new(),
            FreePlayerLookup::Faction(faction_id) => organizing
                .faction_by_id(faction_id)
                .map_or_else(Vec::new, |faction| faction.name().to_vec()),
            FreePlayerLookup::BlockedNullFaction { map_key } => {
                return Err(PlayerRankAddBlock::NullFaction { map_key });
            }
        };
        self.ranks.push(PlayerRankEntry {
            player_id,
            name,
            occupation,
            level,
            faction_name,
        });
        Ok(())
    }

    pub(crate) fn ranks(&self) -> &[PlayerRankEntry] {
        &self.ranks
    }

    pub(crate) fn add_to_byte_array(
        &self,
        destination: &mut Vec<u8>,
    ) -> Result<(), PlayerRanksSerializationBlock> {
        let count = i32::try_from(self.ranks.len()).map_err(|_| {
            PlayerRanksSerializationBlock::CountOutOfRange {
                count: self.ranks.len(),
            }
        })?;
        let mut payload = Vec::new();
        payload.extend_from_slice(&count.to_le_bytes());
        for rank in &self.ranks {
            payload.extend_from_slice(&rank.player_id.to_le_bytes());
            write_player_rank_string(&mut payload, &rank.name);
            payload.extend_from_slice(&rank.occupation.to_le_bytes());
            payload.extend_from_slice(&rank.level.to_le_bytes());
            write_player_rank_string(&mut payload, &rank.faction_name);
        }
        destination.extend_from_slice(&payload);
        Ok(())
    }

    /// Публикует exact `0x7F801/0x17` всем подключённым GameServer-ам.
    pub(crate) fn update_ranks_to_game_server(
        &self,
        sender: Option<&ServerCommandHandle>,
    ) -> Result<PlayerRanksGameServerUpdate, PlayerRanksSerializationBlock> {
        let mut payload = Vec::new();
        self.add_to_byte_array(&mut payload)?;

        let mut message = CMessage::new(0x0007_F801);
        message.base_mut().add_long(0x17);
        message.base_mut().add(&payload);
        Ok(PlayerRanksGameServerUpdate {
            rank_count: self.ranks.len(),
            payload_length: payload.len(),
            delivery: message.send_all(sender),
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PlayerRanksSerializationBlock {
    CountOutOfRange {
        count: usize,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerRankAddBlock {
    NullFaction { map_key: i32 },
}

impl fmt::Display for PlayerRanksSerializationBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CountOutOfRange { count } => write!(
                formatter,
                "PlayerRanks содержит {count} записей вне signed 32-битного диапазона"
            ),
        }
    }
}

impl Error for PlayerRanksSerializationBlock {}

fn write_player_rank_string(destination: &mut Vec<u8>, value: &[u8]) {
    let prefix = value.split(|byte| *byte == 0).next().unwrap_or_default();
    destination.extend_from_slice(prefix);
    destination.push(0);
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp


// ============================================================================
// FUNCTION: CHonorRanks::tagDBData::tagDBData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp
// RVA: 0x0001A9B0
// ADDRESS: 0041a9b0
// PROTOTYPE: undefined __thiscall tagDBData(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0041ab25
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp
// RVA: 0x0001AB25
// ADDRESS: 0041ab25
// PROTOTYPE: undefined Catch@0041ab25()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerRanks::AddToByteArray
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp:75
// RVA: 0x0001B760
// ADDRESS: 0041b760
// PROTOTYPE: bool __thiscall AddToByteArray(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1)
//
// Реализовано выше через owned `Vec<u8>` с exact insertion-order wire.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerRanks::UpdateRanksToGameServer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp:134
// RVA: 0x0001B890
// ADDRESS: 0041b890
// PROTOTYPE: void __thiscall UpdateRanksToGameServer(void)
//
// Реализовано выше через готовые World `CMessage` и server fan-out.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerRanks::~CPlayerRanks
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp:28
// RVA: 0x0001BF30
// ADDRESS: 0041bf30
// PROTOTYPE: void __thiscall ~CPlayerRanks(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerRanks::getInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp:33
// RVA: 0x0001BF50
// ADDRESS: 0041bf50
// PROTOTYPE: CPlayerRanks * __cdecl getInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: GetPlayerRanks
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp:166
// RVA: 0x0001BFD0
// ADDRESS: 0041bfd0
// PROTOTYPE: CPlayerRanks * __cdecl GetPlayerRanks(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerRanks::Release
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp:63
// RVA: 0x0001C090
// ADDRESS: 0041c090
// PROTOTYPE: void __thiscall Release(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerRanks::StatPlayerRanks
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp:109
// RVA: 0x0001C0D0
// ADDRESS: 0041c0d0
// PROTOTYPE: void __thiscall StatPlayerRanks(void)
//
// Реализовано связанным проходом в `CGame::run_main_loop_maintenance_stage`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerRanks::AddRank
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp:121
// RVA: 0x0001C1A0
// ADDRESS: 0041c1a0
// PROTOTYPE: void __thiscall AddRank(int param_1, basic_string<char,std::char_traits<char>,std::allocator<char>_> * param_2, ushort param_3, ushort param_4)
//
// Реализовано выше через явный `COrganizingCtrl` и insertion-order `Vec`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerRanks::OnStatRanks
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp:144
// RVA: 0x0001C370
// ADDRESS: 0041c370
// PROTOTYPE: void __stdcall OnStatRanks(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerRanks::Initialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp:43
// RVA: 0x0001C450
// ADDRESS: 0041c450
// PROTOTYPE: bool __thiscall Initialize(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//








// ============================================================================
// FUNCTION: FUN_0053bba0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp
// RVA: 0x0013BBA0
// ADDRESS: 0053bba0
// PROTOTYPE: undefined FUN_0053bba0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0053bbb0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp
// RVA: 0x0013BBB0
// ADDRESS: 0053bbb0
// PROTOTYPE: undefined FUN_0053bbb0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0053bbd0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp
// RVA: 0x0013BBD0
// ADDRESS: 0053bbd0
// PROTOTYPE: undefined FUN_0053bbd0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_0053bbf0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp
// RVA: 0x0013BBF0
// ADDRESS: 0053bbf0
// PROTOTYPE: undefined FUN_0053bbf0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
