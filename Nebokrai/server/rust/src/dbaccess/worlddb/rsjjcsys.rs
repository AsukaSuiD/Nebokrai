//! DB-владелец `CRsJJcSys` исторического WorldServer из `rsjjcsys.cpp`.
//!
//! Статус `LoadJJcData` RVA `0x00115CC0`, `SaveJJcData` RVA `0x00116990`,
//! private `DbJJC` RVA `0x00116D30`, `JJcWeekClear` RVA `0x00117190`,
//! `JJcSeasonClear` RVA `0x001171C0` и no-op `LoadJJcRank` RVA `0x00117460`
//! — `IMPLEMENTED`; технические catch/cleanup ниже остаются документацией.
//! Точная пара:
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
//! `LoadJJcData` сохраняет исходный успешный no-op при отсутствии DB-строки:
//! exact `0x005163B9` выставляет `AL=1`, а catch-path `0x005163FF` — `AL=0`.
//! Tiberius заменяет только ADO recordset; parameter binding исключает старую
//! строковую подстановку ID, не меняя выбор строки и порядок восьми счётчиков.
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
//!
//! Exact `DbJJC` выполняет `sp_JJcWeek_BAKE` и только после его успеха
//! закрывает это соединение, открывает новое и выполняет `sp_JJcWeekClear`.
//! `JJcWeekClear` наблюдаемо возвращает только результат старого
//! `_beginthreadex`, а не DB execution; поэтому системный JJC callback всё
//! ещё владеет запуском, тогда как `run_jjc_week_clear_database` точно
//! реализует тело worker-а. `JJcSeasonClear` открывает одно самостоятельное
//! соединение и выполняет `sp_JJcSeasonClear`. В обоих случаях ADO Command,
//! COM cleanup и Win32 thread заменены Tiberius/Rust lifetime и уже внешним
//! worker-ом; последовательность procedures и самостоятельные границы
//! соединений сохранены.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::io;

use tiberius::{Client, Config, Query, Row, ToSql};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use super::rssetup::{WorldDatabaseSettings, WorldTdsClient};
use crate::worldserver::appworld::player::CPlayer;

const SAVE_JJC_DATA_SQL: &str = "EXEC sp_JJccUpdatePlayer @id=@P1, @jjcLevel=@P2, @jjcScore=@P3, @weekJoin=@P4, @weekWin=@P5, @weekLose=@P6, @weekTie=@P7, @seasonJoin=@P8, @seasonWin=@P9, @seasonLose=@P10, @seasonTie=@P11";
const JJC_WEEK_BAKE_SQL: &str = "EXEC sp_JJcWeek_BAKE";
const JJC_WEEK_CLEAR_SQL: &str = "EXEC sp_JJcWeekClear";
const JJC_SEASON_CLEAR_SQL: &str = "EXEC sp_JJcSeasonClear";

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

/// Структурированная замена достигнутых DB-catch `CRsJJcSys`.
#[derive(Debug)]
pub(crate) struct RsJjcSysNotice {
    pub(crate) operation: RsJjcSysOperation,
    pub(crate) error: RsJjcSysDatabaseError,
}

/// DB operation, которой принадлежал исходный `PrintErr`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RsJjcSysOperation {
    SavePlayer,
    WeekBake,
    WeekClear,
    SeasonClear,
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

