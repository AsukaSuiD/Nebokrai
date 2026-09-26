//! World DB-владелец `CRsEnemyFactions` из `rsenemyfactions.cpp`,
//! подтверждённый `worldserver.exe` и `worldserver.pdb`.
//!
//! Load очищает live-list до открытия отдельного соединения, читает
//! `CSL_FactionWar` в recordset order и публикует каждую полную строку сразу.
//! Поздняя ошибка оставляет очищенное состояние или уже загруженный префикс.
//!
//! Save в caller-транзакции сначала выполняет DELETE, затем INSERT для каждого
//! элемента в list order. Первая SQL-ошибка завершает метод; begin/commit/
//! rollback принадлежат вызывающему. `dwDisandTime` форматируется как signed
//! decimal bit-pattern. Null list item после DELETE останавливается до прежнего UB.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

use tiberius::{Query, Row};

use crate::persistence::rssetup::{
    WorldDatabaseConnectionError, WorldDatabaseSettings, WorldTdsClient,
};

const DELETE_ENEMY_FACTIONS_SQL: &str = "DELETE FROM CSL_FactionWar";
const INSERT_ENEMY_FACTION_SQL: &str = "INSERT INTO CSL_FactionWar VALUES(@P1,@P2,@P3)";
const LOAD_ENEMY_FACTIONS_SQL: &str = "SELECT * FROM CSL_FactionWar";

#[derive(Clone, Copy, Debug)]
pub struct EnemyFactionSaveSnapshot {
    pub faction_id_1: i32,
    pub faction_id_2: i32,
    pub disband_time: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct EnemyFactionLoadSnapshot {
    pub faction_id_1: i32,
    pub faction_id_2: i32,
    pub disband_time: u32,
}

#[derive(Debug)]
pub enum EnemyFactionsLoadOutcome {
    ReturnedTrue(Vec<EnemyFactionLoadSnapshot>),
    ReturnedFalse(Vec<EnemyFactionLoadSnapshot>),
}

#[derive(Clone, Copy, Debug)]
pub struct EnemyFactionNullEntryBlock {
    pub row_index: usize,
}

#[derive(Debug)]
pub enum EnemyFactionsSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(EnemyFactionNullEntryBlock),
}

#[derive(Debug)]
pub enum RsEnemyFactionsNotice {
    LoadSettingsMissing,
    LoadConnectionFailed(WorldDatabaseConnectionError),
    LoadQueryFailed(RsEnemyFactionsDatabaseError),
    LoadRowMissingValue {
        row_index: usize,
        column: &'static str,
    },
    LoadRowFailed {
        row_index: usize,
        column: &'static str,
        error: RsEnemyFactionsDatabaseError,
    },
    MissingConnection,
    DeleteFailed(RsEnemyFactionsDatabaseError),
    InsertFailed {
        row_index: usize,
        error: RsEnemyFactionsDatabaseError,
    },
}

#[derive(Debug)]
pub struct RsEnemyFactionsDatabaseError(tiberius::error::Error);

impl fmt::Display for RsEnemyFactionsDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ошибка TDS World enemy factions DB: {}", self.0)
    }
}

impl Error for RsEnemyFactionsDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<tiberius::error::Error> for RsEnemyFactionsDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self(error)
    }
}

pub trait RsEnemyFactionsOwner {
    fn load_all_enemy_factions(
        &mut self,
    ) -> impl std::future::Future<Output = EnemyFactionsLoadOutcome> + Send;

