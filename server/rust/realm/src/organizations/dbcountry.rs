//! DB-проекции стран WorldServer из `dbcountry.cpp`, подтверждённые
//! `worldserver.exe` и `worldserver.pdb`. Data-уровень перенесён в Realm
//! `organizations/`.
//!
//! Save/load сохраняют byte country ID, ordered ministers, technology и exile
//! records, исходные значения bool и порядок SQL-команд. Tiberius, `BTreeMap`
//! и owned snapshots заменяют ADO/COM и MSVC containers; транзакция, rollback
//! и нормализация provider-order поверх исходного контракта не добавляются.
//!
//! Трейт `DbCountryOwner` и `TiberiusDbCountry` остаются в worldserver до
//! волны владельца: сигнатуры `load`/`save` ссылаются на ещё не перенесённый
//! `CCountryHandler`, а inherent impl неотделим от типа правилом орфанов.

use std::error::Error;
use std::fmt;

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
