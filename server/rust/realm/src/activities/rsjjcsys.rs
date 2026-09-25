//! DB-владелец JJC WorldServer из `rsjjcsys.cpp`, перенесённый в Realm
//! `activities/`.
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.
//!
//! Owner сохраняет load/save игрока, weekly/season clear, исходный порядок
//! команд и значения bool. Player snapshot и Tiberius заменяют прямой доступ
//! к layout, ADO/COM и ручной lifetime без объединения запросов в новую
//! транзакцию и без отката уже выполненных DB-эффектов.
//! Узкий target-трейт заменяет прямую ссылку на `CPlayer`: его реализация
//! живёт у владельца игрока и делегирует inherent-методам игрока.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;
use std::io;

use tiberius::{Client, Config, Query, Row, ToSql};
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

use crate::persistence::row::get_integer;
use crate::persistence::rssetup::{WorldDatabaseSettings, WorldTdsClient};

/// Узкая точка доступа к данным игрока, нужная загрузке JJC.
/// Реализация живёт у владельца игрока (старый `CPlayer`) и делегирует
/// его inherent-методам; имена специально отличаются, чтобы не вступать
/// в конфликт с ними.
pub trait RsJjcSysPlayerTarget {
    fn jjc_target_player_id(&self) -> i32;

    fn apply_loaded_jjc_snapshot(&mut self, jjc_level: u32, jjc_score: u32, counters: [u16; 8]);
}

const SAVE_JJC_DATA_SQL: &str = "EXEC sp_JJccUpdatePlayer @id=@P1, @jjcLevel=@P2, @jjcScore=@P3, @weekJoin=@P4, @weekWin=@P5, @weekLose=@P6, @weekTie=@P7, @seasonJoin=@P8, @seasonWin=@P9, @seasonLose=@P10, @seasonTie=@P11";
const JJC_WEEK_BAKE_SQL: &str = "EXEC sp_JJcWeek_BAKE";
const JJC_WEEK_CLEAR_SQL: &str = "EXEC sp_JJcWeekClear";
const JJC_SEASON_CLEAR_SQL: &str = "EXEC sp_JJcSeasonClear";

type JjcTdsClient = Client<Compat<TcpStream>>;

#[derive(Clone, Copy, Debug)]
pub struct PlayerJjcDataSnapshot {
    pub id: i32,
    pub jjc_level: u32,
    pub jjc_score: u32,
    pub week_join: u16,
    pub week_win: u16,
    pub week_lose: u16,
    pub week_tie: u16,
    pub season_join: u16,
    pub season_win: u16,
    pub season_lose: u16,
    pub season_tie: u16,
}

#[derive(Debug)]
pub struct RsJjcSysNotice {
    pub operation: RsJjcSysOperation,
    pub error: RsJjcSysDatabaseError,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RsJjcSysOperation {
    SavePlayer,
    WeekBake,
    WeekClear,
    SeasonClear,
}

#[derive(Debug)]
pub enum RsJjcSysDatabaseError {
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

pub trait RsJjcSysOwner {
    fn load_jjc_data(
        &mut self,
        player: &mut dyn RsJjcSysPlayerTarget,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = PlayerJjcLoadOutcome>;

 /// Оригинал `LoadJJcRank` этой пары `worldserver.exe`/`worldserver.pdb` не менял vector и возвращал
 /// успех. Обобщённый vector не скрывает JJC layout: он вообще не читается.
    fn load_jjc_rank<Rank>(&mut self, _ranks: &mut Vec<Rank>) -> bool {
        true
    }

    fn save_jjc_data(&mut self, snapshot: &PlayerJjcDataSnapshot) -> impl std::future::Future<Output = bool> + Send;

 /// Выполняет тело detached weekly worker-а: BAKE, затем Clear на новом
 /// соединении. Результат не подменяет bool `JJcWeekClear` thread-start.
    fn run_jjc_week_clear_database(&mut self) -> impl std::future::Future<Output = bool> + Send;

    fn clear_jjc_season(&mut self) -> impl std::future::Future<Output = bool> + Send;

    fn pop_notice(&mut self) -> Option<RsJjcSysNotice>;
}

#[derive(Debug)]
pub enum PlayerJjcLoadFailure {
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
pub enum PlayerJjcLoadOutcome {
    ReturnedTrue { row_found: bool },
    ReturnedFalse(PlayerJjcLoadFailure),
}

pub struct TiberiusRsJjcSys {
    config: Config,
    notices: VecDeque<RsJjcSysNotice>,
}

impl TiberiusRsJjcSys {
    pub fn new(settings: &WorldDatabaseSettings) -> Self {
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
        // В WorldServer этот параметр читается из JJC-поля player snapshot-а.
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
    match get_integer(row, column) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(PlayerJjcLoadFailure::MissingRequiredValue { column }),
        Err(source) => Err(PlayerJjcLoadFailure::Database(source)),
    }
}

impl RsJjcSysOwner for TiberiusRsJjcSys {
    fn load_jjc_data(
        &mut self,
        player: &mut dyn RsJjcSysPlayerTarget,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = PlayerJjcLoadOutcome> {
        async move {
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

        let player_id = player.jjc_target_player_id();
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
        player.apply_loaded_jjc_snapshot(jjc_level, jjc_score, counters);
        PlayerJjcLoadOutcome::ReturnedTrue { row_found: true }
        }
    }

    fn save_jjc_data(&mut self, snapshot: &PlayerJjcDataSnapshot) -> impl std::future::Future<Output = bool> + Send {
        async move {
        match Self::execute_save(self.config.clone(), snapshot).await {
            Ok(()) => true,
            Err(error) => self.maintenance_failure(RsJjcSysOperation::SavePlayer, error),
        }
        }
    }

    fn run_jjc_week_clear_database(&mut self) -> impl std::future::Future<Output = bool> + Send {
        async move {
        if let Err(error) = Self::execute_maintenance(self.config.clone(), JJC_WEEK_BAKE_SQL).await {
            return self.maintenance_failure(RsJjcSysOperation::WeekBake, error);
        }
        match Self::execute_maintenance(self.config.clone(), JJC_WEEK_CLEAR_SQL).await {
            Ok(()) => true,
            Err(error) => self.maintenance_failure(RsJjcSysOperation::WeekClear, error),
        }
        }
    }

    fn clear_jjc_season(&mut self) -> impl std::future::Future<Output = bool> + Send {
        async move {
        match Self::execute_maintenance(self.config.clone(), JJC_SEASON_CLEAR_SQL).await {
            Ok(()) => true,
            Err(error) => self.maintenance_failure(RsJjcSysOperation::SeasonClear, error),
        }
        }
    }

    fn pop_notice(&mut self) -> Option<RsJjcSysNotice> {
        self.notices.pop_front()
    }
}
