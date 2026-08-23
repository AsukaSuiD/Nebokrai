//! DB-владелец стран WorldServer из `dbcountry.cpp`.
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.
//!
//! Save/load сохраняют byte country ID, ordered ministers, technology и exile
//! records, исходные значения bool и порядок SQL-команд. Tiberius, `BTreeMap`
//! и owned snapshots заменяют ADO/COM и MSVC containers; транзакция, rollback
//! и нормализация provider-order поверх исходного контракта не добавляются.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

use encoding_rs::WINDOWS_1251;
use tiberius::{Query, Row};

use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::worldserver::appworld::country::country::{
    CCountry, CountryMinisterState,
};
use crate::worldserver::appworld::country::countryhandler::{
    CCountryHandler, CountryAppendDisposition,
};
use crate::worldserver::appworld::country::countryparam::CCountryParam;

const COUNTRY_SELECT_SQL: &str = "SELECT TOP 1 id FROM CSL_Countrys WHERE id = @P1";
const COUNTRY_LOAD_SQL: &str = "SELECT * FROM CSL_Countrys";
const COUNTRY_UPDATE_SQL: &str = "UPDATE TOP (1) CSL_Countrys SET treasury = @P1, power = @P2, tech_exp = @P3, tech_lel = @P4, king_id = @P5, king_name = @P6, king_appoint = @P7, king_salary = @P8, control_point = @P9, material_point = @P10, war_point = @P11, war_res = @P12, minister_2_id = @P13, minister_2_name = @P14, minister_2_appoint = @P15, minister_2_salary = @P16, minister_3_id = @P17, minister_3_name = @P18, minister_3_appoint = @P19, minister_3_salary = @P20, minister_4_id = @P21, minister_4_name = @P22, minister_4_appoint = @P23, minister_4_salary = @P24, minister_5_id = @P25, minister_5_name = @P26, minister_5_appoint = @P27, minister_5_salary = @P28, minister_6_id = @P29, minister_6_name = @P30, minister_6_appoint = @P31, minister_6_salary = @P32, minister_7_id = @P33, minister_7_name = @P34, minister_7_appoint = @P35, minister_7_salary = @P36 WHERE id = @P37";

#[derive(Clone, Debug)]
pub(crate) struct CountryKingSaveSnapshot {
    pub(crate) id: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) appointed: bool,
    pub(crate) salary_received: bool,
    pub(crate) control_point: i32,
    pub(crate) material_point: i32,
    pub(crate) war_point: i32,
}

#[derive(Clone, Debug)]
pub(crate) struct CountryMinisterSaveSnapshot {
    pub(crate) id: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) appointed: bool,
    pub(crate) salary_received: bool,
}

#[derive(Clone, Debug)]
pub(crate) struct CountrySaveSnapshot {
    pub(crate) country_id: u8,
    pub(crate) treasury: i32,
    pub(crate) power: i32,
    pub(crate) tech_current_exp: i32,
    pub(crate) tech_level: i32,
    pub(crate) king: CountryKingSaveSnapshot,
    pub(crate) country_war_result: i32,
    pub(crate) ministers: [Option<CountryMinisterSaveSnapshot>; 6],
}

#[derive(Debug)]
pub(crate) enum DbCountryNotice {
    MissingConnection,
    LoadFailed {
        row_index: Option<usize>,
        failure: DbCountryLoadFailure,
    },
    SaveFailed(DbCountryDatabaseError),
}

#[derive(Debug)]
pub(crate) enum DbCountryLoadFailure {
    MissingConnection,
    Database(DbCountryDatabaseError),
    MissingRequiredValue { column: String },
    NumericOutsideRange { column: String, value: i64 },
    ParameterUnavailable { field: &'static str },
}

#[derive(Debug)]
pub(crate) struct DbCountryDatabaseError(tiberius::error::Error);

impl fmt::Display for DbCountryDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ошибка TDS World country DB: {}", self.0)
    }
}

impl Error for DbCountryDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<tiberius::error::Error> for DbCountryDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self(error)
    }
}

pub(crate) trait DbCountryOwner {
    async fn load(
        &mut self,
        country_handler: &mut CCountryHandler,
        country_parameters: &mut CCountryParam,
        active_connection: Option<&mut WorldTdsClient>,
    ) -> bool;

