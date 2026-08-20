//! DB-владелец `CDBCountry` исторического WorldServer из `dbcountry.cpp`.
//!
//! Статус `CDBCountry::Save` RVA `0x000F59A0` — `IMPLEMENTED`; constructor,
//! destructor, `Load` и прочий корпус ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная
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

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

use encoding_rs::WINDOWS_1251;
use tiberius::Query;

use crate::dbaccess::worlddb::rssetup::WorldTdsClient;

const COUNTRY_SELECT_SQL: &str = "SELECT TOP 1 id FROM CSL_Countrys WHERE id = @P1";
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
    /// Сохраняет исходную copy-paste категорию `load Country`.
    SaveFailed(DbCountryDatabaseError),
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
// STATUS: UNKNOWN (сохранены только метаданные исследования)
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
