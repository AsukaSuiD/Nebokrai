//! DB-владелец союзов и конфедераций WorldServer из `rsunion.cpp`; трейт
//! `RsUnionOwner` и его data-семья перенесены в Realm `organizations/rsunion`,
//! здесь остаётся Tiberius-реализация и их реэкспорт для переходных потребителей.
//! Источник контракта — точная пара `Nworldserver.exe` и `WorldServer.pdb`.
//!
//! Owner сохраняет отдельные delete/save/load операции, ordered membership,
//! исходные signed IDs и false для missing connection/catch. Tiberius и owned
//! values заменяют ADO/COM и nullable pointers без изменения SQL-порядка,
//! partial effects или наблюдаемых результатов.

use std::collections::{BTreeMap, VecDeque};

use encoding_rs::WINDOWS_1251;
use tiberius::{Query, Row};

use crate::dbaccess::worlddb::rssetup::{WorldDatabaseSettings, WorldTdsClient};
use crate::worldserver::appworld::organizingsystem::faction::current_local_member_time;
use crate::worldserver::appworld::organizingsystem::organizing::{
    EPurviewOwnState, TagMemInfo, UnterminatedMemberField,
};

pub(crate) use nebokrai_realm::organizations::rsunion::*;

const UNION_BASE_SELECT_SQL: &str = "SELECT TOP 1 ID FROM CSL_UNION_BaseProperty WHERE ID = @P1";
const UNION_BASE_UPDATE_SQL: &str =
    "UPDATE TOP (1) CSL_UNION_BaseProperty SET MasterID = @P1 WHERE ID = @P2";
const UNION_MEMBER_INSERT_PREFIX: &[u8] = b"INSERT INTO CSL_UNION_Members (UnionID,FactionID,MemberLvl,Title,bControbute,\t\t\t\t\t\t PV_Disband,PV_Exit,PV_DubJobLvl,PV_ConMem,PV_FireOut,PV_Pronounce,PV_LeaveWord,\t\t\t\t\t\t PV_EditLeaveWord,PV_ObtainTax,PV_OperCityGate,PV_EndueROR)\t\t\t\t\t\t VALUES (";
const UNION_MEMBER_INSERT_CAPACITY: usize = 500;
const LOAD_ALL_CONFEDERATIONS_SQL: &str = "SELECT * FROM CSL_UNION_BaseProperty";

#[derive(Default)]
pub(crate) struct TiberiusRsUnion {
    settings: Option<WorldDatabaseSettings>,
    notices: VecDeque<RsUnionNotice>,
}

impl TiberiusRsUnion {
    pub(crate) fn new(settings: WorldDatabaseSettings) -> Self {
        Self {
            settings: Some(settings),
            notices: VecDeque::new(),
        }
    }

    fn load_failed(
        &mut self,
        operation: RsUnionOperation,
        error: UnionLoadReadError,
        records: Vec<UnionDatabaseLoadRecord>,
        reported_count: i32,
    ) -> UnionLoadOutcome {
        self.notices.push_back(RsUnionNotice {
            operation,
            error: error.into(),
        });
        UnionLoadOutcome::ReturnedFalse {
            records,
            reported_count,
        }
    }
}

