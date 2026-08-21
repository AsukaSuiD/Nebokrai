//! Общий рейтинг игроков исторического WorldServer.
//!
//! Статус World `CPlayerRanks::AddToByteArray` RVA `0x0001B760` и
//! `UpdateRanksToGameServer` RVA `0x0001B890`, `StatPlayerRanks` RVA
//! `0x0001C0D0`, `AddRank` RVA `0x0001C1A0`, `OnStatRanks` RVA
//! `0x0001C370` и `Initialize` RVA `0x0001C450`: `IMPLEMENTED`. Точная пара:
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
//!
//! `Initialize` копирует время и signed maximum из `COrganizingParam`, заменяет
//! в календарной копии только текущие year/month/day и переносит событие на
//! сутки лишь при strict `< now`. `OnStatRanks` после stat/publication снова
//! берёт исходное время, подставляет текущую дату и уже безусловно добавляет
//! сутки. Эти три присваивания исправляют неточность decompiler-а и подтверждены
//! инструкциями `0x0041C4DD..0x0041C4E9` и `0x0041C3EE..0x0041C40C`.
//! `Option<TimerId>` заменяет неинициализированный constructor-ом event ID;
//! сам `CTimer` остаётся библиотечным ordered owner-ом.
//! `Default` повторяет подтверждённую inline-инициализацию singleton-а, а
//! явная передача единственного `CPlayerRanks` заменяет process-global
//! `getInstance`/`GetPlayerRanks`. `Vec` освобождается обычным Rust `Drop`;
//! исходный dangling singleton после `Release` не воспроизводится. Сам
//! `Release` остаётся сырой границей до прямой связи с timer-owner-ом в
//! shutdown-пути `CGame`.
//! Два входных поля теперь приходят из восстановленного `COrganizingParam`;
//! PlayerRanks по-прежнему принимает узкую typed-проекцию и не дублирует его
//! позиционный parser.

use std::error::Error;
use std::fmt;

use crate::nets::networld::message::{CMessage, SendMessageError};
use crate::nets::servers::ServerCommandHandle;
use crate::public::date::{TagTime, TagTimeArithmeticBlock};
use crate::public::timer::{CTimer, TimerId};
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

#[derive(Clone, Debug, Default)]
pub(crate) struct CPlayerRanks {
    stat: bool,
    stat_time: TagTime,
    stat_event_id: Option<TimerId>,
    maximum_count: Option<i32>,
    ranks: Vec<PlayerRankEntry>,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PlayerRanksInitializationConfig {
    pub(crate) stat_time: TagTime,
    pub(crate) maximum_count: i32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct PlayerRanksInitializationReport {
    pub(crate) scheduled_time: TagTime,
    pub(crate) event_id: TimerId,
    pub(crate) legacy_result: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PlayerRanksScheduleBlock {
    DateArithmetic(TagTimeArithmeticBlock),
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct PlayerRanksGameServerUpdate {
    pub(crate) rank_count: usize,
    pub(crate) payload_length: usize,
    pub(crate) delivery: Result<i32, SendMessageError>,
}

impl CPlayerRanks {
    /// Копирует параметры и ставит первое календарное событие exact owner-а.
    pub(crate) fn initialize<Callback: Copy>(
        &mut self,
        configuration: PlayerRanksInitializationConfig,
        current_time: TagTime,
        timer: &mut CTimer<Callback>,
        callback: Callback,
    ) -> Result<PlayerRanksInitializationReport, PlayerRanksScheduleBlock> {
        self.maximum_count = Some(configuration.maximum_count);
        self.stat_time = configuration.stat_time;

        let mut scheduled_time = self.stat_time_for_current_date(current_time);
        if scheduled_time.legacy_lt(current_time) {
            scheduled_time
                .add_day(1)
                .map_err(PlayerRanksScheduleBlock::DateArithmetic)?;
        }

        self.stat = false;
        let event_id = timer.set_time_event(scheduled_time, callback, 0);
        self.stat_event_id = Some(event_id);
        Ok(PlayerRanksInitializationReport {
            scheduled_time,
            event_id,
            legacy_result: true,
        })
    }

    /// Готовит следующее событие после синхронного `OnStatRanks` callback-а.
    pub(crate) fn next_stat_time(
        &self,
        current_time: TagTime,
    ) -> Result<TagTime, PlayerRanksScheduleBlock> {
        let mut scheduled_time = self.stat_time_for_current_date(current_time);
        scheduled_time
            .add_day(1)
            .map_err(PlayerRanksScheduleBlock::DateArithmetic)?;
        Ok(scheduled_time)
    }

    /// Фиксирует side effects после exact `SetTimeEvent` следующего дня.
    pub(crate) fn finish_stat_schedule(&mut self, event_id: TimerId) {
        self.stat_event_id = Some(event_id);
        self.stat = true;
    }

    pub(crate) const fn stat_enabled(&self) -> bool {
        self.stat
    }

    pub(crate) const fn stat_time(&self) -> TagTime {
        self.stat_time
    }

    pub(crate) const fn stat_event_id(&self) -> Option<TimerId> {
        self.stat_event_id
    }

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

    fn stat_time_for_current_date(&self, current_time: TagTime) -> TagTime {
        let mut scheduled_time = self.stat_time;
        scheduled_time.year = current_time.year;
        scheduled_time.month = current_time.month;
        scheduled_time.day = current_time.day;
        scheduled_time
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
// STATUS: IMPLEMENTED
// Rust `Drop` для `Vec<PlayerRankEntry>` выполняет тот же полный release списка.
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
// STATUS: IMPLEMENTED
// `CPlayerRanks::default` и явный owner заменяют nullable process-global singleton.
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
// STATUS: IMPLEMENTED
// Все callers получают тот же один owner явным mutable/shared borrow-ом.
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
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp:144
// RVA: 0x0001C370
// ADDRESS: 0041c370
// PROTOTYPE: void __stdcall OnStatRanks(long param_1)
//
// Реализовано async timer-adapter-ом в `CGame::run_main_loop_timer_stage`;
// календарная часть и state mutation находятся в этом owner-е выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayerRanks::Initialize
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\worldserver\playerranks.cpp:43
// RVA: 0x0001C450
// ADDRESS: 0041c450
// PROTOTYPE: bool __thiscall Initialize(void)
//
// Реализовано выше через `PlayerRanksInitializationConfig` и готовый `CTimer`.
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
