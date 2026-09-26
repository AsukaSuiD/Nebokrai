//! DB-владелец Gods Battle WorldServer из `rsgodsbattle.cpp`.
//!
//! Сохраняются SaveFactionXYD, SaveNpcFaction и top-ten query: порядок строк
//! провайдера, значения bool и уже применённые записи при последующей ошибке.
//! Tiberius и typed records заменяют ADO/COM и fixed buffers, не добавляя
//! merge, rollback или сортировку результатов.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

use encoding_rs::WINDOWS_1251;
use tiberius::{Query, Row};

use crate::persistence::rssetup::{
    WorldDatabaseConnectionError, WorldDatabaseSettings, WorldTdsClient,
};
use nebokrai_shared::resources::CGodsBattleConf;

const DELETE_FACTION_XYD_SQL: &str = "DELETE FROM CSL_GODSBATTLE";
const LOAD_FACTION_XYD_SQL: &str = "SELECT * FROM CSL_GODSBATTLE";
const LOAD_NPC_FACTION_SQL: &str = "SELECT NPC_NAME,Faciton FROM CSL_GODSBATTLE_NPC";
const INSERT_FACTION_XYD_SQL: &str =
    "INSERT INTO CSL_GODSBATTLE (RegionID, AFactionXYD, BFactionXYD) VALUES (@P1, @P2, @P3)";
const DELETE_NPC_FACTIONS_SQL: &str = "DELETE FROM CSL_GODSBATTLE_NPC";
const OPEN_NPC_FACTIONS_SQL: &str = "SELECT TOP 0 * FROM CSL_GODSBATTLE_NPC";
const INSERT_NPC_FACTION_SQL: &str =
    "INSERT INTO CSL_GODSBATTLE_NPC (NPC_NAME, Faciton) VALUES (@P1, @P2)";
const TOP_TEN_SZL_SQL: &str = "SELECT TOP 10 Name,SZL,Levels FROM CSL_PLAYER_ABILITY WHERE GodsBattleFaction = @P1 ORDER BY SZL DESC";
const TOP_TEN_NAME_VISIBLE_BYTES: usize = 16;
const NPC_FACTION_NAME_VISIBLE_BYTES: usize = 16;

#[derive(Clone, Copy, Debug)]
pub struct GodsBattleFactionXydSnapshot {
    pub a_faction_xyd: i32,
    pub b_faction_xyd: u32,
}

#[derive(Clone, Debug)]
pub struct GodsBattleNpcFactionSnapshot {
    pub faction: i32,
    pub name: Vec<u8>,
}

#[derive(Clone, Copy, Debug)]
pub enum GodsBattleSaveOperation {
    FactionXyd,
    NpcFaction,
}

#[derive(Debug)]
pub enum RsGodsBattleNotice {
    MissingConnection {
        operation: GodsBattleSaveOperation,
    },
    DeleteFailedIgnored {
        operation: GodsBattleSaveOperation,
        error: RsGodsBattleDatabaseError,
    },
    SaveFailed {
        operation: GodsBattleSaveOperation,
        row_index: Option<usize>,
        error: RsGodsBattleDatabaseError,
    },
    TopTenFailed {
        faction: i32,
        row_index: Option<usize>,
        failure: GodsBattleTopTenFailure,
    },
    LoadFactionXydFailed {
        row_index: Option<usize>,
        failure: GodsBattleFactionXydLoadFailure,
    },
    GetNpcFactionFailed {
        row_index: Option<usize>,
        failure: GodsBattleNpcFactionLoadFailure,
    },
    AutonomousNpcSaveConnectionFailed {
        error: WorldDatabaseConnectionError,
    },
}

#[derive(Debug)]
pub enum GodsBattleFactionXydLoadFailure {
    Connection(WorldDatabaseConnectionError),
    Database(RsGodsBattleDatabaseError),
    MissingRequiredValue {
        column: &'static str,
    },
    NumericOutsideUnsignedLong {
        column: &'static str,
        value: i64,
    },
}

#[derive(Debug)]
pub enum GodsBattleNpcFactionLoadFailure {
    Connection(WorldDatabaseConnectionError),
    Database(RsGodsBattleDatabaseError),
    MissingRequiredValue {
        column: &'static str,
    },
    NumericOutsideUnsignedLong {
        column: &'static str,
        value: i64,
    },
}

