//! DB-владелец `CRsUnion` исторического WorldServer из `rsunion.cpp`.
//!
//! Статус `DelConfederation` RVA `0x000F7EF0`, `SaveConfeMembers` RVA
//! `0x000F7FE0` и `SaveConfederation` RVA `0x000F81C0` — `IMPLEMENTED`;
//! constructor, destructor и остальные функции ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
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
//! INSERT вместе с NUL не помещается в старый `char[256]`, возвращается
//! локальный `BLOCKED_MISSING_FACT`, а результат `_sprintf` overflow не
//! придумывается. Tiberius `SELECT TOP 1`, `UPDATE TOP (1)` и INSERT заменяют
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

use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::fmt;

use encoding_rs::WINDOWS_1251;
use tiberius::Query;

use crate::dbaccess::worlddb::rssetup::WorldTdsClient;
use crate::worldserver::appworld::organizingsystem::organizing::{
    TagMemInfo, UnterminatedMemberField,
};
use crate::worldserver::appworld::organizingsystem::union::CUnion;

const UNION_BASE_SELECT_SQL: &str = "SELECT TOP 1 ID FROM CSL_UNION_BaseProperty WHERE ID = @P1";
const UNION_BASE_UPDATE_SQL: &str =
    "UPDATE TOP (1) CSL_UNION_BaseProperty SET MasterID = @P1 WHERE ID = @P2";
const UNION_MEMBER_INSERT_PREFIX: &[u8] = b"INSERT INTO CSL_UNION_Members (UnionID,FactionID,MemberLvl,Title,bControbute,\t\t\t\t\t\t PV_Disband,PV_Exit,PV_DubJobLvl,PV_ConMem,PV_FireOut,PV_Pronounce,PV_LeaveWord,\t\t\t\t\t\t PV_EditLeaveWord,PV_ObtainTax,PV_OperCityGate,PV_EndueROR)\t\t\t\t\t\t VALUES (";
const UNION_BASE_INSERT_CAPACITY: usize = 256;
const UNION_MEMBER_INSERT_CAPACITY: usize = 500;

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

/// Неизвестный результат переполнения base INSERT `char[256]`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct UnionBaseInsertBufferBlock {
    pub(crate) required_with_nul: usize,
    pub(crate) capacity: usize,
}

impl fmt::Display for UnionBaseInsertBufferBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "union base INSERT требует {} байт при старой ёмкости {}",
            self.required_with_nul, self.capacity
        )
    }
}

impl Error for UnionBaseInsertBufferBlock {}

/// Безопасная граница старого null/overread/overflow пути union save.
#[derive(Debug)]
pub(crate) enum UnionSaveBlock {
    NullUnionPointer,
    BaseInsertBuffer(UnionBaseInsertBufferBlock),
    MemberTitle(UnterminatedMemberField),
}

impl fmt::Display for UnionSaveBlock {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NullUnionPointer => formatter
                .write_str("SaveConfederation разыменовывал null CUnion при живом соединении"),
            Self::BaseInsertBuffer(block) => block.fmt(formatter),
            Self::MemberTitle(block) => block.fmt(formatter),
        }
    }
}

impl Error for UnionSaveBlock {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::NullUnionPointer => None,
            Self::BaseInsertBuffer(block) => Some(block),
            Self::MemberTitle(block) => Some(block),
        }
    }
}

impl From<UnionBaseInsertBufferBlock> for UnionSaveBlock {
    fn from(block: UnionBaseInsertBufferBlock) -> Self {
        Self::BaseInsertBuffer(block)
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
    SaveConfederation,
    DeleteConfederation,
    DeleteConfederationMembers,
    SaveConfederationMember,
}

#[derive(Debug)]
pub(crate) enum RsUnionSaveError {
    Database(RsUnionDatabaseError),
    MissingConnection,
}

impl fmt::Display for RsUnionSaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => error.fmt(formatter),
            Self::MissingConnection => write!(formatter, "не передано соединение World union DB"),
        }
    }
}

impl Error for RsUnionSaveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::MissingConnection => None,
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
    notices: VecDeque<RsUnionNotice>,
}

impl RsUnionOwner for TiberiusRsUnion {
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
            let insert = match build_union_base_insert(snapshot) {
                Ok(insert) => insert,
                Err(block) => {
                    return UnionSaveOutcome::BlockedMissingFact(block.into());
                }
            };
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

fn build_union_base_insert(
    snapshot: &UnionSaveSnapshot<'_>,
) -> Result<Vec<u8>, UnionBaseInsertBufferBlock> {
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

    let required_with_nul = sql.len() + 1;
    if required_with_nul > UNION_BASE_INSERT_CAPACITY {
        // BLOCKED_MISSING_FACT: `_sprintf` писал в `char[256]`; результат
        // переполнения и дальнейшего ExecuteCn не задан exact EXE.
        return Err(UnionBaseInsertBufferBlock {
            required_with_nul,
            capacity: UNION_BASE_INSERT_CAPACITY,
        });
    }
    Ok(sql)
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
