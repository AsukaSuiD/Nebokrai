//! World DB-владелец `CRsGenVar` из `rsgenvar.cpp`, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`, перенесённый в Realm `persistence/`.
//!
//! Load читает `CSL_GENVAR` в recordset order и сразу передаёт VarName/SValue/
//! CValue в `CVariableList`; поздняя ошибка оставляет применённый префикс.
//!
//! Save пропускает пустые имена. Новая переменная INSERT-ится целиком,
//! существующая обновляет только CValue. Ошибка UPDATE оригиналом игнорировалась,
//! поэтому общий результат может остаться успешным; ошибка INSERT — фатальна.
//! SQL намеренно остаётся буквальным batch без escaping. CP1251 и Tiberius
//! заменяют ANSI/ADO transport, не меняя текст или порядок.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

use encoding_rs::WINDOWS_1251;
use tiberius::{Query, Row};

use crate::content::variablelist::{
    CVariableList, VariableDatabaseLoadDisposition, VariableListSaveSource,
};
use crate::persistence::rssetup::{
    WorldDatabaseConnectionError, WorldDatabaseSettings, WorldTdsClient,
};

const SELECT_PREFIX: &[u8] = b"SELECT * FROM CSL_GENVAR WHERE VarName = '";
const SELECT_SUFFIX: &[u8] = b"'";
const INSERT_PREFIX: &[u8] = b"INSERT INTO CSL_GENVAR(VarName, SValue,CValue) VALUES('";
const INSERT_MIDDLE_INITIAL: &[u8] = b"','";
const INSERT_MIDDLE_CURRENT: &[u8] = b"','";
const INSERT_SUFFIX: &[u8] = b"')";
const UPDATE_PREFIX: &[u8] = b"UPDATE CSL_GENVAR SET CValue='";
const UPDATE_MIDDLE: &[u8] = b"' WHERE VarName = '";
const UPDATE_SUFFIX: &[u8] = b"'";
const LOAD_GENERAL_VARIABLES_SQL: &str = "SELECT * FROM CSL_GENVAR";

#[derive(Debug)]
pub enum GenVarSaveOutcome {
    Saved,
    Failed,
}

#[derive(Debug)]
pub enum GenVarLoadOutcome {
    ReturnedTrue { loaded_rows: usize, applied_rows: usize },
    ReturnedFalse { loaded_rows: usize, applied_rows: usize },
}

#[derive(Clone, Copy, Debug)]
pub enum GenVarDatabaseOperation {
    Select,
    Insert,
    Update,
}

#[derive(Debug)]
pub enum RsGenVarNotice {
    LoadSettingsMissing,
    LoadConnectionFailed(WorldDatabaseConnectionError),
    LoadQueryFailed(RsGenVarDatabaseError),
    LoadRowMissingValue { row_index: usize, column: &'static str },
    LoadRowFailed {
        row_index: usize,
        column: &'static str,
        error: RsGenVarDatabaseError,
    },
    SaveFailed {
        variable_index: usize,
        operation: GenVarDatabaseOperation,
        error: RsGenVarDatabaseError,
    },
    UpdateFailedIgnored {
        variable_index: usize,
        error: RsGenVarDatabaseError,
    },
}

#[derive(Debug)]
pub struct RsGenVarDatabaseError(tiberius::error::Error);

impl fmt::Display for RsGenVarDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "ошибка TDS World general-variable DB: {}",
            self.0
        )
    }
}

impl Error for RsGenVarDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<tiberius::error::Error> for RsGenVarDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self(error)
    }
}

pub trait RsGenVarOwner {
 /// Открывает отдельное World DB connection и применяет recordset в его
 /// исходном порядке к уже опубликованному списку.
    fn load_general_variables(
        &mut self,
        variables: &mut CVariableList,
    ) -> impl std::future::Future<Output = GenVarLoadOutcome> + Send;

    fn save<S: VariableListSaveSource>(
        &mut self,
        variables: &S,
        active_transaction: &mut WorldTdsClient,
    ) -> impl std::future::Future<Output = GenVarSaveOutcome>;

    fn pop_notice(&mut self) -> Option<RsGenVarNotice>;
}

#[derive(Default)]
pub struct TiberiusRsGenVar {
    settings: Option<WorldDatabaseSettings>,
    notices: VecDeque<RsGenVarNotice>,
}

impl TiberiusRsGenVar {
    pub fn new(settings: WorldDatabaseSettings) -> Self {
        Self {
            settings: Some(settings),
            notices: VecDeque::new(),
        }
    }
}