/// Узкая объектная граница достигнутых load/save-операций `CRsJJcSys`.
pub(crate) trait RsJjcSysOwner {
    /// Загружает caller-строку JJC; отсутствие строки остаётся успешным no-op.
    async fn load_jjc_data(
        &mut self,
        player: &mut CPlayer,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> PlayerJjcLoadOutcome;

    /// Exact `LoadJJcRank` этой пары EXE/PDB не менял vector и возвращал
    /// успех. Обобщённый vector не скрывает JJC layout: он вообще не читается.
    fn load_jjc_rank<Rank>(&mut self, _ranks: &mut Vec<Rank>) -> bool {
        true
    }

    /// Открывает отдельное соединение и выполняет одну исходную procedure.
    async fn save_jjc_data(&mut self, snapshot: &PlayerJjcDataSnapshot) -> bool;

    /// Выполняет тело detached weekly worker-а: BAKE, затем Clear на новом
    /// соединении. Результат не подменяет bool `JJcWeekClear` thread-start.
    async fn run_jjc_week_clear_database(&mut self) -> bool;

    /// Выполняет отдельный сезонный reset и возвращает исходный bool owner-а.
    async fn clear_jjc_season(&mut self) -> bool;

    /// Забирает следующий `Clear JJc week data failed`-эквивалент.
    fn pop_notice(&mut self) -> Option<RsJjcSysNotice>;
}

#[derive(Debug)]
pub(crate) enum PlayerJjcLoadFailure {
    ZeroPlayerId,
    MissingConnection,
    Database(tiberius::error::Error),
    MissingRequiredValue { column: &'static str },
    NumericOutsideLegacyRange {
        column: &'static str,
        value: i64,
        target: &'static str,
    },
}

#[derive(Debug)]
pub(crate) enum PlayerJjcLoadOutcome {
    ReturnedTrue { row_found: bool },
    ReturnedFalse(PlayerJjcLoadFailure),
}

/// Linux/TDS-замена достигнутых load/save-частей исходного singleton-а.
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

    /// Выполняет один ADO Command без параметров на новом самостоятельном DB
    /// соединении; Drop закрывает client до следующей операции DbJJC.
    async fn execute_maintenance(
        config: Config,
        statement: &'static str,
    ) -> Result<(), RsJjcSysDatabaseError> {
        let mut client = Self::connect(config).await?;
        client.simple_query(statement).await?.into_results().await?;
        Ok(())
    }

