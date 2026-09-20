//! World DB-владелец `CRsRegion` из `rsregion.cpp`, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Save копирует весь `tagRegionParam` до проверки caller connection, затем
//! обновляет существующий `CSL_Region` либо вставляет новую строку и записывает
//! ownership/tax поля одним updateable-recordset эквивалентом. Число затронутых
//! строк не проверяется; caller-транзакция не открывается и не завершается здесь.
//!
//! Load использует отдельное соединение и `ORDER BY RegionID`. Для известного
//! live region он читает остальные пять полей и сразу публикует их; unknown/null
//! region пропускает строку без дальнейшего чтения. Поздняя ошибка сохраняет
//! уже применённый префикс. Tiberius заменяет ADO без staging или rollback.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

use futures_util::TryStreamExt;
use tiberius::{Query, Row};

use crate::dbaccess::worlddb::rssetup::{
    WorldDatabaseConnectionError, WorldDatabaseSettings, WorldTdsClient,
};

const REGION_SELECT_SQL: &str = "SELECT TOP 1 RegionID FROM CSL_Region WHERE RegionID = @P1";
const REGION_UPDATE_SQL: &str = "UPDATE TOP (1) CSL_Region SET OwnedFactionID = @P1, OwnedUnionID = @P2, CurTaxRate = @P3, TodayTotalTax = @P4, TotalTax = @P5 WHERE RegionID = @P6";
const REGION_INSERT_SQL: &str = "INSERT INTO CSL_Region (RegionID, OwnedFactionID, OwnedUnionID, CurTaxRate, TodayTotalTax, TotalTax) VALUES (@P1, @P2, @P3, @P4, @P5, @P6)";
const LOAD_REGION_PARAMETERS_SQL: &str = "SELECT * FROM CSL_Region ORDER BY RegionID";

#[derive(Clone, Copy, Debug)]
pub(crate) struct RegionSaveSnapshot {
    pub(crate) region_id: i32,
    pub(crate) max_tax_rate: i32,
    pub(crate) current_tax_rate: i32,
    pub(crate) total_tax: u32,
    pub(crate) today_total_tax: u32,
    pub(crate) superior_region_id: i32,
    pub(crate) turn_in_tax_rate: i32,
    pub(crate) owned_faction_id: i32,
    pub(crate) owned_union_id: i32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RegionDatabaseParameters {
    pub(crate) region_id: i32,
    pub(crate) owned_faction_id: i32,
    pub(crate) owned_union_id: i32,
    pub(crate) current_tax_rate: i32,
    pub(crate) today_total_tax: i32,
    pub(crate) total_tax: i32,
}

/// Явная safe-граница `s_mapRegionList` для построчной DB-публикации.
///
/// Проверка target отделена от применения, потому что EXE не читал остальные
/// поля recordset у отсутствующего либо null региона.
pub(crate) trait RegionParameterLoadTarget {
    fn has_region_parameter_target(&self, region_id: i32) -> bool;
    fn apply_region_database_parameters(&mut self, parameters: RegionDatabaseParameters) -> bool;
}

/// Наблюдаемый normal-result `CRsRegion::LoadRegionParam` и уже применённый
/// prefix при DB/field error.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum RegionParametersLoadOutcome {
    ReturnedTrue {
        visited_rows: usize,
        applied_rows: usize,
    },
    ReturnedFalse {
        visited_rows: usize,
        applied_rows: usize,
    },
}

#[derive(Debug)]
pub(crate) struct RsRegionNotice {
    pub(crate) error: RsRegionError,
}

#[derive(Debug)]
pub(crate) enum RsRegionError {
    Save(RsRegionSaveError),
    Load(RsRegionLoadError),
}

#[derive(Debug)]
pub(crate) enum RsRegionSaveError {
    Database(RsRegionDatabaseError),
    MissingConnection,
}

#[derive(Debug)]
pub(crate) enum RsRegionLoadError {
    MissingSettings,
    Connection(WorldDatabaseConnectionError),
    Database(RsRegionDatabaseError),
    MissingRequiredValue { row_index: usize, column: &'static str },
}

impl fmt::Display for RsRegionLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingSettings => formatter.write_str("отсутствует снимок настроек World DB"),
            Self::Connection(error) => error.fmt(formatter),
            Self::Database(error) => error.fmt(formatter),
            Self::MissingRequiredValue { row_index, column } => write!(
                formatter,
                "в строке CSL_Region {row_index} отсутствует обязательное поле {column}"
            ),
        }
    }
}

impl Error for RsRegionLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Connection(error) => Some(error),
            Self::Database(error) => Some(error),
            Self::MissingSettings | Self::MissingRequiredValue { .. } => None,
        }
    }
}

impl fmt::Display for RsRegionSaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => error.fmt(formatter),
            Self::MissingConnection => {
                formatter.write_str("не передано соединение World region DB")
            }
        }
    }
}

impl Error for RsRegionSaveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::MissingConnection => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct RsRegionDatabaseError(tiberius::error::Error);

impl fmt::Display for RsRegionDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ошибка TDS World region DB: {}", self.0)
    }
}

impl Error for RsRegionDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<tiberius::error::Error> for RsRegionDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self(error)
    }
}

pub(crate) trait RsRegionOwner {
 /// Открывает отдельное connection и немедленно применяет recordset-prefix
 /// к уже опубликованным регионам в порядке `RegionID` SQL-result-а.
    async fn load_region_parameters(
        &mut self,
        target: &mut dyn RegionParameterLoadTarget,
    ) -> RegionParametersLoadOutcome;

