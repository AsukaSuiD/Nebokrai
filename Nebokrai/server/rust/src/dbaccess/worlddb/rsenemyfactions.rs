//! DB-владелец `CRsEnemyFactions` исторического WorldServer из
//! `rsenemyfactions.cpp`.
//!
//! Контракт `LoadAllEnemyFactions` и
//! `SaveAllEnemyFactions` —; constructor,
//! destructor и прочий корпус не входят в этот owner и остаются.
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! исходный путь PDB:
//!
//! Загрузчик очищал live-list до открытия собственного независимого World DB
//! connection, читал `SELECT * FROM CSL_FactionWar` в recordset-order и после
//! каждой успешно разобранной строки немедленно вызывал
//! `CFactionWarSys::AddOneEnmeyFaction`. Ошибка connection/query/field оставляла
//! очищенный либо уже набранный prefix; caller игнорировал его bool, писал
//! success-log и продолжал relation/INI стадиями. Rust возвращает этот prefix
//! как typed outcome и не подменяет самостоятельное connection открытием
//! caller-транзакции.
//!
//! Save-владелец выполнял буквальный `DELETE FROM CSL_FactionWar`, затем обходил
//! переданную по значению копию `std::list<tagEnemyFaction*>` в list-order и
//! для каждой записи выполнял
//! `INSERT INTO CSL_FactionWar VALUES(%d,%d,%d)`. Первая ошибка DELETE либо
//! INSERT немедленно возвращала `false`; после всех строк возвращался `true`.
//! Метод использовал caller-owned connection внутри уже начатой транзакции и
//! сам не выполнял begin/commit/rollback.
//!
//! Оригинал PDB задаёт `tagEnemyFaction` размером `0x0C`: signed `long`
//! `lFactionID1` по `+0`, signed `long lFactionID2` по `+4` и unsigned `long`
//! `dwDisandTime` по `+8`; локальный SQL-буфер имел размер `0x1F4`. Оригинал
//! входит в контракт: call-site..
//! передаёт третьим `%d` именно поле `+8`, поэтому Rust сохраняет исходную
//! signed decimal-интерпретацию через `u32 as i32`. Выходы,
//! и catch-путь ставят `AL=0`, а полный успех
//! — `AL=1`.
//!
//! `Option` сохраняет nullable connection и nullable элементы pointer-list.
//! Для null-элемента EXE без проверки разыменовывал `node->_Myval` после уже
//! успешного DELETE и возможных предыдущих INSERT. Достижимость и дальнейший
//! эффект такого UB не доказаны, поэтому безопасная граница возвращает только
//! индекс и не назначает исходнику `false`, skip либо commit. Ordered slice,
//! Tiberius и Rust `Drop` заменяют копию `std::list`, ADO/COM и compiler
//! cleanup; null recordset, catch и служебный эпилог не имеют отдельной
//! семантики.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

use tiberius::{Query, Row};

use crate::dbaccess::worlddb::rssetup::{
    WorldDatabaseConnectionError, WorldDatabaseSettings, WorldTdsClient,
};

const DELETE_ENEMY_FACTIONS_SQL: &str = "DELETE FROM CSL_FactionWar";
const INSERT_ENEMY_FACTION_SQL: &str = "INSERT INTO CSL_FactionWar VALUES(@P1,@P2,@P3)";
const LOAD_ENEMY_FACTIONS_SQL: &str = "SELECT * FROM CSL_FactionWar";

/// Три точных 32-битных поля одной caller-owned save-копии.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EnemyFactionSaveSnapshot {
    pub(crate) faction_id_1: i32,
    pub(crate) faction_id_2: i32,
    pub(crate) disband_time: u32,
}

/// Три оригинал поля одной DB-строки, загруженной до faction-war registry.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EnemyFactionLoadSnapshot {
    pub(crate) faction_id_1: i32,
    pub(crate) faction_id_2: i32,
    pub(crate) disband_time: u32,
}

/// Bool loader-а вместе с уже применяемым caller-ом list-prefix.
#[derive(Debug)]
pub(crate) enum EnemyFactionsLoadOutcome {
    ReturnedTrue(Vec<EnemyFactionLoadSnapshot>),
    ReturnedFalse(Vec<EnemyFactionLoadSnapshot>),
}

/// Локальная неизвестность исходного null-разыменования внутри ordered списка.
#[derive(Clone, Copy, Debug)]
pub(crate) struct EnemyFactionNullEntryBlock {
    pub(crate) row_index: usize,
}

/// Наблюдаемый bool-результат владельца либо локальный missing-fact.
#[derive(Debug)]
pub(crate) enum EnemyFactionsSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(EnemyFactionNullEntryBlock),
}

/// Структурированная замена действующих `PrintErr`-ветвей владельца.
#[derive(Debug)]
pub(crate) enum RsEnemyFactionsNotice {
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

/// Ошибка действующей ADO/TDS-границы без runtime SQL и значений строк.
#[derive(Debug)]
pub(crate) struct RsEnemyFactionsDatabaseError(tiberius::error::Error);

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

/// Узкая объектная граница действующего `CRsEnemyFactions` save-владельца.
pub(crate) trait RsEnemyFactionsOwner {
 /// Открывает самостоятельное connection и читает relation rows в DB-order.
    async fn load_all_enemy_factions(&mut self) -> EnemyFactionsLoadOutcome;

 /// Полностью заменяет строки `CSL_FactionWar` внутри caller-транзакции.
    async fn save_all_enemy_factions(
        &mut self,
        snapshot: &[Option<EnemyFactionSaveSnapshot>],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> EnemyFactionsSaveOutcome;

 /// Забирает следующий исходный log-эквивалент.
    fn pop_notice(&mut self) -> Option<RsEnemyFactionsNotice>;
}

/// Linux/TDS-замена действующей части исходного `CRsEnemyFactions`.
#[derive(Default)]
pub(crate) struct TiberiusRsEnemyFactions {
    settings: Option<WorldDatabaseSettings>,
    notices: VecDeque<RsEnemyFactionsNotice>,
}

impl TiberiusRsEnemyFactions {
 /// Создаёт DB-owner с тем же immutable World DB snapshot, из которого
 /// оригинал открывал свой отдельный ADO connection.
    pub(crate) fn new(settings: WorldDatabaseSettings) -> Self {
        Self {
            settings: Some(settings),
            notices: VecDeque::new(),
        }
    }
}

impl RsEnemyFactionsOwner for TiberiusRsEnemyFactions {
    async fn load_all_enemy_factions(&mut self) -> EnemyFactionsLoadOutcome {
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
                            self.notices.push_back(RsEnemyFactionsNotice::LoadRowMissingValue {
                                row_index,
                                column: $column,
                            });
                            return EnemyFactionsLoadOutcome::ReturnedFalse(relations);
                        }
                        Err(EnemyFactionRowReadError::Database(error)) => {
                            self.notices.push_back(RsEnemyFactionsNotice::LoadRowFailed {
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

    async fn save_all_enemy_factions(
        &mut self,
        snapshot: &[Option<EnemyFactionSaveSnapshot>],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> EnemyFactionsSaveOutcome {
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
                return EnemyFactionsSaveOutcome::BlockedMissingFact(EnemyFactionNullEntryBlock {
                    row_index,
                });
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
