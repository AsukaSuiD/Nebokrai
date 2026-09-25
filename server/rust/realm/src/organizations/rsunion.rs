//! World DB-владелец `CRsUnion` из `rsunion.cpp`, подтверждённый
//! `Nworldserver.exe` и `WorldServer.pdb`. Трейт `RsUnionOwner` и его
//! data-семья перенесены в Realm `organizations/`; Tiberius-реализация
//! остаётся в старом пакете (`dbaccess/worlddb/rsunion`) и там же
//! реэкспортирует этот модуль.
//!
//! Owner сохраняет отдельные delete/save/load операции, ordered membership,
//! исходные signed IDs и false для missing connection/catch. Tiberius и owned
//! values заменяют ADO/COM и nullable pointers без изменения SQL-порядка,
//! partial effects или наблюдаемых результатов.
//!
//! Async-методы трейта записаны в desugared-форме по ADR-0013. Каждое future
//! захватывает только `&mut self`, Sync-ссылки `UnionSaveSnapshot` и
//! Send-соединение `&mut WorldTdsClient`, поэтому все методы помечены
//! `+ Send`.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::content::organizing::{TagMemInfo, UnterminatedMemberField};
use crate::organizations::union::CUnion;
use crate::persistence::rssetup::{WorldDatabaseConnectionError, WorldTdsClient};

/// Безопасный результат одного base-row `LoadAllConfederation` до публикации
/// concrete `CUnion` у следующего owner-а.
pub struct UnionDatabaseLoadRecord {
    pub union_id: i32,
    pub name: Vec<u8>,
    pub master_id: i32,
    pub members: BTreeMap<i32, TagMemInfo>,
}

/// Результат `LoadAllConfederation`; `records` всегда содержит уже завершённый
/// prefix, как исходные записи в controller-map до первого отказа.
///
/// Owner возвращает `int`, а не `bool`, и увеличивает local counter
/// до чтения каждого base-row. Поэтому `reported_count` может на единицу
/// опережать `records.len()` на отказавшем base/member-row — именно это число
/// попадает в последующий `Initialize` log.
pub enum UnionLoadOutcome {
    ReturnedTrue {
        records: Vec<UnionDatabaseLoadRecord>,
        reported_count: i32,
    },
    ReturnedFalse {
        records: Vec<UnionDatabaseLoadRecord>,
        reported_count: i32,
    },
}

pub struct UnionSaveSnapshot<'union> {
    pub union_id: i32,
    pub name: &'union [u8],
    pub master_id: i32,
    pub members: &'union BTreeMap<i32, TagMemInfo>,
}

impl<'union> UnionSaveSnapshot<'union> {
    pub fn from_union(union: &'union CUnion) -> Self {
        Self {
            union_id: union.union_id(),
            name: union.name(),
            master_id: union.master_id(),
            members: union.members(),
        }
    }
}

#[derive(Debug)]
pub enum UnionSaveBlock {
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

#[derive(Debug)]
pub enum UnionMembersSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(UnterminatedMemberField),
}

#[derive(Debug)]
pub enum UnionSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(UnionSaveBlock),
}

#[derive(Debug)]
pub struct RsUnionNotice {
    pub operation: RsUnionOperation,
    pub error: RsUnionSaveError,
}

#[derive(Clone, Copy, Debug)]
pub enum RsUnionOperation {
    LoadAllConfederation,
    LoadConfederationMembers,
    LoadUnionMemberInfo,
    SaveConfederation,
    DeleteConfederation,
    DeleteConfederationMembers,
    SaveConfederationMember,
}

#[derive(Debug)]
pub enum RsUnionSaveError {
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
pub struct RsUnionDatabaseError(tiberius::error::Error);

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

pub trait RsUnionOwner {
 /// Открывает самостоятельное World DB connection и возвращает готовый
 /// prefix base/member rows в точном исходном порядке.
    fn load_all_confederations(
        &mut self,
    ) -> impl std::future::Future<Output = UnionLoadOutcome> + Send;

    fn save_confederation(
        &mut self,
        snapshot: Option<&UnionSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = UnionSaveOutcome> + Send;

    fn save_confe_members(
        &mut self,
        snapshot: Option<&UnionSaveSnapshot<'_>>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = UnionMembersSaveOutcome> + Send;

    fn del_confederation(
        &mut self,
        union_id: i32,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn pop_notice(&mut self) -> Option<RsUnionNotice>;
}
