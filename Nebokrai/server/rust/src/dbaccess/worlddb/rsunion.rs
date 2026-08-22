//! DB-владелец `CRsUnion` исторического WorldServer из `rsunion.cpp`.
//!
//! Статус `DelConfederation` RVA `0x000F7EF0`, `SaveConfeMembers` RVA
//! `0x000F7FE0`, `SaveConfederation` RVA `0x000F81C0`,
//! `GetUnionMemInfo` RVA `0x000F8500`, `LoadConfeMembers` RVA `0x000F8860`
//! и `LoadAllConfederation` RVA `0x000F93A0` — `IMPLEMENTED`; constructor и
//! destructor ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`;
//! исходный путь PDB:
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsunion.cpp`.
//!
//! PDB задаёт `DelConfederation(long, connection) -> bool`. Raw показывает
//! единственный batch `DELETE CSL_UNION_BaseProperty WHERE ID = %d` на уже
//! активном caller-connection. Exact `0x004F7EF0..0x004F7FD1` имеет статус
//! `VERIFIED_DISASSEMBLY`: `_sprintf` получает исходный signed `long`,
//! успешный `ExecuteCn` явно ставит `AL=1`, а его `false` и null connection —
//! `AL=0`. Ошибка Execute пишет `Delete Union ERROR`; affected rows не
//! проверяются, поэтому отсутствующий ID тоже считается успешным DELETE.
//!
//! Параметризованный TDS сохраняет signed ID и тот же SQL Server DELETE,
//! заменяя только `_sprintf`/ADO transport. Любой `i32` заведомо помещался в
//! исходный `char[256]`, поэтому buffer-границы нет. Метод не открывает и не
//! завершает транзакцию: соседний `DoSaveData` владеет begin/commit. Rust-
//! ссылка выражает валидный connection, а `Option` отдельно сохраняет исходный
//! null-путь; structured notice заменяет два достигнутых `PrintErr` без SQL и
//! runtime union ID.
//!
//! `SaveConfederation(CUnion*, connection)` сначала открывал updateable
//! recordset по signed union ID. Для существующей строки он менял только
//! `MasterID`; при EOF выполнял отдельный INSERT `ID,Name,MasterID`. Имя
//! передавалось как неэкранированный ANSI `%s`, поэтому Rust сохраняет
//! C-string prefix, одинарные кавычки и исходную SQL-parser семантику. Если
//! Rust строит INSERT в owned buffer и не переносит переполнение старого
//! `char[256]`. Tiberius `SELECT TOP 1`, `UPDATE TOP (1)` и INSERT заменяют
//! только ADO recordset; affected rows исходно не проверялись.
//!
//! После base-row owner безусловно вызывает `SaveConfeMembers` на том же
//! caller-connection и возвращает `false` при его `false`. Member-owner сначала
//! удаляет все строки union ID, затем проходит
//! `map<long, COrganizing::tagMemInfo>` в signed-key порядке и выполняет один
//! literal INSERT на значение. Пустая map после DELETE даёт `true`; delete-
//! ошибка и первый insert/catch дают `false`, сохраняя уже выполненные эффекты
//! caller-транзакции. Старый INSERT не экранировал `strTitle` и использовал
//! все одиннадцать `listPV`.
//!
//! Exact `0x004F7FE0..0x004F81BA` имеет статус `VERIFIED_DISASSEMBLY`: оба
//! ID берутся virtual `GetID`, одиннадцать прав читаются по `+0x80..+0xA8`,
//! normal epilogue ставит `AL=1`, null/delete-error/catch — `AL=0`. Диапазон
//! `0x004F81C0..0x004F84FA` подтверждает порядок select/update-or-insert,
//! аргументы `ID,Name,MasterID`, обязательный member-owner, normal `AL=1` и
//! общий `AL=0` для missing connection/catch. После этих ответов reverse
//! прекращён.
//!
//! `BTreeMap` заменяет только MSVC tree и сохраняет signed порядок. Полный
//! `TagMemInfo` уже принадлежит `organizing.rs`; при отсутствии NUL в его
//! `strTitle` member-owner возвращает локальный `BLOCKED_MISSING_FACT` после
//! DELETE/предыдущих INSERT. Null connection проверяется верхним owner-ом до
//! чтения union pointer. Напротив, null union при живом connection исходно
//! немедленно разыменовывался; его неизвестная UB-реакция не заменяется
//! безопасным `false` и остаётся отдельной блокирующей границей.
//!
//! Load-цепочка открывает самостоятельное World DB connection, буквально
//! читает `CSL_UNION_BaseProperty` в recordset-order и для каждого base-row
//! читает `CSL_UNION_Members WHERE UnionID= %d`. Затем для каждого member
//! отдельный `GetUnionMemInfo` читает faction `Name` из
//! `CSL_FACTION_BaseProperty`. Отказ base/member recordset либо его field
//! прекращает проход, не отменяя уже сформированный prefix; отдельный отказ
//! `GetUnionMemInfo` только пропускает текущий member и продолжает цикл.
//! `CUnion` создаётся и публикуется следующим owner-ом
//! `COrganizingCtrl::Initialize`: этот DB owner возвращает именно безопасную
//! data-проекцию, поэтому не подменяет ещё не реконструированный парный
//! `CRsFaction` частичным init-вызовом.
//!
//! PDB задаёт верхнему loader-у `int`, который `Initialize` подставляет в `%d`
//! без проверки успеха. В RAW `local_148` увеличивается перед чтением каждого
//! base-row: при отказе этого row журналируемый счётчик поэтому может быть на
//! единицу больше опубликованного prefix. Rust хранит оба значения отдельно;
//! overflow signed счётчика не сохраняется как внутренний дефект.
//! Старые copy/stack lifetime и неинициализированные region/last-online bytes
//! заменены полностью определёнными Rust-полями; Title и faction Name всё ещё
//! получают исходное правило short-copy: видимый ANSI prefix короче 21 байта,
//! иначе пустое поле. Неизвестный numeric `listPV` не materialize-ится как
//! invalid Rust enum: проход останавливается typed notice, а не создаёт UB.

use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::fmt;

use encoding_rs::WINDOWS_1251;
use tiberius::{Query, Row};

use crate::dbaccess::worlddb::rssetup::{
    WorldDatabaseConnectionError, WorldDatabaseSettings, WorldTdsClient,
};
use crate::worldserver::appworld::organizingsystem::organizing::{
    EPurviewOwnState, TagMemInfo, UnterminatedMemberField,
};
use crate::worldserver::appworld::organizingsystem::faction::current_local_member_time;
use crate::worldserver::appworld::organizingsystem::union::CUnion;

const UNION_BASE_SELECT_SQL: &str = "SELECT TOP 1 ID FROM CSL_UNION_BaseProperty WHERE ID = @P1";
const UNION_BASE_UPDATE_SQL: &str =
    "UPDATE TOP (1) CSL_UNION_BaseProperty SET MasterID = @P1 WHERE ID = @P2";
const UNION_MEMBER_INSERT_PREFIX: &[u8] = b"INSERT INTO CSL_UNION_Members (UnionID,FactionID,MemberLvl,Title,bControbute,\t\t\t\t\t\t PV_Disband,PV_Exit,PV_DubJobLvl,PV_ConMem,PV_FireOut,PV_Pronounce,PV_LeaveWord,\t\t\t\t\t\t PV_EditLeaveWord,PV_ObtainTax,PV_OperCityGate,PV_EndueROR)\t\t\t\t\t\t VALUES (";
const UNION_MEMBER_INSERT_CAPACITY: usize = 500;
const LOAD_ALL_CONFEDERATIONS_SQL: &str = "SELECT * FROM CSL_UNION_BaseProperty";

/// Безопасный результат одного base-row `LoadAllConfederation` до публикации
/// concrete `CUnion` у следующего owner-а.
pub(crate) struct UnionDatabaseLoadRecord {
    pub(crate) union_id: i32,
    pub(crate) name: Vec<u8>,
    pub(crate) master_id: i32,
    pub(crate) members: BTreeMap<i32, TagMemInfo>,
}

/// Результат `LoadAllConfederation`; `records` всегда содержит уже завершённый
/// prefix, как исходные записи в controller-map до первого отказа.
///
/// PDB задаёт `int` return, а не `bool`: exact RAW увеличивает local counter
/// до чтения каждого base-row. Поэтому `reported_count` может на единицу
/// опережать `records.len()` на отказавшем base/member-row — именно это число
/// попадает в последующий `Initialize` log.
pub(crate) enum UnionLoadOutcome {
    ReturnedTrue {
        records: Vec<UnionDatabaseLoadRecord>,
        reported_count: i32,
    },
    ReturnedFalse {
        records: Vec<UnionDatabaseLoadRecord>,
        reported_count: i32,
    },
}

/// Caller-owned save-копия `CUnion`, созданная исходным `CloneSaveData`.
pub(crate) struct UnionSaveSnapshot<'union> {
    pub(crate) union_id: i32,
    /// Логические bytes старого `std::string`; `%s` видит prefix до первого NUL.
    pub(crate) name: &'union [u8],
    pub(crate) master_id: i32,
    /// Полная копия `m_Members` в signed-key порядке.
    pub(crate) members: &'union BTreeMap<i32, TagMemInfo>,
}

impl<'union> UnionSaveSnapshot<'union> {
    /// Заимствует ровно те поля `CUnion` save-копии, которые читает DB-owner.
    pub(crate) fn from_union(union: &'union CUnion) -> Self {
        Self {
            union_id: union.union_id(),
            name: union.name(),
            master_id: union.master_id(),
            members: union.members(),
        }
    }
}

/// Безопасная граница старого null/overread пути union save.
#[derive(Debug)]
pub(crate) enum UnionSaveBlock {
    NullUnionPointer,
    MemberTitle(UnterminatedMemberField),
}

impl fmt::Display for UnionSaveBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NullUnionPointer => formatter
                .write_str("SaveConfederation разыменовывал null CUnion при живом соединении"),
            Self::MemberTitle(block) => block.fmt(formatter),
        }
    }
}

impl Error for UnionSaveBlock {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NullUnionPointer => None,
            Self::MemberTitle(block) => Some(block),
        }
    }
}

impl From<UnterminatedMemberField> for UnionSaveBlock {
    fn from(block: UnterminatedMemberField) -> Self {
        Self::MemberTitle(block)
    }
}

/// Исходный bool `SaveConfeMembers` либо локальный missing-fact.
#[derive(Debug)]
pub(crate) enum UnionMembersSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(UnterminatedMemberField),
}

/// Исходный bool `SaveConfederation` либо локальный missing-fact.
#[derive(Debug)]
pub(crate) enum UnionSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(UnionSaveBlock),
}

/// Структурированная замена достигнутых `CRsUnion` log-ветвей.
#[derive(Debug)]
pub(crate) struct RsUnionNotice {
    pub(crate) operation: RsUnionOperation,
    pub(crate) error: RsUnionSaveError,
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum RsUnionOperation {
    LoadAllConfederation,
    LoadConfederationMembers,
    LoadUnionMemberInfo,
    SaveConfederation,
    DeleteConfederation,
    DeleteConfederationMembers,
    SaveConfederationMember,
}

#[derive(Debug)]
pub(crate) enum RsUnionSaveError {
    Database(RsUnionDatabaseError),
    MissingConnection,
    MissingSettings,
    Connection(WorldDatabaseConnectionError),
    MissingRequiredValue(&'static str),
    InvalidPurview { column: &'static str, value: i32 },
    MissingFaction { faction_id: i32 },
}

impl fmt::Display for RsUnionSaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => error.fmt(formatter),
            Self::MissingConnection => write!(formatter, "не передано соединение World union DB"),
            Self::MissingSettings => write!(formatter, "не заданы параметры World union DB"),
            Self::Connection(error) => error.fmt(formatter),
            Self::MissingRequiredValue(column) => {
                write!(formatter, "в union DB отсутствует обязательное поле {column}")
            }
            Self::InvalidPurview { column, value } => {
                write!(formatter, "недопустимое union-право {column}={value}")
            }
            Self::MissingFaction { faction_id } => {
                write!(formatter, "не найдена faction {faction_id} для union member")
            }
        }
    }
}

impl Error for RsUnionSaveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::Connection(error) => Some(error),
            Self::MissingConnection => None,
            Self::MissingSettings
            | Self::MissingRequiredValue(_)
            | Self::InvalidPurview { .. }
            | Self::MissingFaction { .. } => None,
        }
    }
}

#[derive(Debug)]
pub(crate) struct RsUnionDatabaseError(tiberius::error::Error);

impl fmt::Display for RsUnionDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ошибка TDS World union DB: {}", self.0)
    }
}

impl Error for RsUnionDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<tiberius::error::Error> for RsUnionDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self(error)
    }
}

/// Узкая объектная граница достигнутой стадии исходного `CRsUnion`.
pub(crate) trait RsUnionOwner {
    /// Открывает самостоятельное World DB connection и возвращает готовый
    /// prefix base/member rows в точном исходном порядке.
    async fn load_all_confederations(&mut self) -> UnionLoadOutcome;

    /// Сохраняет base-row и ordered member-map внутри caller-транзакции.
    async fn save_confederation(
        &mut self,
        snapshot: Option<&UnionSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> UnionSaveOutcome;

    /// Перезаписывает ordered member-снимок внутри caller-транзакции.
    async fn save_confe_members(
        &mut self,
        snapshot: Option<&UnionSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> UnionMembersSaveOutcome;

    /// Удаляет base-property row конфедерации внутри caller-транзакции.
    async fn del_confederation(
        &mut self,
        union_id: i32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> bool;

    /// Забирает следующий исходный log-эквивалент.
    fn pop_notice(&mut self) -> Option<RsUnionNotice>;
}

/// Linux/TDS-замена достигнутой части исходного `CRsUnion`.
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
            // Исследовательский разбор увеличивает счётчик перед чтением ID.
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
            // BLOCKED_MISSING_FACT: exact 0x004F8237 сразу загружает vtable из
            // CUnion*. Access violation не является доказанным bool `false`.
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

/// Ошибка одного exact field/query шага load-цепочки до преобразования в
/// operator-visible notice. Она не содержит runtime SQL либо DB values.
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
                // Exact `LoadConfeMembers` не проверяет false
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
    // Exact сравнивал длину с 0x15, хотя destination Title/Name больше.
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

    // При 63 title bytes, любых i32 и завершающем NUL exact format требует не
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

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsunion.cpp



// ============================================================================
// FUNCTION: CRsUnion::GetUnionMemInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsunion.cpp:284
// RVA: 0x000F8500
// ADDRESS: 004f8500
// PROTOTYPE: bool __thiscall GetUnionMemInfo(tagMemInfo * param_1, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004f87f4
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsunion.cpp:319
// RVA: 0x000F87F4
// ADDRESS: 004f87f4
// PROTOTYPE: undefined Catch@004f87f4()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004f8843
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsunion.cpp:324
// RVA: 0x000F8843
// ADDRESS: 004f8843
// PROTOTYPE: undefined FUN_004f8843()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsUnion::LoadConfeMembers
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsunion.cpp:207
// RVA: 0x000F8860
// ADDRESS: 004f8860
// PROTOTYPE: bool __thiscall LoadConfeMembers(CUnion * param_1, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004f9331
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsunion.cpp:274
// RVA: 0x000F9331
// ADDRESS: 004f9331
// PROTOTYPE: undefined Catch@004f9331()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004f937f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsunion.cpp:279
// RVA: 0x000F937F
// ADDRESS: 004f937f
// PROTOTYPE: undefined FUN_004f937f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CRsUnion::LoadAllConfederation
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsunion.cpp:145
// RVA: 0x000F93A0
// ADDRESS: 004f93a0
// PROTOTYPE: int __thiscall LoadAllConfederation(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004f98ca
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\rsunion.cpp:198
// RVA: 0x000F98CA
// ADDRESS: 004f98ca
// PROTOTYPE: undefined Catch@004f98ca()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