impl RsUnionOwner for TiberiusRsUnion {
    async fn load_all_confederations(&mut self) -> UnionLoadOutcome {
        let Some(settings) = self.settings.clone() else {
            self.notices.push_back(RsUnionNotice {
                operation: RsUnionOperation::LoadAllConfederation,
                error: RsUnionSaveError::MissingSettings,
            });
            return UnionLoadOutcome::ReturnedFalse {
                records: Vec::new(),
                reported_count: 0,
            };
        };
        let mut connection = match settings.connect().await {
            Ok(connection) => connection,
            Err(error) => {
                self.notices.push_back(RsUnionNotice {
                    operation: RsUnionOperation::LoadAllConfederation,
                    error: RsUnionSaveError::Connection(error),
                });
                return UnionLoadOutcome::ReturnedFalse {
                    records: Vec::new(),
                    reported_count: 0,
                };
            }
        };
        let stream = match Query::new(LOAD_ALL_CONFEDERATIONS_SQL)
            .query(&mut connection)
            .await
        {
            Ok(stream) => stream,
            Err(error) => {
                self.notices.push_back(RsUnionNotice {
                    operation: RsUnionOperation::LoadAllConfederation,
                    error: RsUnionSaveError::Database(error.into()),
                });
                return UnionLoadOutcome::ReturnedFalse {
                    records: Vec::new(),
                    reported_count: 0,
                };
            }
        };
        let rows = match stream.into_first_result().await {
            Ok(rows) => rows,
            Err(error) => {
                self.notices.push_back(RsUnionNotice {
                    operation: RsUnionOperation::LoadAllConfederation,
                    error: RsUnionSaveError::Database(error.into()),
                });
                return UnionLoadOutcome::ReturnedFalse {
                    records: Vec::new(),
                    reported_count: 0,
                };
            }
        };

        let mut records = Vec::with_capacity(rows.len());
        let mut reported_count = 0_i32;
        for row in &rows {
 // Счётчик увеличивается до чтения ID.
 // Saturation заменяет только недостижимое переполнение signed
 // `int`: старый overflow не является игровым контрактом.
            reported_count = reported_count.saturating_add(1);
            let union_id = match read_legacy_i32(row, "ID") {
                Ok(value) => value,
                Err(error) => return self.load_failed(RsUnionOperation::LoadAllConfederation, error, records, reported_count),
            };
            let name = match read_legacy_text(row, "Name") {
                Ok(value) => value,
                Err(error) => return self.load_failed(RsUnionOperation::LoadAllConfederation, error, records, reported_count),
            };
            let master_id = match read_legacy_i32(row, "MasterID") {
                Ok(value) => value,
                Err(error) => return self.load_failed(RsUnionOperation::LoadAllConfederation, error, records, reported_count),
            };
            let members = match load_confe_members(&mut connection, union_id, &mut self.notices).await {
                Ok(members) => members,
                Err(error) => return self.load_failed(RsUnionOperation::LoadConfederationMembers, error, records, reported_count),
            };
            records.push(UnionDatabaseLoadRecord { union_id, name, master_id, members });
        }
        UnionLoadOutcome::ReturnedTrue {
            records,
            reported_count,
        }
    }

    async fn save_confederation(
        &mut self,
        snapshot: Option<&UnionSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> UnionSaveOutcome {
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsUnionNotice {
                operation: RsUnionOperation::SaveConfederation,
                error: RsUnionSaveError::MissingConnection,
            });
            return UnionSaveOutcome::ReturnedFalse;
        };
        let Some(snapshot) = snapshot else {
 // typed boundary: оригинал сразу загружает vtable из
 // CUnion*. Access violation не является исходным bool `false`.
            return UnionSaveOutcome::BlockedMissingFact(UnionSaveBlock::NullUnionPointer);
        };

        let mut select_query = Query::new(UNION_BASE_SELECT_SQL);
        select_query.bind(snapshot.union_id);
        let base_row_exists = match select_query.query(&mut *active_transaction).await {
            Ok(stream) => match stream.into_row().await {
                Ok(row) => row.is_some(),
                Err(error) => {
                    self.push_database_notice(RsUnionOperation::SaveConfederation, error);
                    return UnionSaveOutcome::ReturnedFalse;
                }
            },
            Err(error) => {
                self.push_database_notice(RsUnionOperation::SaveConfederation, error);
                return UnionSaveOutcome::ReturnedFalse;
            }
        };