impl RsGenVarOwner for TiberiusRsGenVar {
    fn load_general_variables(
        &mut self,
        variables: &mut CVariableList,
    ) -> impl std::future::Future<Output = GenVarLoadOutcome> + Send {
        async move {
        let Some(settings) = self.settings.as_ref() else {
            self.notices.push_back(RsGenVarNotice::LoadSettingsMissing);
            return GenVarLoadOutcome::ReturnedFalse { loaded_rows: 0, applied_rows: 0 };
        };
        let mut connection = match settings.connect().await {
            Ok(connection) => connection,
            Err(error) => {
                self.notices.push_back(RsGenVarNotice::LoadConnectionFailed(error));
                return GenVarLoadOutcome::ReturnedFalse { loaded_rows: 0, applied_rows: 0 };
            }
        };
        let stream = match Query::new(LOAD_GENERAL_VARIABLES_SQL).query(&mut connection).await {
            Ok(stream) => stream,
            Err(error) => {
                self.notices.push_back(RsGenVarNotice::LoadQueryFailed(error.into()));
                return GenVarLoadOutcome::ReturnedFalse { loaded_rows: 0, applied_rows: 0 };
            }
        };
        let rows = match stream.into_first_result().await {
            Ok(rows) => rows,
            Err(error) => {
                self.notices.push_back(RsGenVarNotice::LoadQueryFailed(error.into()));
                return GenVarLoadOutcome::ReturnedFalse { loaded_rows: 0, applied_rows: 0 };
            }
        };
        let mut applied_rows = 0;
        for (row_index, row) in rows.iter().enumerate() {
            macro_rules! field {
                ($column:literal) => {
                    match read_legacy_text(row, $column) {
                        Ok(value) => value,
                        Err(GenVarRowReadError::Missing) => {
                            self.notices.push_back(RsGenVarNotice::LoadRowMissingValue { row_index, column: $column });
                            return GenVarLoadOutcome::ReturnedFalse { loaded_rows: row_index, applied_rows };
                        }
                        Err(GenVarRowReadError::Database(error)) => {
                            self.notices.push_back(RsGenVarNotice::LoadRowFailed { row_index, column: $column, error: error.into() });
                            return GenVarLoadOutcome::ReturnedFalse { loaded_rows: row_index, applied_rows };
                        }
                    }
                };
            }
            let name = field!("VarName");
            let saved = field!("SValue");
            let current = field!("CValue");
            if !matches!(variables.load_one_var(&name, &saved, &current), VariableDatabaseLoadDisposition::NameNotDeclared) {
                applied_rows += 1;
            }
        }
        GenVarLoadOutcome::ReturnedTrue { loaded_rows: rows.len(), applied_rows }
        }
    }

    fn save<S: VariableListSaveSource>(
        &mut self,
        variables: &S,
        active_transaction: &mut WorldTdsClient,
    ) -> impl std::future::Future<Output = GenVarSaveOutcome> {
        async move {
        for variable_index in 0..variables.variable_count() {
            let row = variables.save_row(variable_index);
            let name = visible_c_string(&row.name);
            if name.is_empty() {
                continue;
            }

            let select = build_sql(&[SELECT_PREFIX, name, SELECT_SUFFIX]);
            let missing = match query_is_empty(active_transaction, select).await {
                Ok(missing) => missing,
                Err(error) => {
                    self.notices.push_back(RsGenVarNotice::SaveFailed {
                        variable_index,
                        operation: GenVarDatabaseOperation::Select,
                        error: error.into(),
                    });
                    return GenVarSaveOutcome::Failed;
                }
            };

            if missing {
                let initial_value = visible_c_string(&row.initial_value);
                let current_value = visible_c_string(&row.current_value);
                let insert = build_sql(&[
                        INSERT_PREFIX,
                        name,
                        INSERT_MIDDLE_INITIAL,
                        initial_value,
                        INSERT_MIDDLE_CURRENT,
                        current_value,
                        INSERT_SUFFIX,
                    ]);
                if let Err(error) = execute_batch(active_transaction, insert).await {
                    self.notices.push_back(RsGenVarNotice::SaveFailed {
                        variable_index,
                        operation: GenVarDatabaseOperation::Insert,
                        error: error.into(),
                    });
                    return GenVarSaveOutcome::Failed;
                }
            } else {
                let current_value = visible_c_string(&row.current_value);
                let update = build_sql(&[
                        UPDATE_PREFIX,
                        current_value,
                        UPDATE_MIDDLE,
                        name,
                        UPDATE_SUFFIX,
                    ]);
                if let Err(error) = execute_batch(active_transaction, update).await {
                    self.notices.push_back(RsGenVarNotice::UpdateFailedIgnored {
                        variable_index,
                        error: error.into(),
                    });
                }
            }
        }

        GenVarSaveOutcome::Saved
        }
    }

    fn pop_notice(&mut self) -> Option<RsGenVarNotice> {
        self.notices.pop_front()
    }
}

enum GenVarRowReadError {
    Missing,
    Database(tiberius::error::Error),
}

fn read_legacy_text(row: &Row, column: &'static str) -> Result<Vec<u8>, GenVarRowReadError> {
    match row.try_get::<&str, _>(column) {
        Ok(Some(value)) => {
            let (encoded, _, _) = WINDOWS_1251.encode(value);
            Ok(encoded.into_owned())
        }
        Ok(None) => Err(GenVarRowReadError::Missing),
        Err(error) => Err(GenVarRowReadError::Database(error)),
    }
}

fn visible_c_string(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

fn build_sql(fragments: &[&[u8]]) -> String {
    let output_bytes = fragments
        .iter()
        .map(|fragment| fragment.len())
        .sum::<usize>();
    let mut sql = Vec::with_capacity(output_bytes);
    for fragment in fragments {
        sql.extend_from_slice(fragment);
    }
    let (decoded, _, _) = WINDOWS_1251.decode(&sql);
    decoded.into_owned()
}

async fn query_is_empty(
    connection: &mut WorldTdsClient,
    sql: String,
) -> Result<bool, tiberius::error::Error> {
    Ok(connection
        .simple_query(sql)
        .await?
        .into_row()
        .await?
        .is_none())
}

async fn execute_batch(
    connection: &mut WorldTdsClient,
    sql: String,
) -> Result<(), tiberius::error::Error> {
    connection.simple_query(sql).await?.into_results().await?;
    Ok(())
}