    fn maintenance_failure(&mut self, operation: RsJjcSysOperation, error: RsJjcSysDatabaseError) -> bool {
        self.notices.push_back(RsJjcSysNotice { operation, error });
        false
    }
}

fn read_jjc_integer(
    row: &Row,
    column: &'static str,
) -> Result<i64, PlayerJjcLoadFailure> {
    let first_error = match row.try_get::<i32, _>(column) {
        Ok(Some(value)) => return Ok(i64::from(value)),
        Ok(None) => return Err(PlayerJjcLoadFailure::MissingRequiredValue { column }),
        Err(source) => source,
    };
    if let Ok(Some(value)) = row.try_get::<u8, _>(column) {
        return Ok(i64::from(value));
    }
    if let Ok(Some(value)) = row.try_get::<i16, _>(column) {
        return Ok(i64::from(value));
    }
    if let Ok(Some(value)) = row.try_get::<i64, _>(column) {
        return Ok(value);
    }
    Err(PlayerJjcLoadFailure::Database(first_error))
}

impl RsJjcSysOwner for TiberiusRsJjcSys {
    async fn load_jjc_data(
        &mut self,
        player: &mut CPlayer,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> PlayerJjcLoadOutcome {
        macro_rules! integer {
            ($row:expr, $column:literal, $target:ty) => {{
                let value = match read_jjc_integer($row, $column) {
                    Ok(value) => value,
                    Err(failure) => return PlayerJjcLoadOutcome::ReturnedFalse(failure),
                };
                match <$target>::try_from(value) {
                    Ok(value) => value,
                    Err(_) => {
                        return PlayerJjcLoadOutcome::ReturnedFalse(
                            PlayerJjcLoadFailure::NumericOutsideLegacyRange {
                                column: $column,
                                value,
                                target: stringify!($target),
                            },
                        );
                    }
                }
            }};
        }

        let player_id = player.get_id();
        if player_id == 0 {
            return PlayerJjcLoadOutcome::ReturnedFalse(PlayerJjcLoadFailure::ZeroPlayerId);
        }
        let Some(active_transaction) = active_transaction else {
            return PlayerJjcLoadOutcome::ReturnedFalse(
                PlayerJjcLoadFailure::MissingConnection,
            );
        };
        let mut query = Query::new("select * from csl_player_jjc where id = @P1");
        query.bind(player_id);
        let row = match query.query(active_transaction).await {
            Ok(stream) => match stream.into_row().await {
                Ok(row) => row,
                Err(source) => {
                    return PlayerJjcLoadOutcome::ReturnedFalse(
                        PlayerJjcLoadFailure::Database(source),
                    );
                }
            },
            Err(source) => {
                return PlayerJjcLoadOutcome::ReturnedFalse(
                    PlayerJjcLoadFailure::Database(source),
                );
            }
        };
        let Some(row) = row else {
            return PlayerJjcLoadOutcome::ReturnedTrue { row_found: false };
        };

        let jjc_level = integer!(&row, "JjcLevel", u32);
        let jjc_score = integer!(&row, "JjcScore", u32);
        let counters = [
            integer!(&row, "weekJoinCnt", u16),
            integer!(&row, "weekWinCnt", u16),
            integer!(&row, "weekLoseCnt", u16),
            integer!(&row, "weekTieCnt", u16),
            integer!(&row, "seasonJoinCnt", u16),
            integer!(&row, "seasonWinCnt", u16),
            integer!(&row, "seasonLoseCnt", u16),
            integer!(&row, "seasonTieCnt", u16),
        ];
        player.apply_loaded_jjc_data(jjc_level, jjc_score, counters);
        PlayerJjcLoadOutcome::ReturnedTrue { row_found: true }
    }

    async fn save_jjc_data(&mut self, snapshot: &PlayerJjcDataSnapshot) -> bool {
        match Self::execute_save(self.config.clone(), snapshot).await {
            Ok(()) => true,
            Err(error) => self.maintenance_failure(RsJjcSysOperation::SavePlayer, error),
        }
    }

    async fn run_jjc_week_clear_database(&mut self) -> bool {
        if let Err(error) = Self::execute_maintenance(self.config.clone(), JJC_WEEK_BAKE_SQL).await {
            return self.maintenance_failure(RsJjcSysOperation::WeekBake, error);
        }
        match Self::execute_maintenance(self.config.clone(), JJC_WEEK_CLEAR_SQL).await {
            Ok(()) => true,
            Err(error) => self.maintenance_failure(RsJjcSysOperation::WeekClear, error),
        }
    }

    async fn clear_jjc_season(&mut self) -> bool {
        match Self::execute_maintenance(self.config.clone(), JJC_SEASON_CLEAR_SQL).await {
            Ok(()) => true,
            Err(error) => self.maintenance_failure(RsJjcSysOperation::SeasonClear, error),
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
// STATUS: IMPLEMENTED_API_SHAPE_REPLACED
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
// STATUS: IMPLEMENTED/VERIFIED_DISASSEMBLY
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:103
// RVA: 0x00115CC0
// ADDRESS: 00515cc0
// PROTOTYPE: bool __thiscall LoadJJcData(CPlayer * param_1, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_2)
//
// IMPLEMENTED_OWNER: `RsJjcSysOwner::load_jjc_data` выше; отсутствие строки
// успешно, DB/типовые ошибки возвращают false, как подтверждает AL-tail EXE.
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
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:146
// RVA: 0x00116D30
// ADDRESS: 00516d30
// PROTOTYPE: uint __stdcall DbJJC(void * param_1)
//
// IMPLEMENTED_OWNER: `RsJjcSysOwner::run_jjc_week_clear_database` строго
// выполняет `sp_JJcWeek_BAKE`, закрывает первый TDS client и только затем
// открывает второй для `sp_JJcWeekClear`. Win32 thread entry/COM lifetime
// заменены async body и Rust Drop; JJC runtime владеет самим scheduling.
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
// STATUS: IMPLEMENTED_API_SHAPE_REPLACED
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
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:199
// RVA: 0x001171C0
// ADDRESS: 005171c0
// PROTOTYPE: bool __thiscall JJcSeasonClear(void)
//
// IMPLEMENTED_OWNER: `RsJjcSysOwner::clear_jjc_season` открывает отдельный
// TDS client и выполняет literal `sp_JJcSeasonClear`, возвращая bool owner-а.
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
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsjjcsys.cpp:18
// RVA: 0x00117460
// ADDRESS: 00517460
// PROTOTYPE: bool __thiscall LoadJJcRank(vector<tagJJcRank,std::allocator<tagJJcRank>_> * param_1)
//
// IMPLEMENTED_OWNER: default `RsJjcSysOwner::load_jjc_rank` оставляет vector
// нетронутым и возвращает true, как exact единственный security-cookie tail.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//





































































































































// COMPONENT_VARIANT_END: WorldServer