    fn save_all_enemy_factions(
        &mut self,
        snapshot: &[Option<EnemyFactionSaveSnapshot>],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = EnemyFactionsSaveOutcome> + Send;

    fn pop_notice(&mut self) -> Option<RsEnemyFactionsNotice>;
}

#[derive(Default)]
pub struct TiberiusRsEnemyFactions {
    settings: Option<WorldDatabaseSettings>,
    notices: VecDeque<RsEnemyFactionsNotice>,
}

impl TiberiusRsEnemyFactions {
    /// Создаёт DB-owner с тем же immutable World DB snapshot, из которого
    /// оригинал открывал свой отдельный ADO connection.
    pub fn new(settings: WorldDatabaseSettings) -> Self {
        Self {
            settings: Some(settings),
            notices: VecDeque::new(),
        }
    }
}

impl RsEnemyFactionsOwner for TiberiusRsEnemyFactions {
    fn load_all_enemy_factions(
        &mut self,
    ) -> impl std::future::Future<Output = EnemyFactionsLoadOutcome> + Send {
        async move {
            let Some(settings) = self.settings.as_ref() else {
                self.notices
                    .push_back(RsEnemyFactionsNotice::LoadSettingsMissing);
                return EnemyFactionsLoadOutcome::ReturnedFalse(Vec::new());
            };
            let mut connection = match settings.connect().await {
                Ok(connection) => connection,
                Err(error) => {
                    self.notices
                        .push_back(RsEnemyFactionsNotice::LoadConnectionFailed(error));
                    return EnemyFactionsLoadOutcome::ReturnedFalse(Vec::new());
                }
            };
            let stream = match Query::new(LOAD_ENEMY_FACTIONS_SQL)
                .query(&mut connection)
                .await
            {
                Ok(stream) => stream,
                Err(error) => {
                    self.notices
                        .push_back(RsEnemyFactionsNotice::LoadQueryFailed(error.into()));
                    return EnemyFactionsLoadOutcome::ReturnedFalse(Vec::new());
                }
            };
            let rows = match stream.into_first_result().await {
                Ok(rows) => rows,
                Err(error) => {
                    self.notices
                        .push_back(RsEnemyFactionsNotice::LoadQueryFailed(error.into()));
                    return EnemyFactionsLoadOutcome::ReturnedFalse(Vec::new());
                }
            };

            let mut relations = Vec::with_capacity(rows.len());
            for (row_index, row) in rows.iter().enumerate() {
                macro_rules! required_i32 {
                    ($column:literal) => {
                        match read_required_i32(row, $column) {
                            Ok(value) => value,
                            Err(EnemyFactionRowReadError::Missing) => {
                                self.notices.push_back(
                                    RsEnemyFactionsNotice::LoadRowMissingValue {
                                        row_index,
                                        column: $column,
                                    },
                                );
                                return EnemyFactionsLoadOutcome::ReturnedFalse(relations);
                            }
                            Err(EnemyFactionRowReadError::Database(error)) => {
                                self.notices
                                    .push_back(RsEnemyFactionsNotice::LoadRowFailed {
                                        row_index,
                                        column: $column,
                                        error: error.into(),
                                    });
                                return EnemyFactionsLoadOutcome::ReturnedFalse(relations);
                            }
                        }
                    };
                }
                let faction_id_1 = required_i32!("FactionID1");
                let faction_id_2 = required_i32!("FactionID2");
                let disband_time = required_i32!("LeaveTime") as u32;
                relations.push(EnemyFactionLoadSnapshot {
                    faction_id_1,
                    faction_id_2,
                    disband_time,
                });
            }
            EnemyFactionsLoadOutcome::ReturnedTrue(relations)
        }
    }

    fn save_all_enemy_factions(
        &mut self,
        snapshot: &[Option<EnemyFactionSaveSnapshot>],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = EnemyFactionsSaveOutcome> + Send {
        async move {
            let Some(active_transaction) = active_transaction else {
                self.notices
                    .push_back(RsEnemyFactionsNotice::MissingConnection);
                return EnemyFactionsSaveOutcome::ReturnedFalse;
            };

            if let Err(error) = execute_batch(active_transaction, DELETE_ENEMY_FACTIONS_SQL).await {
                self.notices
                    .push_back(RsEnemyFactionsNotice::DeleteFailed(error.into()));
                return EnemyFactionsSaveOutcome::ReturnedFalse;
            }

            for (row_index, row) in snapshot.iter().enumerate() {
                let Some(row) = row else {
                    // typed boundary: что наблюдалось после разыменования null
                    // в WorldServer ? Минимум: `mov eax,[edi+8]`;
                    // `mov ecx,[eax+8]`. DELETE и предыдущие INSERT уже выполнены.
                    return EnemyFactionsSaveOutcome::BlockedMissingFact(
                        EnemyFactionNullEntryBlock { row_index },
                    );
                };

                let mut insert = Query::new(INSERT_ENEMY_FACTION_SQL);
                insert.bind(row.faction_id_1);
                insert.bind(row.faction_id_2);
                insert.bind(row.disband_time as i32);
                if let Err(error) = insert.execute(&mut *active_transaction).await {
                    self.notices.push_back(RsEnemyFactionsNotice::InsertFailed {
                        row_index,
                        error: error.into(),
                    });
                    return EnemyFactionsSaveOutcome::ReturnedFalse;
                }
            }

            EnemyFactionsSaveOutcome::ReturnedTrue
        }
    }

    fn pop_notice(&mut self) -> Option<RsEnemyFactionsNotice> {
        self.notices.pop_front()
    }
}

enum EnemyFactionRowReadError {
    Missing,
    Database(tiberius::error::Error),
}

fn read_required_i32(row: &Row, column: &'static str) -> Result<i32, EnemyFactionRowReadError> {
    match row.try_get::<i32, _>(column) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(EnemyFactionRowReadError::Missing),
        Err(error) => Err(EnemyFactionRowReadError::Database(error)),
    }
}

async fn execute_batch(
    connection: &mut WorldTdsClient,
    sql: &'static str,
) -> Result<(), tiberius::error::Error> {
    connection.simple_query(sql).await?.into_results().await?;
    Ok(())
}