        if base_row_exists {
            let mut update_query = Query::new(UNION_BASE_UPDATE_SQL);
            update_query.bind(snapshot.master_id);
            update_query.bind(snapshot.union_id);
            if let Err(error) = update_query.execute(&mut *active_transaction).await {
                self.push_database_notice(RsUnionOperation::SaveConfederation, error);
                return UnionSaveOutcome::ReturnedFalse;
            }
        } else {
            let insert = build_union_base_insert(snapshot);
            if let Err(error) = execute_union_statement(active_transaction, insert).await {
                self.push_database_notice(RsUnionOperation::SaveConfederation, error);
                return UnionSaveOutcome::ReturnedFalse;
            }
        }

        match self
            .save_confe_members(Some(snapshot), Some(active_transaction))
            .await
        {
            UnionMembersSaveOutcome::ReturnedTrue => UnionSaveOutcome::ReturnedTrue,
            UnionMembersSaveOutcome::ReturnedFalse => UnionSaveOutcome::ReturnedFalse,
            UnionMembersSaveOutcome::BlockedMissingFact(block) => {
                UnionSaveOutcome::BlockedMissingFact(block.into())
            }
        }
    }

    async fn save_confe_members(
        &mut self,
        snapshot: Option<&UnionSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> UnionMembersSaveOutcome {
        let Some(snapshot) = snapshot else {
            return UnionMembersSaveOutcome::ReturnedFalse;
        };
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsUnionNotice {
                operation: RsUnionOperation::DeleteConfederationMembers,
                error: RsUnionSaveError::MissingConnection,
            });
            return UnionMembersSaveOutcome::ReturnedFalse;
        };

        let delete = format!(
            "DELETE FROM CSL_UNION_Members WHERE UnionID='{}'",
            snapshot.union_id
        )
        .into_bytes();
        if let Err(error) = execute_union_statement(active_transaction, delete).await {
            self.push_database_notice(RsUnionOperation::DeleteConfederationMembers, error);
            return UnionMembersSaveOutcome::ReturnedFalse;
        }

        for member in snapshot.members.values() {
            let insert = match build_union_member_insert(snapshot.union_id, member) {
                Ok(insert) => insert,
                Err(block) => return UnionMembersSaveOutcome::BlockedMissingFact(block),
            };
            if let Err(error) = execute_union_statement(active_transaction, insert).await {
                self.push_database_notice(RsUnionOperation::SaveConfederationMember, error);
                return UnionMembersSaveOutcome::ReturnedFalse;
            }
        }

        UnionMembersSaveOutcome::ReturnedTrue
    }

    async fn del_confederation(
        &mut self,
        union_id: i32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool {
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsUnionNotice {
                operation: RsUnionOperation::DeleteConfederation,
                error: RsUnionSaveError::MissingConnection,
            });
            return false;
        };

        let mut query = Query::new("DELETE CSL_UNION_BaseProperty WHERE ID = @P1");
        query.bind(union_id);
        match query.execute(active_transaction).await {
            Ok(_) => true,
            Err(error) => {
                self.notices.push_back(RsUnionNotice {
                    operation: RsUnionOperation::DeleteConfederation,
                    error: RsUnionSaveError::Database(error.into()),
                });
                false
            }
        }
    }

    fn pop_notice(&mut self) -> Option<RsUnionNotice> {
        self.notices.pop_front()
    }
}

impl TiberiusRsUnion {
    fn push_database_notice(&mut self, operation: RsUnionOperation, error: tiberius::error::Error) {
        self.notices.push_back(RsUnionNotice {
            operation,
            error: RsUnionSaveError::Database(error.into()),
        });
    }
}

