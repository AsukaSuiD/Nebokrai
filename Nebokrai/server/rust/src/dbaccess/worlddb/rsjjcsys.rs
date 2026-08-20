//! DB-владелец `CRsJJcSys` исторического WorldServer из `rsjjcsys.cpp`.
//!
//! Статус `SaveJJcData` RVA `0x00116990` — `IMPLEMENTED`; остальные функции
//! ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp`.
//!
//! PDB задаёт `tagPlayerJJcData` размером `0x10`: восемь последовательных
//! `unsigned short` от `wWeekJoinCnt` по `+0` до `wSeasonTieCnt` по `+0xE`;
//! сам подобъект лежит в `CPlayer+0x992`. `dwJJcLevel/dwJJcScore` находятся в
//! `tagBaseProperty` по `+0x174/+0x178`, то есть в `CPlayer+0x70C/+0x710`, а
//! inherited signed ID читается по `CPlayer+8`.
//!
//! Функция вызывает `sp_JJccUpdatePlayer` с параметрами `@id`, `@jjcLevel`,
//! `@jjcScore`, четырьмя week- и четырьмя season-счётчиками. Все параметры
//! создавались как `adInteger` размером четыре байта; source `DWORD` передаётся
//! в TDS как `i64`, чтобы SQL Server сохранил прежнюю проверку диапазона при
//! приведении к procedure `int`, а `WORD` расширяется без знака до `i32`.
//! Read-only `GameDB05.bak`, SHA-256
//! `E0C4F191A23B9B61B881157E585E5C9AE71BE5EFCB1DB15410231F29FD297DAD`,
//! содержит точный текст процедуры: все одиннадцать параметров объявлены
//! `int`; сначала выполняется `UPDATE csl_player_jjc`, а при `@@rowcount = 0` —
//! `INSERT` с теми же значениями. Rust вызывает именно эту procedure и не
//! воспроизводит её upsert вручную.
//!
//! В исходнике доказан дефект: `@seasonTie` получает адрес
//! `wWeekTieCnt` (`CPlayer+0x998`), а настоящее `wSeasonTieCnt` по `+0x9A0`
//! вообще не читается. Rust намеренно дублирует `week_tie`; отдельное поле
//! snapshot сохранено как карта PDB-layout, но не отправляется процедуре.
//!
//! Хотя PDB-сигнатура принимает connection по значению, exact
//! `0x00516A2A..0x00516A60` передаёт адрес его локальной копии в `CreateCn`, а
//! тот сначала освобождает прежний COM pointer и создаёт/открывает новое World
//! DB соединение. Поэтому save не входит в caller-транзакцию. Отдельный
//! `tiberius` client сохраняет этот observable boundary; его Drop заменяет COM
//! Release. Exact `0x00516C83..0x00516D12` подтверждает один `Command::Execute`,
//! `AL=1` после успеха и `AL=0` из catch. Catch содержит ошибочный текст
//! `Clear JJc week data failed`; typed notice сохраняет именно эту операцию,
//! не публикуя credentials или player values. ADO Command/Parameters, COM/SEH
//! cleanup заменены закреплённым `tiberius` и безопасным владением Rust.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::io;

use tiberius::{Client, Config, ToSql};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use super::rssetup::WorldDatabaseSettings;

const SAVE_JJC_DATA_SQL: &str = "EXEC sp_JJccUpdatePlayer @id=@P1, @jjcLevel=@P2, @jjcScore=@P3, @weekJoin=@P4, @weekWin=@P5, @weekLose=@P6, @weekTie=@P7, @seasonJoin=@P8, @seasonWin=@P9, @seasonLose=@P10, @seasonTie=@P11";

type JjcTdsClient = Client<Compat<TcpStream>>;

/// Полный PDB-layout `CPlayer::tagPlayerJJcData` вместе с двумя base-полями.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PlayerJjcDataSnapshot {
    pub(crate) id: i32,
    pub(crate) jjc_level: u32,
    pub(crate) jjc_score: u32,
    pub(crate) week_join: u16,
    pub(crate) week_win: u16,
    pub(crate) week_lose: u16,
    pub(crate) week_tie: u16,
    pub(crate) season_join: u16,
    pub(crate) season_win: u16,
    pub(crate) season_lose: u16,
    /// Исходный save не читает это поле и ошибочно повторяет `week_tie`.
    pub(crate) season_tie: u16,
}

/// Структурированная замена catch `Clear JJc week data failed`.
#[derive(Debug)]
pub(crate) struct RsJjcSysNotice {
    pub(crate) error: RsJjcSysDatabaseError,
}

/// Ошибка отдельной connection/procedure-границы без runtime значений.
#[derive(Debug)]
pub(crate) enum RsJjcSysDatabaseError {
    Connect(io::Error),
    Tds(tiberius::error::Error),
}

impl fmt::Display for RsJjcSysDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connect(error) => write!(
                formatter,
                "не установлено отдельное World JJc DB соединение: {error}"
            ),
            Self::Tds(error) => write!(formatter, "ошибка TDS World JJc DB: {error}"),
        }
    }
}

impl Error for RsJjcSysDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Connect(error) => Some(error),
            Self::Tds(error) => Some(error),
        }
    }
}

impl From<tiberius::error::Error> for RsJjcSysDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self::Tds(error)
    }
}

