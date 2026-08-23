//! DB-reader журнала increment-shop WorldServer.
//!
//! Источник контракта `CDbIncrementLog::LoadAll` —
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`.
//! Запрос сохраняет `DATEDIFF(day, log_time, GETDATE()) <= days` и обязательный
//! порядок `player_id, log_time`. Каждая уже прочитанная строка публиковалась в
//! `CIncrementLog` до перехода к следующей, поэтому typed результат отдельно
//! возвращает действующий prefix при ошибке следующей строки.
//!
//! Tiberius, параметр `@P1`, owned строки и `chrono::NaiveDateTime` заменяют
//! только ADO connection/recordset, BSTR/VARIANT и `VariantTimeToSystemTime`.
//! Идентификатор DB-строки исходник не читал; ID, календарь, длина описания и
//! общие лимиты дополнительно не проверяются.

use std::error::Error;
use std::fmt;

use chrono::{Datelike, NaiveDateTime, Timelike};
use encoding_rs::WINDOWS_1251;
use futures_util::TryStreamExt;
use tiberius::{Query, Row};

use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::public::date::TagTime;

const LOAD_RECENT_SQL: &str = "SELECT id,type,money,description,log_time,player_id FROM increment_log WHERE DATEDIFF(day,log_time,GETDATE())<=@P1 ORDER BY player_id,log_time";

#[derive(Clone, Debug)]
pub(crate) struct DbIncrementLogRow {
    pub(crate) player_id: i32,
    pub(crate) time: TagTime,
    pub(crate) entry_type: u8,
    pub(crate) money: i32,
    pub(crate) description: Vec<u8>,
}

#[derive(Debug)]
pub(crate) enum DbIncrementLogLoadFailure {
    MissingConnection,
    Database(tiberius::error::Error),
    MissingRequiredValue { row_index: usize, column: &'static str },
    NumericOutsideRange { row_index: usize, column: &'static str },
}

impl fmt::Display for DbIncrementLogLoadFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingConnection => formatter.write_str("отсутствует соединение Log DB"),
            Self::Database(error) => write!(formatter, "ошибка TDS increment log: {error}"),
            Self::MissingRequiredValue { row_index, column } => write!(
                formatter,
                "в строке increment log {row_index} отсутствует поле {column}"
            ),
            Self::NumericOutsideRange { row_index, column } => write!(
                formatter,
                "поле {column} строки increment log {row_index} вне legacy-диапазона"
            ),
        }
    }
}

impl Error for DbIncrementLogLoadFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct DbIncrementLogLoad {
    pub(crate) rows: Vec<DbIncrementLogRow>,
    pub(crate) completion: Result<(), DbIncrementLogLoadFailure>,
}

pub(crate) async fn load_recent(
    active_connection: Option<&mut WorldTdsClient>,
    retention_days: u32,
) -> DbIncrementLogLoad {
    let Some(active_connection) = active_connection else {
        return DbIncrementLogLoad {
            rows: Vec::new(),
            completion: Err(DbIncrementLogLoadFailure::MissingConnection),
        };
    };
    let mut query = Query::new(LOAD_RECENT_SQL);
    query.bind(retention_days as i32);
    let mut stream = match query.query(&mut *active_connection).await {
        Ok(stream) => stream,
        Err(error) => {
            return DbIncrementLogLoad {
                rows: Vec::new(),
                completion: Err(DbIncrementLogLoadFailure::Database(error)),
            };
        }
    };
    let mut rows = Vec::new();
    let mut row_index = 0usize;
    loop {
        let item = match stream.try_next().await {
            Ok(Some(item)) => item,
            Ok(None) => break,
            Err(error) => {
                return DbIncrementLogLoad {
                    rows,
                    completion: Err(DbIncrementLogLoadFailure::Database(error)),
                };
            }
        };
        let Some(source_row) = item.into_row() else {
            continue;
        };
        match parse_row(&source_row, row_index) {
            Ok(row) => rows.push(row),
            Err(failure) => {
                return DbIncrementLogLoad {
                    rows,
                    completion: Err(failure),
                };
            }
        }
        row_index += 1;
    }
    DbIncrementLogLoad {
        rows,
        completion: Ok(()),
    }
}

fn parse_row(
    row: &Row,
    row_index: usize,
) -> Result<DbIncrementLogRow, DbIncrementLogLoadFailure> {
    let player_id = required_i32(row, row_index, "player_id")?;
    let money = required_i32(row, row_index, "money")?;
    let entry_type = required_u8(row, row_index, "type")?;
    let time = row
        .try_get::<NaiveDateTime, _>("log_time")
        .map_err(DbIncrementLogLoadFailure::Database)?
        .ok_or(DbIncrementLogLoadFailure::MissingRequiredValue {
            row_index,
            column: "log_time",
        })?;
    let description = row
        .try_get::<&str, _>("description")
        .map_err(DbIncrementLogLoadFailure::Database)?
        .ok_or(DbIncrementLogLoadFailure::MissingRequiredValue {
            row_index,
            column: "description",
        })?;
    let (description, _, _) = WINDOWS_1251.encode(description);
    let description = description
        .iter()
        .copied()
        .take_while(|byte| *byte != 0)
        .collect();

    Ok(DbIncrementLogRow {
        player_id,
        time: TagTime::from_fields([
            time.year() as u16,
            time.month() as u16,
            time.weekday().num_days_from_sunday() as u16,
            time.day() as u16,
            time.hour() as u16,
            time.minute() as u16,
            time.second() as u16,
            time.and_utc().timestamp_subsec_millis() as u16,
        ]),
        entry_type,
        money,
        description,
    })
}

fn required_i32(
    row: &Row,
    row_index: usize,
    column: &'static str,
) -> Result<i32, DbIncrementLogLoadFailure> {
    if let Ok(value) = row.try_get::<i32, _>(column) {
        return value.ok_or(DbIncrementLogLoadFailure::MissingRequiredValue {
            row_index,
            column,
        });
    }
    if let Ok(value) = row.try_get::<i16, _>(column) {
        return value
            .map(i32::from)
            .ok_or(DbIncrementLogLoadFailure::MissingRequiredValue { row_index, column });
    }
    if let Ok(value) = row.try_get::<u8, _>(column) {
        return value
            .map(i32::from)
            .ok_or(DbIncrementLogLoadFailure::MissingRequiredValue { row_index, column });
    }
    match row.try_get::<i64, _>(column) {
        Ok(Some(value)) => i32::try_from(value).map_err(|_| {
            DbIncrementLogLoadFailure::NumericOutsideRange { row_index, column }
        }),
        Ok(None) => Err(DbIncrementLogLoadFailure::MissingRequiredValue { row_index, column }),
        Err(error) => Err(DbIncrementLogLoadFailure::Database(error)),
    }
}

fn required_u8(
    row: &Row,
    row_index: usize,
    column: &'static str,
) -> Result<u8, DbIncrementLogLoadFailure> {
    let value = required_i32(row, row_index, column)?;
    u8::try_from(value)
        .map_err(|_| DbIncrementLogLoadFailure::NumericOutsideRange { row_index, column })
}