    async fn save(
        &mut self,
        snapshot: Option<&RegionSaveSnapshot>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

    fn pop_notice(&mut self) -> Option<RsRegionNotice>;
}

#[derive(Default)]
pub(crate) struct TiberiusRsRegion {
    settings: Option<WorldDatabaseSettings>,
    notices: VecDeque<RsRegionNotice>,
}

impl TiberiusRsRegion {
    pub(crate) fn new(settings: WorldDatabaseSettings) -> Self {
        Self {
            settings: Some(settings),
            notices: VecDeque::new(),
        }
    }
}

impl RsRegionOwner for TiberiusRsRegion {
    async fn load_region_parameters(
        &mut self,
        target: &mut dyn RegionParameterLoadTarget,
    ) -> RegionParametersLoadOutcome {
        let Some(settings) = self.settings.as_ref() else {
            return self.load_failure(
                RsRegionLoadError::MissingSettings,
                0,
                0,
            );
        };
        let mut connection = match settings.connect().await {
            Ok(connection) => connection,
            Err(error) => return self.load_failure(RsRegionLoadError::Connection(error), 0, 0),
        };
        let mut rows = match Query::new(LOAD_REGION_PARAMETERS_SQL)
            .query(&mut connection)
            .await
        {
            Ok(rows) => rows,
            Err(error) => {
                return self.load_failure(RsRegionLoadError::Database(error.into()), 0, 0);
            }
        };

        let mut visited_rows = 0usize;
        let mut applied_rows = 0usize;
        loop {
            let item = match rows.try_next().await {
                Ok(Some(item)) => item,
                Ok(None) => {
                    return RegionParametersLoadOutcome::ReturnedTrue {
                        visited_rows,
                        applied_rows,
                    };
                }
                Err(error) => {
                    return self.load_failure(
                        RsRegionLoadError::Database(error.into()),
                        visited_rows,
                        applied_rows,
                    );
                }
            };
            let Some(row) = item.into_row() else {
                continue;
            };
            let row_index = visited_rows;
            let region_id = match required_i32(&row, row_index, "RegionID") {
                Ok(value) => value,
                Err(error) => return self.load_failure(error, visited_rows, applied_rows),
            };
            visited_rows += 1;
            if !target.has_region_parameter_target(region_id) {
                continue;
            }

            macro_rules! required {
                ($column:literal) => {
                    match required_i32(&row, row_index, $column) {
                        Ok(value) => value,
                        Err(error) => return self.load_failure(error, visited_rows, applied_rows),
                    }
                };
            }
            let parameters = RegionDatabaseParameters {
                region_id,
                owned_faction_id: required!("OwnedFactionID"),
                owned_union_id: required!("OwnedUnionID"),
                current_tax_rate: required!("CurTaxRate"),
                today_total_tax: required!("TodayTotalTax"),
                total_tax: required!("TotalTax"),
            };
            if target.apply_region_database_parameters(parameters) {
                applied_rows += 1;
            }
        }
    }

    async fn save(
        &mut self,
        snapshot: Option<&RegionSaveSnapshot>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool {
        let Some(snapshot) = snapshot else {
            return false;
        };
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(RsRegionNotice {
                error: RsRegionError::Save(RsRegionSaveError::MissingConnection),
            });
            return false;
        };

        let mut select = Query::new(REGION_SELECT_SQL);
        select.bind(snapshot.region_id);
        let row_exists = match select.query(&mut *active_transaction).await {
            Ok(stream) => match stream.into_row().await {
                Ok(row) => row.is_some(),
                Err(error) => return self.database_failure(error),
            },
            Err(error) => return self.database_failure(error),
        };

        let mut save = Query::new(if row_exists {
            REGION_UPDATE_SQL
        } else {
            REGION_INSERT_SQL
        });
        if row_exists {
            save.bind(snapshot.owned_faction_id);
            save.bind(snapshot.owned_union_id);
            save.bind(snapshot.current_tax_rate);
            save.bind(i64::from(snapshot.today_total_tax));
            save.bind(i64::from(snapshot.total_tax));
            save.bind(snapshot.region_id);
        } else {
            save.bind(snapshot.region_id);
            save.bind(snapshot.owned_faction_id);
            save.bind(snapshot.owned_union_id);
            save.bind(snapshot.current_tax_rate);
            save.bind(i64::from(snapshot.today_total_tax));
            save.bind(i64::from(snapshot.total_tax));
        }

        match save.execute(active_transaction).await {
            Ok(_) => true,
            Err(error) => self.database_failure(error),
        }
    }

    fn pop_notice(&mut self) -> Option<RsRegionNotice> {
        self.notices.pop_front()
    }
}

impl TiberiusRsRegion {
    fn database_failure(&mut self, error: tiberius::error::Error) -> bool {
        self.notices.push_back(RsRegionNotice {
            error: RsRegionError::Save(RsRegionSaveError::Database(error.into())),
        });
        false
    }

    fn load_failure(
        &mut self,
        error: RsRegionLoadError,
        visited_rows: usize,
        applied_rows: usize,
    ) -> RegionParametersLoadOutcome {
        self.notices.push_back(RsRegionNotice {
            error: RsRegionError::Load(error),
        });
        RegionParametersLoadOutcome::ReturnedFalse {
            visited_rows,
            applied_rows,
        }
    }
}

fn required_i32(
    row: &Row,
    row_index: usize,
    column: &'static str,
) -> Result<i32, RsRegionLoadError> {
    match row.try_get::<i32, _>(column) {
        Ok(Some(value)) => Ok(value),
        Ok(None) => Err(RsRegionLoadError::MissingRequiredValue { row_index, column }),
        Err(error) => Err(RsRegionLoadError::Database(error.into())),
    }
}
