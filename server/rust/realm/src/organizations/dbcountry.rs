//! DB-проекции стран WorldServer из `dbcountry.cpp`, подтверждённые
//! `worldserver.exe` и `worldserver.pdb`. Data-уровень и трейт владельца
//! перенесены в Realm `organizations/`.
//!
//! Save/load сохраняют byte country ID, ordered ministers, technology и exile
//! records, исходные значения bool и порядок SQL-команд. Tiberius, `BTreeMap`
//! и owned snapshots заменяют ADO/COM и MSVC containers; транзакция, rollback
//! и нормализация provider-order поверх исходного контракта не добавляются.
//!
//! Трейт `DbCountryOwner` перенесён волной владельца после `CCountryHandler`;
//! `TiberiusDbCountry` остаётся facade impl в старом
//! `dbaccess/worlddb/dbcountry` по прецеденту rsunion/rsfaction и там же
//! реэкспортирует этот модуль.
//!
//! Async-методы трейта записаны в desugared-форме по ADR-0013. Каждое future
//! захватывает только `&mut self`, Sync-ссылки snapshot/handler/parameters и
//! Send-соединение `&mut WorldTdsClient`, поэтому `load`/`save` помечены
//! `+ Send`.

use std::error::Error;
use std::fmt;

use crate::content::countryparam::CCountryParam;
use crate::organizations::countryhandler::CCountryHandler;
use crate::persistence::rssetup::WorldTdsClient;

#[derive(Clone, Debug)]
pub struct CountryKingSaveSnapshot {
    pub id: i32,
    pub name: Vec<u8>,
    pub appointed: bool,
    pub salary_received: bool,
    pub control_point: i32,
    pub material_point: i32,
    pub war_point: i32,
}

#[derive(Clone, Debug)]
pub struct CountryMinisterSaveSnapshot {
    pub id: i32,
    pub name: Vec<u8>,
    pub appointed: bool,
    pub salary_received: bool,
}

#[derive(Clone, Debug)]
pub struct CountrySaveSnapshot {
    pub country_id: u8,
    pub treasury: i32,
    pub power: i32,
    pub tech_current_exp: i32,
    pub tech_level: i32,
    pub king: CountryKingSaveSnapshot,
    pub country_war_result: i32,
    pub ministers: [Option<CountryMinisterSaveSnapshot>; 6],
}

#[derive(Debug)]
pub enum DbCountryNotice {
    MissingConnection,
    LoadFailed {
        row_index: Option<usize>,
        failure: DbCountryLoadFailure,
    },
    SaveFailed(DbCountryDatabaseError),
}

#[derive(Debug)]
pub enum DbCountryLoadFailure {
    MissingConnection,
    Database(DbCountryDatabaseError),
    MissingRequiredValue { column: String },
    NumericOutsideRange { column: String, value: i64 },
    ParameterUnavailable { field: &'static str },
}

#[derive(Debug)]
pub struct DbCountryDatabaseError(tiberius::error::Error);

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

pub trait DbCountryOwner {
    fn load(
        &mut self,
        country_handler: &mut CCountryHandler,
        country_parameters: &mut CCountryParam,
        active_connection: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn save(
        &mut self,
        snapshot: Option<&CountrySaveSnapshot>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn pop_notice(&mut self) -> Option<DbCountryNotice>;
}