/// Ошибка одного оригинал field/query шага load-цепочки до преобразования в
/// диагностическое сообщение. Она не содержит runtime SQL либо DB values.
#[derive(Debug)]
enum UnionLoadReadError {
    Database(tiberius::error::Error),
    MissingRequiredValue(&'static str),
    InvalidPurview { column: &'static str, value: i32 },
    MissingFaction { faction_id: i32 },
}

impl From<UnionLoadReadError> for RsUnionSaveError {
    fn from(error: UnionLoadReadError) -> Self {
        match error {
            UnionLoadReadError::Database(error) => Self::Database(error.into()),
            UnionLoadReadError::MissingRequiredValue(column) => Self::MissingRequiredValue(column),
            UnionLoadReadError::InvalidPurview { column, value } => {
                Self::InvalidPurview { column, value }
            }
            UnionLoadReadError::MissingFaction { faction_id } => Self::MissingFaction { faction_id },
        }
    }
}

fn read_legacy_i32(row: &Row, column: &'static str) -> Result<i32, UnionLoadReadError> {
    match row.try_get::<i32, _>(column) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(UnionLoadReadError::MissingRequiredValue(column)),
        Err(error) => Err(UnionLoadReadError::Database(error)),
    }
}

fn read_legacy_bool(row: &Row, column: &'static str) -> Result<bool, UnionLoadReadError> {
    match row.try_get::<bool, _>(column) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(UnionLoadReadError::MissingRequiredValue(column)),
        Err(error) => Err(UnionLoadReadError::Database(error)),
    }
}

fn read_legacy_text(row: &Row, column: &'static str) -> Result<Vec<u8>, UnionLoadReadError> {
    match row.try_get::<&str, _>(column) {
        Ok(Some(value)) => {
            let (encoded, _, _) = WINDOWS_1251.encode(value);
            Ok(encoded.into_owned())
        }
        Ok(None) => Err(UnionLoadReadError::MissingRequiredValue(column)),
        Err(error) => Err(UnionLoadReadError::Database(error)),
    }
}

async fn load_confe_members(
    connection: &mut WorldTdsClient,
    union_id: i32,
    notices: &mut VecDeque<RsUnionNotice>,
) -> Result<BTreeMap<i32, TagMemInfo>, UnionLoadReadError> {
 // Literal сохраняет исходный пробел перед signed `%d`; значение заведомо
 // вмещалось в `char[512]` старого owner-а.
    let sql = format!("SELECT * FROM CSL_UNION_Members WHERE UnionID= {union_id}");
    let rows = connection
        .simple_query(sql)
        .await
        .map_err(UnionLoadReadError::Database)?
        .into_first_result()
        .await
        .map_err(UnionLoadReadError::Database)?;
    let member_time = current_local_member_time();
    let mut members = BTreeMap::new();
    for row in &rows {
        let faction_id = read_legacy_i32(row, "FactionID")?;
        let job_level = read_legacy_i32(row, "MemberLvl")?;
        let title = read_legacy_text(row, "Title")?;
        let contribute = read_legacy_bool(row, "bControbute")?;
        let purview = read_union_purview(row)?;
        let name = match load_union_member_name(connection, faction_id).await {
            Ok(name) => name,
            Err(error) => {
 // Оригинал `LoadConfeMembers` не проверяет false
 // `GetUnionMemInfo`: его собственный PrintErr уже случился,
 // current member не вставляется, следующий record читается.
                notices.push_back(RsUnionNotice {
                    operation: RsUnionOperation::LoadUnionMemberInfo,
                    error: error.into(),
                });
                continue;
            }
        };
        let member = TagMemInfo::from_complete_fields(
            faction_id,
            copy_legacy_short_field(&name),
            0,
            0,
            job_level,
            copy_legacy_short_field(&title),
            purview,
            [0; 64],
            member_time,
            contribute,
        );
 // `map::operator[]` перезаписывал duplicate key последней строкой.
        members.insert(faction_id, member);
    }
    Ok(members)
}

async fn load_union_member_name(
    connection: &mut WorldTdsClient,
    faction_id: i32,
) -> Result<Vec<u8>, UnionLoadReadError> {
    let sql = format!("SELECT * FROM CSL_FACTION_BaseProperty WHERE ID={faction_id}");
    let rows = connection
        .simple_query(sql)
        .await
        .map_err(UnionLoadReadError::Database)?
        .into_first_result()
        .await
        .map_err(UnionLoadReadError::Database)?;
    let Some(row) = rows.first() else {
        return Err(UnionLoadReadError::MissingFaction { faction_id });
    };
    read_legacy_text(row, "Name")
}