/// Узкая объектная граница достигнутого `CRsJJcSys::SaveJJcData`.
pub(crate) trait RsJjcSysOwner {
    /// Открывает отдельное соединение и выполняет одну исходную procedure.
    async fn save_jjc_data(&mut self, snapshot: &PlayerJjcDataSnapshot) -> bool;

    /// Забирает следующий `Clear JJc week data failed`-эквивалент.
    fn pop_notice(&mut self) -> Option<RsJjcSysNotice>;
}

/// Linux/TDS-замена достигнутой save-части исходного singleton-а.
pub(crate) struct TiberiusRsJjcSys {
    config: Config,
    notices: VecDeque<RsJjcSysNotice>,
}

impl TiberiusRsJjcSys {
    /// Копирует DB config; каждое сохранение всё равно создаёт новый client.
    pub(crate) fn new(settings: &WorldDatabaseSettings) -> Self {
        Self {
            config: settings.tds_config(),
            notices: VecDeque::new(),
        }
    }

    async fn connect(config: Config) -> Result<JjcTdsClient, RsJjcSysDatabaseError> {
        let tcp = TcpStream::connect(config.get_addr())
            .await
            .map_err(RsJjcSysDatabaseError::Connect)?;
        tcp.set_nodelay(true)
            .map_err(RsJjcSysDatabaseError::Connect)?;
        Client::connect(config, tcp.compat_write())
            .await
            .map_err(RsJjcSysDatabaseError::Tds)
    }

    async fn execute_save(
        config: Config,
        snapshot: &PlayerJjcDataSnapshot,
    ) -> Result<(), RsJjcSysDatabaseError> {
        let mut client = Self::connect(config).await?;
        let jjc_level = i64::from(snapshot.jjc_level);
        let jjc_score = i64::from(snapshot.jjc_score);
        let week_join = i32::from(snapshot.week_join);
        let week_win = i32::from(snapshot.week_win);
        let week_lose = i32::from(snapshot.week_lose);
        let week_tie = i32::from(snapshot.week_tie);
        let season_join = i32::from(snapshot.season_join);
        let season_win = i32::from(snapshot.season_win);
        let season_lose = i32::from(snapshot.season_lose);
        // VERIFIED_DISASSEMBLY: WorldServer 0x00516C5C передаёт [player+0x998].
        let season_tie = week_tie;
        let parameters: [&dyn ToSql; 11] = [
            &snapshot.id,
            &jjc_level,
            &jjc_score,
            &week_join,
            &week_win,
            &week_lose,
            &week_tie,
            &season_join,
            &season_win,
            &season_lose,
            &season_tie,
        ];
        client.execute(SAVE_JJC_DATA_SQL, &parameters).await?;
        Ok(())
    }
}

impl RsJjcSysOwner for TiberiusRsJjcSys {
    async fn save_jjc_data(&mut self, snapshot: &PlayerJjcDataSnapshot) -> bool {
        match Self::execute_save(self.config.clone(), snapshot).await {
            Ok(()) => true,
            Err(error) => {
                self.notices.push_back(RsJjcSysNotice { error });
                false
            }
        }
    }

    fn pop_notice(&mut self) -> Option<RsJjcSysNotice> {
        self.notices.pop_front()
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp




// ============================================================================
// FUNCTION: CRsJJcSys::GetInstance
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:11
// RVA: 0x00115C50
// ADDRESS: 00515c50
// PROTOTYPE: CRsJJcSys * __cdecl GetInstance(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsJJcSys::LoadJJcData
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:103
// RVA: 0x00115CC0
// ADDRESS: 00515cc0
// PROTOTYPE: bool __thiscall LoadJJcData(CPlayer * param_1, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005163bd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:140
// RVA: 0x001163BD
// ADDRESS: 005163bd
// PROTOTYPE: undefined Catch@005163bd()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00516401
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:143
// RVA: 0x00116401
// ADDRESS: 00516401
// PROTOTYPE: undefined FUN_00516401()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsJJcSys::DbJJC
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:146
// RVA: 0x00116D30
// ADDRESS: 00516d30
// PROTOTYPE: uint __stdcall DbJJC(void * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@00517112
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:163
// RVA: 0x00117112
// ADDRESS: 00517112
// PROTOTYPE: undefined Catch@00517112()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0051712c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:179
// RVA: 0x0011712C
// ADDRESS: 0051712c
// PROTOTYPE: undefined Catch@0051712c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00517173
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:184
// RVA: 0x00117173
// ADDRESS: 00517173
// PROTOTYPE: undefined FUN_00517173()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsJJcSys::JJcWeekClear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:188
// RVA: 0x00117190
// ADDRESS: 00517190
// PROTOTYPE: bool __thiscall JJcWeekClear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsJJcSys::JJcSeasonClear
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:199
// RVA: 0x001171C0
// ADDRESS: 005171c0
// PROTOTYPE: bool __thiscall JJcSeasonClear(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@005173f0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:216
// RVA: 0x001173F0
// ADDRESS: 005173f0
// PROTOTYPE: undefined Catch@005173f0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_00517438
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:219
// RVA: 0x00117438
// ADDRESS: 00517438
// PROTOTYPE: undefined FUN_00517438()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsJJcSys::LoadJJcRank
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:18
// RVA: 0x00117460
// ADDRESS: 00517460
// PROTOTYPE: bool __thiscall LoadJJcRank(vector<tagJJcRank,std::allocator<tagJJcRank>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





































































































































// COMPONENT_VARIANT_END: WorldServer