    async fn save(
        &mut self,
        snapshot: Option<&CountrySaveSnapshot>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

    fn pop_notice(&mut self) -> Option<DbCountryNotice>;
}

#[derive(Default)]
pub(crate) struct TiberiusDbCountry {
    notices: VecDeque<DbCountryNotice>,
}

impl DbCountryOwner for TiberiusDbCountry {
    async fn load(
        &mut self,
        country_handler: &mut CCountryHandler,
        country_parameters: &mut CCountryParam,
        active_connection: Option<&mut WorldTdsClient>,
    ) -> bool {
        macro_rules! load_failed {
            ($row_index:expr, $failure:expr) => {{
                self.notices.push_back(DbCountryNotice::LoadFailed {
                    row_index: $row_index,
                    failure: $failure,
                });
                return false;
            }};
        }

        let Some(active_connection) = active_connection else {
            load_failed!(None, DbCountryLoadFailure::MissingConnection);
        };
        let stream = match Query::new(COUNTRY_LOAD_SQL)
            .query(&mut *active_connection)
            .await
        {
            Ok(stream) => stream,
            Err(error) => load_failed!(
                None,
                DbCountryLoadFailure::Database(error.into())
            ),
        };
        let rows = match stream.into_first_result().await {
            Ok(rows) => rows,
            Err(error) => load_failed!(
                None,
                DbCountryLoadFailure::Database(error.into())
            ),
        };

        for (row_index, row) in rows.iter().enumerate() {
            macro_rules! required_i32 {
                ($column:expr) => {
                    match read_ado_long(row, $column) {
                        Ok(Some(value)) => value,
                        Ok(None) => load_failed!(
                            Some(row_index),
                            DbCountryLoadFailure::MissingRequiredValue {
                                column: $column.to_owned(),
                            }
                        ),
                        Err(ReadCountryLongError::Database(error)) => load_failed!(
                            Some(row_index),
                            DbCountryLoadFailure::Database(error.into())
                        ),
                        Err(ReadCountryLongError::OutsideRange(value)) => load_failed!(
                            Some(row_index),
                            DbCountryLoadFailure::NumericOutsideRange {
                                column: $column.to_owned(),
                                value,
                            }
                        ),
                    }
                };
            }
            macro_rules! required_bool {
                ($column:expr) => {
                    match read_ado_bool(row, $column) {
                        Ok(Some(value)) => value,
                        Ok(None) => load_failed!(
                            Some(row_index),
                            DbCountryLoadFailure::MissingRequiredValue {
                                column: $column.to_owned(),
                            }
                        ),
                        Err(error) => load_failed!(
                            Some(row_index),
                            DbCountryLoadFailure::Database(error.into())
                        ),
                    }
                };
            }
            macro_rules! required_name {
                ($column:expr) => {
                    match row.try_get::<&str, _>($column) {
                        Ok(Some(value)) => encode_legacy_name(value),
                        Ok(None) => load_failed!(
                            Some(row_index),
                            DbCountryLoadFailure::MissingRequiredValue {
                                column: $column.to_owned(),
                            }
                        ),
                        Err(error) => load_failed!(
                            Some(row_index),
                            DbCountryLoadFailure::Database(error.into())
                        ),
                    }
                };
            }

            let country_id_value = required_i32!("id");
            let country_id = match u8::try_from(country_id_value) {
                Ok(value) => value,
                Err(_) => load_failed!(
                    Some(row_index),
                    DbCountryLoadFailure::NumericOutsideRange {
                        column: "id".to_owned(),
                        value: i64::from(country_id_value),
                    }
                ),
            };
            let (mut country, _) = CCountry::with_constructor_state(country_parameters);
            country.country_id = country_id;

            let maximum_treasury = match country_parameters.max_country_treasury() {
                Some(value) => value,
                None => load_failed!(
                    Some(row_index),
                    DbCountryLoadFailure::ParameterUnavailable {
                        field: "_max_country_treasury",
                    }
                ),
            };
            country.treasury = required_i32!("treasury").max(0).min(maximum_treasury);

            let maximum_power = match country_parameters.max_country_power() {
                Some(value) => value,
                None => load_failed!(
                    Some(row_index),
                    DbCountryLoadFailure::ParameterUnavailable {
                        field: "_max_country_power",
                    }
                ),
            };
            country.power = required_i32!("power").max(0).min(maximum_power);
            country.tech_current_exp = required_i32!("tech_exp").min(country.tech_level_up_exp);
            country.tech_level = required_i32!("tech_lel").max(0);
            country.king.id = required_i32!("king_id");
            country.king.name = required_name!("king_name");
            country.king.appointed = required_bool!("king_appoint");
            country.king.salary_received = required_bool!("king_salary");

            let maximum_control = match country_parameters.max_king_control_point() {
                Some(value) => value,
                None => load_failed!(
                    Some(row_index),
                    DbCountryLoadFailure::ParameterUnavailable {
                        field: "_max_king_control_point",
                    }
                ),
            };
            country.king.control_point = required_i32!("control_point").min(maximum_control);
            let maximum_material = match country_parameters.max_king_material_point() {
                Some(value) => value,
                None => load_failed!(
                    Some(row_index),
                    DbCountryLoadFailure::ParameterUnavailable {
                        field: "_max_king_material_point",
                    }
                ),
            };
            country.king.material_point = required_i32!("material_point").min(maximum_material);
            let maximum_war = match country_parameters.max_king_war_point() {
                Some(value) => value,
                None => load_failed!(
                    Some(row_index),
                    DbCountryLoadFailure::ParameterUnavailable {
                        field: "_max_king_war_point",
                    }
                ),
            };
            country.king.war_point = required_i32!("war_point").min(maximum_war);
            country.country_war_result = required_i32!("war_res");

            for job in 2_u8..=7 {
                let id_column = format!("minister_{job}_id");
                let name_column = format!("minister_{job}_name");
                let appointed_column = format!("minister_{job}_appoint");
                let salary_column = format!("minister_{job}_salary");
                let minister = CountryMinisterState {
                    id_type: job,
                    quest_switch: false,
                    snapshot: CountryMinisterSaveSnapshot {
                        id: required_i32!(id_column.as_str()),
                        name: required_name!(name_column.as_str()),
                        appointed: required_bool!(appointed_column.as_str()),
                        salary_received: required_bool!(salary_column.as_str()),
                    },
                };
                let _ = country.set_minister_from_db(job, Some(minister));
            }
            let append = country_handler.append_country(Some(Box::new(country)));
            debug_assert!(matches!(append, CountryAppendDisposition::Stored { .. }));
        }
        true
    }

