//! DB-владелец `CRsRegion` исторического WorldServer из `rsregion.cpp`.
//!
//! Статус `CRsRegion::Save` RVA `0x000EEB50` — `IMPLEMENTED`; constructor,
//! destructor и `LoadRegionParam` ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsregion.cpp`.
//!
//! `Save(CWorldRegion*, connection)` сначала проверяет region pointer, затем
//! byte-copy-ит весь `CWorldRegion::tagRegionParam` и лишь после этого проверяет
//! caller-connection. Точный PDB задаёт структуру размером `0x24`: девять
//! последовательных 32-битных полей `lID`, `lMaxTaxRate`, `lCurrentTaxRate`,
//! `dwTotalTax`, `dwTodayTotalTax`, `lSupRegionID`, `lTurnInTaxRate`,
//! `lOwnedFactionID`, `lOwnedUnionID`; `dw*` имеют тип `unsigned long`,
//! остальные — `long`. `RegionSaveSnapshot` сохраняет всю копию, хотя этот
//! DB-owner читает из неё только шесть значений.
//!
//! Исходный updateable ADO recordset выбирал `CSL_Region` по `RegionID`. При
//! EOF он открывал table-recordset, делал `AddNew` и задавал `RegionID`; затем
//! в обоих случаях строго записывал `OwnedFactionID`, `OwnedUnionID`,
//! `CurTaxRate`, `TodayTotalTax`, `TotalTax` и единожды вызывал `Update`.
//! `SELECT TOP 1`, `UPDATE TOP (1)` либо явный `INSERT` через Tiberius заменяют
//! только ADO/COM-механику. Число обновлённых строк не проверяется, а
//! `VT_UI4` tax-поля передаются как неотрицательные TDS `i64`, после чего
//! исходная MSSQL-схема выполняет то же целевое преобразование.
//!
//! Exact call-site `0x004EEBF2..0x004EEC04` имеет статус
//! `VERIFIED_DISASSEMBLY`: потерянный raw-аргумент `_sprintf` берётся из первой
//! копии структуры, то есть `lID`. Эпилоги `0x004EF071..0x004EF0CB` отдельно
//! подтверждают `AL=1` только после успешного `Update` и общий `AL=0` для null
//! region, missing connection и catch `Save CSL_Region ERROR`. После этих
//! ответов reverse прекращён.
//!
//! `Option` сохраняет порядок null-проверок: null region возвращает тихий
//! `false`, а missing connection создаёт исходный log-эквивалент `Save Thread
//! Sent Connect Not Found.`. DB-ошибка создаёт один structured notice без SQL
//! и runtime RegionID. Метод использует уже активную caller-транзакцию и сам не
//! выполняет begin/commit/rollback. `VecDeque`, Tiberius и Rust `Drop` заменяют
//! только STL/ADO/COM/compiler cleanup; raw реализованного владельца, catch и
//! служебный cleanup удалены.

use std::collections::VecDeque;
use std::error::Error;
use std::fmt;

use tiberius::Query;

use crate::dbaccess::worlddb::rssetup::WorldTdsClient;

const REGION_SELECT_SQL: &str = "SELECT TOP 1 RegionID FROM CSL_Region WHERE RegionID = @P1";
const REGION_UPDATE_SQL: &str = "UPDATE TOP (1) CSL_Region SET OwnedFactionID = @P1, OwnedUnionID = @P2, CurTaxRate = @P3, TodayTotalTax = @P4, TotalTax = @P5 WHERE RegionID = @P6";
const REGION_INSERT_SQL: &str = "INSERT INTO CSL_Region (RegionID, OwnedFactionID, OwnedUnionID, CurTaxRate, TodayTotalTax, TotalTax) VALUES (@P1, @P2, @P3, @P4, @P5, @P6)";

/// Полная девятиполевая caller-owned копия `tagRegionParam`.
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

/// Структурированная замена достигнутых log-ветвей `CRsRegion::Save`.
#[derive(Debug)]
pub(crate) struct RsRegionNotice {
    pub(crate) error: RsRegionSaveError,
}

#[derive(Debug)]
pub(crate) enum RsRegionSaveError {
    Database(RsRegionDatabaseError),
    MissingConnection,
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

/// Узкая объектная граница достигнутого `CRsRegion::Save`.
pub(crate) trait RsRegionOwner {
    /// Upsert-ит одну region-строку внутри уже активной caller-транзакции.
    async fn save(
        &mut self,
        snapshot: Option<&RegionSaveSnapshot>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

    /// Забирает следующий исходный log-эквивалент.
    fn pop_notice(&mut self) -> Option<RsRegionNotice>;
}

/// Linux/TDS-замена достигнутой части исходного `CRsRegion`.
#[derive(Default)]
pub(crate) struct TiberiusRsRegion {
    notices: VecDeque<RsRegionNotice>,
}

impl RsRegionOwner for TiberiusRsRegion {
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
                error: RsRegionSaveError::MissingConnection,
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
            error: RsRegionSaveError::Database(error.into()),
        });
        false
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsregion.cpp



// ============================================================================
// FUNCTION: CRsRegion::Save
// STATUS: IMPLEMENTED
// Реализация и локальная спецификация находятся выше.

// ============================================================================
// FUNCTION: CRsRegion::LoadRegionParam
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsregion.cpp:22
// RVA: 0x000EF0F0
// ADDRESS: 004ef0f0
// PROTOTYPE: bool __thiscall LoadRegionParam(map<long,CGame::tagRegion,std::less<long>,std::allocator<std::pair<long_const_,CGame::tagRegion>_>_> * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004ef585
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsregion.cpp:65
// RVA: 0x000EF585
// ADDRESS: 004ef585
// PROTOTYPE: undefined Catch@004ef585()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//







// COMPONENT_VARIANT_END: WorldServer