fn read_union_purview(
    row: &Row,
) -> Result<[EPurviewOwnState; 11], UnionLoadReadError> {
    const COLUMNS: [&str; 11] = [
        "PV_Disband",
        "PV_Exit",
        "PV_DubJobLvl",
        "PV_ConMem",
        "PV_FireOut",
        "PV_Pronounce",
        "PV_LeaveWord",
        "PV_EditLeaveWord",
        "PV_ObtainTax",
        "PV_OperCityGate",
        "PV_EndueROR",
    ];
    let mut output = [EPurviewOwnState::No; 11];
    for (index, column) in COLUMNS.into_iter().enumerate() {
        let value = read_legacy_i32(row, column)?;
        output[index] = match value {
            0 => EPurviewOwnState::No,
            1 => EPurviewOwnState::Forbid,
            2 => EPurviewOwnState::Permit,
            _ => return Err(UnionLoadReadError::InvalidPurview { column, value }),
        };
    }
    Ok(output)
}

fn copy_legacy_short_field<const CAPACITY: usize>(source: &[u8]) -> [u8; CAPACITY] {
    let visible = source
        .iter()
        .position(|byte| *byte == 0)
        .map_or(source, |end| &source[..end]);
    let mut output = [0; CAPACITY];
 // Оригинал сравнивал длину с 0x15, хотя destination Title/Name больше.
    if visible.len() < 0x15 && visible.len() < CAPACITY {
        output[..visible.len()].copy_from_slice(visible);
    }
    output
}

fn build_union_base_insert(snapshot: &UnionSaveSnapshot<'_>) -> Vec<u8> {
    let name_end = snapshot
        .name
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(snapshot.name.len());
    let mut sql = format!(
        "INSERT INTO CSL_UNION_BaseProperty (ID,Name,MasterID) VALUES ({},'",
        snapshot.union_id
    )
    .into_bytes();
    sql.extend_from_slice(&snapshot.name[..name_end]);
    sql.extend_from_slice(format!("',{})", snapshot.master_id).as_bytes());

    sql
}

fn build_union_member_insert(
    union_id: i32,
    member: &TagMemInfo,
) -> Result<Vec<u8>, UnterminatedMemberField> {
    let title = member.title_wire_bytes()?;
    let purview = member.purview.map(|state| state.wire_value());
    let contribute = if member.contribute { 1 } else { 0 };

    let mut sql = UNION_MEMBER_INSERT_PREFIX.to_vec();
    sql.extend_from_slice(format!("{union_id},{},{},'", member.id, member.job_level).as_bytes());
    sql.extend_from_slice(&title[..title.len() - 1]);
    sql.extend_from_slice(
        format!(
            "',{contribute},{},{},{},{},{},{},{},{},{},{},{})",
            purview[0],
            purview[1],
            purview[2],
            purview[3],
            purview[4],
            purview[5],
            purview[6],
            purview[7],
            purview[8],
            purview[9],
            purview[10]
        )
        .as_bytes(),
    );

 // При 63 title bytes, любых i32 и завершающем NUL оригинал format требует не
 // более 480 байт старого `char[500]`; отдельной runtime-границы здесь нет.
    debug_assert!(sql.len() < UNION_MEMBER_INSERT_CAPACITY);
    Ok(sql)
}

async fn execute_union_statement(
    active_transaction: &mut WorldTdsClient,
    sql: Vec<u8>,
) -> Result<(), tiberius::error::Error> {
    let (sql, _, _) = WINDOWS_1251.decode(&sql);
    active_transaction
        .simple_query(sql.into_owned())
        .await?
        .into_results()
        .await?;
    Ok(())
}