    async fn save(
        &mut self,
        snapshot: Option<&CountrySaveSnapshot>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool {
        let Some(snapshot) = snapshot else {
            return false;
        };
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(DbCountryNotice::MissingConnection);
            return false;
        };

        let mut select = Query::new(COUNTRY_SELECT_SQL);
        select.bind(i32::from(snapshot.country_id));
        let row_exists = match select.query(&mut *active_transaction).await {
            Ok(stream) => match stream.into_row().await {
                Ok(row) => row.is_some(),
                Err(error) => return self.database_failure(error),
            },
            Err(error) => return self.database_failure(error),
        };
        if !row_exists {
            return true;
        }

        let (king_name, _, _) = WINDOWS_1251.decode(visible_c_string(&snapshot.king.name));
        let mut update = Query::new(COUNTRY_UPDATE_SQL);
        update.bind(snapshot.treasury);
        update.bind(snapshot.power);
        update.bind(snapshot.tech_current_exp);
        update.bind(snapshot.tech_level);
        update.bind(snapshot.king.id);
        update.bind(king_name.into_owned());
        update.bind(i32::from(snapshot.king.appointed));
        update.bind(i32::from(snapshot.king.salary_received));
        update.bind(snapshot.king.control_point);
        update.bind(snapshot.king.material_point);
        update.bind(snapshot.king.war_point);
        update.bind(snapshot.country_war_result);

        for minister in &snapshot.ministers {
            if let Some(minister) = minister {
                let (name, _, _) = WINDOWS_1251.decode(visible_c_string(&minister.name));
                update.bind(minister.id);
                update.bind(name.into_owned());
                update.bind(i32::from(minister.appointed));
                update.bind(i32::from(minister.salary_received));
            } else {
                update.bind(0_i32);
                update.bind("");
                update.bind(0_i32);
                update.bind(0_i32);
            }
        }
        update.bind(i32::from(snapshot.country_id));

        match update.execute(active_transaction).await {
            Ok(_) => true,
            Err(error) => self.database_failure(error),
        }
    }

    fn pop_notice(&mut self) -> Option<DbCountryNotice> {
        self.notices.pop_front()
    }
}

impl TiberiusDbCountry {
    fn database_failure(&mut self, error: tiberius::error::Error) -> bool {
        self.notices
            .push_back(DbCountryNotice::SaveFailed(error.into()));
        false
    }
}

fn visible_c_string(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

fn encode_legacy_name(value: &str) -> Vec<u8> {
    let (encoded, _, _) = WINDOWS_1251.encode(value);
    visible_c_string(encoded.as_ref()).to_vec()
}

enum ReadCountryLongError {
    Database(tiberius::error::Error),
    OutsideRange(i64),
}

fn read_ado_long(
    row: &Row,
    column: &str,
) -> Result<Option<i32>, ReadCountryLongError> {
    let first_error = match row.try_get::<i32, _>(column) {
        Ok(value) => return Ok(value),
        Err(error) => error,
    };
    if let Ok(value) = row.try_get::<u8, _>(column) {
        return Ok(value.map(i32::from));
    }
    if let Ok(value) = row.try_get::<i16, _>(column) {
        return Ok(value.map(i32::from));
    }
    if let Ok(value) = row.try_get::<i64, _>(column) {
        return value
            .map(|value| {
                i32::try_from(value).map_err(|_| ReadCountryLongError::OutsideRange(value))
            })
            .transpose();
    }
    Err(ReadCountryLongError::Database(first_error))
}

fn read_ado_bool(
    row: &Row,
    column: &str,
) -> Result<Option<bool>, tiberius::error::Error> {
    let first_error = match row.try_get::<bool, _>(column) {
        Ok(value) => return Ok(value),
        Err(error) => error,
    };
    if let Ok(value) = row.try_get::<i32, _>(column) {
        return Ok(value.map(|value| value != 0));
    }
    if let Ok(value) = row.try_get::<u8, _>(column) {
        return Ok(value.map(|value| value != 0));
    }
    if let Ok(value) = row.try_get::<i16, _>(column) {
        return Ok(value.map(|value| value != 0));
    }
    Err(first_error)
}