#[derive(Debug)]
pub enum GodsBattleTopTenFailure {
    MissingConnection,
    Database(RsGodsBattleDatabaseError),
    MissingRequiredValue {
        column: &'static str,
    },
    NumericOutsideUnsignedLong {
        column: &'static str,
        value: i64,
    },
}

#[derive(Debug)]
pub struct RsGodsBattleDatabaseError(tiberius::error::Error);

impl fmt::Display for RsGodsBattleDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ошибка TDS World GodsBattle DB: {}", self.0)
    }
}

impl Error for RsGodsBattleDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<tiberius::error::Error> for RsGodsBattleDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self(error)
    }
}

pub trait RsGodsBattleOwner {
    fn load_faction_xyd(
        &mut self,
        configuration: &mut CGodsBattleConf,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn get_npc_faction(
        &mut self,
        configuration: &mut CGodsBattleConf,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn save_faction_xyd(
        &mut self,
        snapshot: GodsBattleFactionXydSnapshot,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn save_npc_faction(
        &mut self,
        snapshot: &[GodsBattleNpcFactionSnapshot],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn save_npc_faction_autonomous(
        &mut self,
        snapshot: &[GodsBattleNpcFactionSnapshot],
    ) -> impl std::future::Future<Output = bool> + Send;

    fn get_top_ten_szl_players(
        &mut self,
        faction: i32,
        destination: &mut Vec<u8>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn pop_notice(&mut self) -> Option<RsGodsBattleNotice>;
}

pub struct TiberiusRsGodsBattle {
    settings: WorldDatabaseSettings,
    notices: VecDeque<RsGodsBattleNotice>,
}

impl TiberiusRsGodsBattle {
    pub fn new(settings: &WorldDatabaseSettings) -> Self {
        Self {
            settings: settings.clone(),
            notices: VecDeque::new(),
        }
    }

    pub fn notice_checkpoint(&self) -> usize {
        self.notices.len()
    }

    pub fn drain_notices_after(
        &mut self,
        checkpoint: usize,
    ) -> Vec<RsGodsBattleNotice> {
        if checkpoint >= self.notices.len() {
            return Vec::new();
        }
        self.notices.drain(checkpoint..).collect()
    }
}

impl RsGodsBattleOwner for TiberiusRsGodsBattle {
    fn load_faction_xyd(
        &mut self,
        configuration: &mut CGodsBattleConf,
    ) -> impl std::future::Future<Output = bool> + Send {
        async move {
            macro_rules! load_failed {
                ($row_index:expr, $failure:expr) => {{
                    self.notices
                        .push_back(RsGodsBattleNotice::LoadFactionXydFailed {
                            row_index: $row_index,
                            failure: $failure,
                        });
                    return false;
                }};
            }

            let mut connection = match self.settings.connect().await {
                Ok(connection) => connection,
                Err(error) => {
                    load_failed!(None, GodsBattleFactionXydLoadFailure::Connection(error))
                }
            };
            let rows = match connection.simple_query(LOAD_FACTION_XYD_SQL).await {
                Ok(stream) => match stream.into_first_result().await {
                    Ok(rows) => rows,
                    Err(error) => load_failed!(
                        None,
                        GodsBattleFactionXydLoadFailure::Database(error.into())
                    ),
                },
                Err(error) => load_failed!(
                    None,
                    GodsBattleFactionXydLoadFailure::Database(error.into())
                ),
            };

            for (row_index, row) in rows.iter().enumerate() {
                let faction_a = match read_ado_unsigned_long(row, "AFactionXYD") {
                    Ok(Some(value)) => value,
                    Ok(None) => load_failed!(
                        Some(row_index),
                        GodsBattleFactionXydLoadFailure::MissingRequiredValue {
                            column: "AFactionXYD",
                        }
                    ),
                    Err(ReadUnsignedLongError::Database(error)) => load_failed!(
                        Some(row_index),
                        GodsBattleFactionXydLoadFailure::Database(error.into())
                    ),
                    Err(ReadUnsignedLongError::OutsideRange(value)) => load_failed!(
                        Some(row_index),
                        GodsBattleFactionXydLoadFailure::NumericOutsideUnsignedLong {
                            column: "AFactionXYD",
                            value,
                        }
                    ),
                };
                let faction_b = match read_ado_unsigned_long(row, "BFactionXYD") {
                    Ok(Some(value)) => value,
                    Ok(None) => load_failed!(
                        Some(row_index),
                        GodsBattleFactionXydLoadFailure::MissingRequiredValue {
                            column: "BFactionXYD",
                        }
                    ),
                    Err(ReadUnsignedLongError::Database(error)) => load_failed!(
                        Some(row_index),
                        GodsBattleFactionXydLoadFailure::Database(error.into())
                    ),
                    Err(ReadUnsignedLongError::OutsideRange(value)) => load_failed!(
                        Some(row_index),
                        GodsBattleFactionXydLoadFailure::NumericOutsideUnsignedLong {
                            column: "BFactionXYD",
                            value,
                        }
                    ),
                };
                configuration.set_xyd_from_db(faction_a, faction_b);
            }
            true
        }
    }

    fn get_npc_faction(
        &mut self,
        configuration: &mut CGodsBattleConf,
    ) -> impl std::future::Future<Output = bool> + Send {
        async move {
            macro_rules! load_failed {
                ($row_index:expr, $failure:expr) => {{
                    self.notices
                        .push_back(RsGodsBattleNotice::GetNpcFactionFailed {
                            row_index: $row_index,
                            failure: $failure,
                        });
                    return false;
                }};
            }

            let mut connection = match self.settings.connect().await {
                Ok(connection) => connection,
                Err(error) => {
                    load_failed!(None, GodsBattleNpcFactionLoadFailure::Connection(error))
                }
            };
            let rows = match connection.simple_query(LOAD_NPC_FACTION_SQL).await {
                Ok(stream) => match stream.into_first_result().await {
                    Ok(rows) => rows,
                    Err(error) => load_failed!(
                        None,
                        GodsBattleNpcFactionLoadFailure::Database(error.into())
                    ),
                },
                Err(error) => load_failed!(
                    None,
                    GodsBattleNpcFactionLoadFailure::Database(error.into())
                ),
            };

            for (row_index, row) in rows.iter().enumerate() {
                let name = match row.try_get::<&str, _>("NPC_NAME") {
                    Ok(Some(value)) => value,
                    Ok(None) => load_failed!(
                        Some(row_index),
                        GodsBattleNpcFactionLoadFailure::MissingRequiredValue {
                            column: "NPC_NAME",
                        }
                    ),
                    Err(error) => load_failed!(
                        Some(row_index),
                        GodsBattleNpcFactionLoadFailure::Database(error.into())
                    ),
                };
                let (name, _, _) = WINDOWS_1251.encode(name);
                let mut name = visible_c_string(name.as_ref()).to_vec();
                name.truncate(NPC_FACTION_NAME_VISIBLE_BYTES);

                let faction = match read_ado_unsigned_long(row, "Faciton") {
                    Ok(Some(value)) => value,
                    Ok(None) => load_failed!(
                        Some(row_index),
                        GodsBattleNpcFactionLoadFailure::MissingRequiredValue {
                            column: "Faciton",
                        }
                    ),
                    Err(ReadUnsignedLongError::Database(error)) => load_failed!(
                        Some(row_index),
                        GodsBattleNpcFactionLoadFailure::Database(error.into())
                    ),
                    Err(ReadUnsignedLongError::OutsideRange(value)) => load_failed!(
                        Some(row_index),
                        GodsBattleNpcFactionLoadFailure::NumericOutsideUnsignedLong {
                            column: "Faciton",
                            value,
                        }
                    ),
                };
                let _ = configuration.set_npc_faction(&name, faction as i32);
            }
            true
        }
    }

    fn save_faction_xyd(
        &mut self,
        snapshot: GodsBattleFactionXydSnapshot,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send {
        async move {
            let Some(active_transaction) = active_transaction else {
                self.notices
                    .push_back(RsGodsBattleNotice::MissingConnection {
                        operation: GodsBattleSaveOperation::FactionXyd,
                    });
                return false;
            };

            if let Err(error) = execute_batch(active_transaction, DELETE_FACTION_XYD_SQL).await {
                self.notices
                    .push_back(RsGodsBattleNotice::DeleteFailedIgnored {
                        operation: GodsBattleSaveOperation::FactionXyd,
                        error: error.into(),
                    });
            }

            let mut insert = Query::new(INSERT_FACTION_XYD_SQL);
            insert.bind(0_i64);
            insert.bind(i64::from(snapshot.a_faction_xyd as u32));
            insert.bind(i64::from(snapshot.b_faction_xyd));
            match insert.execute(active_transaction).await {
                Ok(_) => true,
                Err(error) => {
                    self.notices.push_back(RsGodsBattleNotice::SaveFailed {
                        operation: GodsBattleSaveOperation::FactionXyd,
                        row_index: None,
                        error: error.into(),
                    });
                    false
                }
            }
        }
    }

    fn save_npc_faction(
        &mut self,
        snapshot: &[GodsBattleNpcFactionSnapshot],
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send {
        async move {
            let Some(active_transaction) = active_transaction else {
                self.notices
                    .push_back(RsGodsBattleNotice::MissingConnection {
                        operation: GodsBattleSaveOperation::NpcFaction,
                    });
                return false;
            };

            if let Err(error) = execute_batch(active_transaction, DELETE_NPC_FACTIONS_SQL).await {
                self.notices
                    .push_back(RsGodsBattleNotice::DeleteFailedIgnored {
                        operation: GodsBattleSaveOperation::NpcFaction,
                        error: error.into(),
                    });
            }

            if let Err(error) = execute_batch(active_transaction, OPEN_NPC_FACTIONS_SQL).await {
                self.notices.push_back(RsGodsBattleNotice::SaveFailed {
                    operation: GodsBattleSaveOperation::NpcFaction,
                    row_index: None,
                    error: error.into(),
                });
                return false;
            }

            for (row_index, row) in snapshot.iter().enumerate() {
                let (name, _, _) = WINDOWS_1251.decode(visible_c_string(&row.name));
                let mut insert = Query::new(INSERT_NPC_FACTION_SQL);
                insert.bind(name.into_owned());
                insert.bind(row.faction);
                if let Err(error) = insert.execute(&mut *active_transaction).await {
                    self.notices.push_back(RsGodsBattleNotice::SaveFailed {
                        operation: GodsBattleSaveOperation::NpcFaction,
                        row_index: Some(row_index),
                        error: error.into(),
                    });
                    return false;
                }
            }

            true
        }
    }

    fn save_npc_faction_autonomous(
        &mut self,
        snapshot: &[GodsBattleNpcFactionSnapshot],
    ) -> impl std::future::Future<Output = bool> + Send {
        async move {
            let mut connection = match self.settings.connect().await {
                Ok(connection) => connection,
                Err(error) => {
                    self.notices
                        .push_back(RsGodsBattleNotice::AutonomousNpcSaveConnectionFailed {
                            error,
                        });
                    return false;
                }
            };
            self.save_npc_faction(snapshot, Some(&mut connection)).await
        }
    }

    fn get_top_ten_szl_players(
        &mut self,
        faction: i32,
        destination: &mut Vec<u8>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send {
        async move {
            macro_rules! top_ten_failed {
                ($row_index:expr, $failure:expr) => {{
                    self.notices.push_back(RsGodsBattleNotice::TopTenFailed {
                        faction,
                        row_index: $row_index,
                        failure: $failure,
                    });
                    return false;
                }};
            }

            let Some(active_transaction) = active_transaction else {
                top_ten_failed!(None, GodsBattleTopTenFailure::MissingConnection);
            };

            let mut query = Query::new(TOP_TEN_SZL_SQL);
            query.bind(faction);
            let stream = match query.query(active_transaction).await {
                Ok(stream) => stream,
                Err(error) => top_ten_failed!(
                    None,
                    GodsBattleTopTenFailure::Database(error.into())
                ),
            };
            let rows = match stream.into_first_result().await {
                Ok(rows) => rows,
                Err(error) => top_ten_failed!(
                    None,
                    GodsBattleTopTenFailure::Database(error.into())
                ),
            };

            let mut decoded = Vec::with_capacity(rows.len());
            for (row_index, row) in rows.iter().enumerate() {
                let name = match row.try_get::<&str, _>("Name") {
                    Ok(Some(name)) => name,
                    Ok(None) => top_ten_failed!(
                        Some(row_index),
                        GodsBattleTopTenFailure::MissingRequiredValue { column: "Name" }
                    ),
                    Err(error) => top_ten_failed!(
                        Some(row_index),
                        GodsBattleTopTenFailure::Database(error.into())
                    ),
                };
                let (name, _, _) = WINDOWS_1251.encode(name);
                let mut name = visible_c_string(name.as_ref()).to_vec();
                name.truncate(TOP_TEN_NAME_VISIBLE_BYTES);

                let szl = match read_ado_unsigned_long(row, "SZL") {
                    Ok(Some(value)) => value,
                    Ok(None) => top_ten_failed!(
                        Some(row_index),
                        GodsBattleTopTenFailure::MissingRequiredValue { column: "SZL" }
                    ),
                    Err(ReadUnsignedLongError::Database(error)) => top_ten_failed!(
                        Some(row_index),
                        GodsBattleTopTenFailure::Database(error.into())
                    ),
                    Err(ReadUnsignedLongError::OutsideRange(value)) => top_ten_failed!(
                        Some(row_index),
                        GodsBattleTopTenFailure::NumericOutsideUnsignedLong {
                            column: "SZL",
                            value,
                        }
                    ),
                };
                let level = match read_ado_unsigned_long(row, "Levels") {
                    Ok(Some(value)) => value,
                    Ok(None) => top_ten_failed!(
                        Some(row_index),
                        GodsBattleTopTenFailure::MissingRequiredValue { column: "Levels" }
                    ),
                    Err(ReadUnsignedLongError::Database(error)) => top_ten_failed!(
                        Some(row_index),
                        GodsBattleTopTenFailure::Database(error.into())
                    ),
                    Err(ReadUnsignedLongError::OutsideRange(value)) => top_ten_failed!(
                        Some(row_index),
                        GodsBattleTopTenFailure::NumericOutsideUnsignedLong {
                            column: "Levels",
                            value,
                        }
                    ),
                };
                decoded.push((name, szl, level));
            }

            for (name, szl, level) in decoded {
                destination.extend_from_slice(&1_i32.to_le_bytes());
                destination.extend_from_slice(&faction.to_le_bytes());
                destination.extend_from_slice(&name);
                destination.push(0);
                destination.extend_from_slice(&szl.to_le_bytes());
                destination.extend_from_slice(&level.to_le_bytes());
            }
            true
        }
    }

    fn pop_notice(&mut self) -> Option<RsGodsBattleNotice> {
        self.notices.pop_front()
    }
}

fn visible_c_string(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

enum ReadUnsignedLongError {
    Database(tiberius::error::Error),
    OutsideRange(i64),
}

fn read_ado_unsigned_long(
    row: &Row,
    column: &'static str,
) -> Result<Option<u32>, ReadUnsignedLongError> {
    let first_error = match row.try_get::<i32, _>(column) {
        Ok(value) => {
            return value
                .map(i64::from)
                .map(|value| {
                    u32::try_from(value)
                        .map_err(|_| ReadUnsignedLongError::OutsideRange(value))
                })
                .transpose()
        }
        Err(error) => error,
    };
    if let Ok(value) = row.try_get::<u8, _>(column) {
        return Ok(value.map(u32::from));
    }
    if let Ok(value) = row.try_get::<i16, _>(column) {
        return value
            .map(i64::from)
            .map(|value| {
                u32::try_from(value)
                    .map_err(|_| ReadUnsignedLongError::OutsideRange(value))
            })
            .transpose();
    }
    if let Ok(value) = row.try_get::<i64, _>(column) {
        return value
            .map(|value| {
                u32::try_from(value)
                    .map_err(|_| ReadUnsignedLongError::OutsideRange(value))
            })
            .transpose();
    }
    Err(ReadUnsignedLongError::Database(first_error))
}

async fn execute_batch(
    connection: &mut WorldTdsClient,
    sql: &'static str,
) -> Result<(), tiberius::error::Error> {
    connection.simple_query(sql).await?.into_results().await?;
    Ok(())
}
