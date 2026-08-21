//! DB-владелец `CDBCountry` исторического WorldServer из `dbcountry.cpp`.
//!
//! Статус `CDBCountry::Save` RVA `0x000F59A0` и `CDBCountry::Load` RVA
//! `0x000F6770` — `IMPLEMENTED`; constructor, destructor и прочий корпус ниже
//! остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная
//! пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256
//! EXE `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`,
//! PDB `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbcountry.cpp`.
//!
//! Owner проверяет country pointer, затем caller-owned connection и выбирает
//! `CSL_Countrys` по unsigned byte `_country_id`. Отсутствующая строка является
//! успешным no-op. Для существующей строки единственный `Recordset::Update`
//! сохраняет строго 12 country/king значений, затем по должностям `2..=7`
//! четыре колонки `minister_%d_id/name/appoint/salary`. Null minister даёт
//! `0, "", 0, 0`; обычный — signed ID, Windows-1251 C-string name и два
//! bool-флага, расширенных до `VT_I4` `0/1`.
//!
//! Exact PDB задаёт `CCountry` поля `_country_id: unsigned char` по `+0x4`,
//! signed `long` treasury/power/tech по `+0x8/+0xC/+0x10/+0x14`, `CKing` по
//! `+0x24` и signed `m_lCountryWarRes` по `+0xA8`. `CKing` содержит inherited
//! signed ID по `+0x4`, byte-exact name по `+0x8`, appointed/salary по
//! `+0x26/+0x27` и signed control/material/war points по `+0x28/+0x2C/+0x30`.
//! `CMinister` использует те же inherited ID/name и два флага по `+0x26/+0x27`.
//! Rust snapshot хранит только этот read-view и не объявляет layout копией ABI.
//!
//! `VERIFIED_DISASSEMBLY`: `0x004F5A22..0x004F5A38` передаёт в select именно
//! zero-extended country byte; `0x004F5AB8..0x004F5AC6` направляет EOF сразу
//! в normal cleanup. `0x004F6181..0x004F61A9` задаёт цикл `2..7` и передаёт
//! его текущий индекс каждому `minister_%d_*`. Normal/no-row выход ставит
//! `AL=1` по `0x004F66F3`, а null country, missing connection и catch —
//! `AL=0` по `0x004F6745`. После каждого конкретного ответа reverse прекращён.
//!
//! Tiberius `SELECT TOP 1` и параметризованный `UPDATE TOP (1)` заменяют только
//! updateable ADO recordset, BSTR/VARIANT и COM lifetime. Число обновлённых
//! строк исходник не проверял. `Vec`, fixed array и Rust `Drop` заменяют только
//! MSVC string/map и compiler cleanup; метод использует уже активную caller-
//! транзакцию и не выполняет begin/commit/rollback. Raw `Save`, его catch и
//! служебный эпилог удалены; точная copy-paste строка catch `load Country`
//! сохранена обязанностью structured notice без SQL и runtime значений.
//!
//! `Load` читает `SELECT * FROM CSL_Countrys` в cursor-order. Treasury/power
//! сначала зажимаются снизу нулём, затем сверху соответствующим максимумом;
//! technology exp ограничивается constructor-значением level-up exp до чтения
//! `tech_lel`, а сам level зажимается только снизу. Эта странная очередность
//! подтверждена RAW и сохраняется как DB-наблюдаемая compatibility quirk.
//! Три king point ограничиваются только сверху. Для каждого row всегда
//! создаются minister owner-ы jobs `2..=7`, после чего handler `Append`
//! перезаписывает duplicate country ID; Rust освобождает прежний owner вместо
//! исходной утечки. Tiberius cursor, `Vec` и Windows-1251 conversion заменяют
//! ADO/BSTR/VARIANT и небезопасные 32-byte stack buffers, не ограничивая имена
//! искусственной длиной.

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

/// Поля king identity/state, которые `CDBCountry::Save` записывал в DB.
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

/// Nullable value одной из шести сохраняемых minister-должностей.
#[derive(Clone, Debug)]
pub(crate) struct CountryMinisterSaveSnapshot {
    pub(crate) id: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) appointed: bool,
    pub(crate) salary_received: bool,
}

/// Полный caller-owned read-view одной country save-копии.
#[derive(Clone, Debug)]
pub(crate) struct CountrySaveSnapshot {
    pub(crate) country_id: u8,
    pub(crate) treasury: i32,
    pub(crate) power: i32,
    pub(crate) tech_current_exp: i32,
    pub(crate) tech_level: i32,
    pub(crate) king: CountryKingSaveSnapshot,
    pub(crate) country_war_result: i32,
    /// Индексы `0..6` буквально соответствуют должностям `2..=7`.
    pub(crate) ministers: [Option<CountryMinisterSaveSnapshot>; 6],
}

/// Структурированная замена достигнутых log-ветвей `CDBCountry::Save`.
#[derive(Debug)]
pub(crate) enum DbCountryNotice {
    MissingConnection,
    LoadFailed {
        row_index: Option<usize>,
        failure: DbCountryLoadFailure,
    },
    /// Сохраняет исходную copy-paste категорию `load Country`.
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

/// Ошибка достигнутой ADO/TDS-границы без SQL и runtime country values.
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

/// Узкая объектная граница достигнутого `CDBCountry::Save`.
pub(crate) trait DbCountryOwner {
    /// Загружает все country rows в live handler, сохраняя cursor order.
    async fn load(
        &mut self,
        country_handler: &mut CCountryHandler,
        country_parameters: &mut CCountryParam,
        active_connection: Option<&mut WorldTdsClient>,
    ) -> bool;

    /// Обновляет существующую country-строку внутри caller-транзакции.
    async fn save(
        &mut self,
        snapshot: Option<&CountrySaveSnapshot>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

    /// Забирает следующий исходный log-эквивалент.
    fn pop_notice(&mut self) -> Option<DbCountryNotice>;
}

/// Linux/TDS-замена достигнутой части исходного `CDBCountry`.
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

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbcountry.cpp



// ============================================================================
// FUNCTION: CDBCountry::Save
// STATUS: IMPLEMENTED
// Реализация и локальная спецификация находятся выше.

// ============================================================================
// FUNCTION: CDBCountry::Load
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbcountry.cpp:15
// RVA: 0x000F6770
// ADDRESS: 004f6770
// PROTOTYPE: bool __thiscall Load(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004f7693
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbcountry.cpp:78
// RVA: 0x000F7693
// ADDRESS: 004f7693
// PROTOTYPE: undefined Catch@004f7693()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004f76f2
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbcountry.cpp:83
// RVA: 0x000F76F2
// ADDRESS: 004f76f2
// PROTOTYPE: undefined FUN_004f76f2()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//






// ============================================================================
// FUNCTION: Unwind@0053aae8
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbcountry.cpp
// RVA: 0x0013AAE8
// ADDRESS: 0053aae8
// PROTOTYPE: undefined Unwind@0053aae8()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// ============================================================================
// FUNCTION: Unwind@0053ab36
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbcountry.cpp
// RVA: 0x0013AB36
// ADDRESS: 0053ab36
// PROTOTYPE: undefined Unwind@0053ab36()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// ============================================================================
// FUNCTION: Unwind@0053ab75
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbcountry.cpp
// RVA: 0x0013AB75
// ADDRESS: 0053ab75
// PROTOTYPE: undefined Unwind@0053ab75()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0053ac67
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbcountry.cpp
// RVA: 0x0013AC67
// ADDRESS: 0053ac67
// PROTOTYPE: undefined Unwind@0053ac67()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//




// COMPONENT_VARIANT_END: WorldServer
