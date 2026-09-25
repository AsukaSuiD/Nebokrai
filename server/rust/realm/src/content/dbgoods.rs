//! DB-owner товаров мира: трейт `DbGoodsOwner` и его data-семья, извлечённые
//! из старого `dbgoods`. Источник контракта — точная пара `Nworldserver.exe`
//! и `WorldServer.pdb`.
//!
//! Generic-параметр `PlayerT` заменяет прямую ссылку на игрока: реализация у
//! владельца DB-слоя делегируется по `CPlayer`. Block-типы ошибок вставки и
//! addon-применения остаются у владельца игрока, поэтому load-семья
//! параметризована `AddonBlockT`/`InsertBlockT` и связана с трейтом через
//! associated types. Будущее `load_goods` захватывает `&mut PlayerT` и
//! выполняется `block_on` в том же потоке, поэтому `Send` не требуется;
//! остальные методы захватывают только Sync-данные и помечены `+ Send`.

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;

use nebokrai_shared::values::{CGuid, GuidParseError};

use crate::content::goods::GoodsBasePropertiesRegistry;
use crate::content::goodsdb::{GoodsAddonPropertySnapshot, GoodsObjectSnapshot};
use crate::content::goodslistener::{GoodsContainerTraversalSnapshot, GoodsTraversalBlock};
use crate::persistence::rssetup::{WorldDatabaseConnectionError, WorldTdsClient};

pub struct GoodsSaveSnapshot<'goods> {
    pub player_id: i32,
    pub goods: &'goods GoodsObjectSnapshot,
    pub place: u8,
    pub position: u8,
}

#[derive(Debug)]
pub enum GoodsSaveBlock {
    GuidGeneration(getrandom::Error),
    EscapedNameBuffer { required_bytes: usize },
}

#[derive(Debug)]
pub enum GoodsSaveOutcome {
    Saved,
    Failed,
    BlockedMissingFact(GoodsSaveBlock),
}

pub struct PlayerGoodsFiledSnapshot<'snapshot> {
    pub player_id: i32,
    pub packet: GoodsContainerTraversalSnapshot<'snapshot>,
    pub equipment: GoodsContainerTraversalSnapshot<'snapshot>,
    pub hand: GoodsContainerTraversalSnapshot<'snapshot>,
    pub wallet: GoodsContainerTraversalSnapshot<'snapshot>,
    pub yuan_bao: GoodsContainerTraversalSnapshot<'snapshot>,
    pub ji_fen: GoodsContainerTraversalSnapshot<'snapshot>,
    pub bank: GoodsContainerTraversalSnapshot<'snapshot>,
    pub depot: GoodsContainerTraversalSnapshot<'snapshot>,
    pub fairy: GoodsContainerTraversalSnapshot<'snapshot>,
    pub battle_fairy: GoodsContainerTraversalSnapshot<'snapshot>,
    pub auction_goods: GoodsContainerTraversalSnapshot<'snapshot>,
    pub auction_wallet: GoodsContainerTraversalSnapshot<'snapshot>,
    pub auction: GoodsContainerTraversalSnapshot<'snapshot>,
    pub ci_qing: GoodsContainerTraversalSnapshot<'snapshot>,
    pub compose_ci_qing: GoodsContainerTraversalSnapshot<'snapshot>,
}

#[derive(Debug)]
pub enum GoodsFiledSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(GoodsTraversalBlock),
}

#[derive(Debug)]
pub enum GoodsLoadFailure {
    Connection(WorldDatabaseConnectionError),
    Database {
        row_index: Option<usize>,
        source: tiberius::error::Error,
    },
    MissingRequiredValue {
        row_index: usize,
        column: &'static str,
    },
    NumericOutsideLegacyRange {
        row_index: usize,
        column: &'static str,
    },
}

#[derive(Debug)]
pub enum GoodsLoadBlock<AddonBlockT, InsertBlockT> {
    NameOutsideWindows1251 {
        row_index: usize,
    },
    GoodsGuid {
        row_index: usize,
        source: GuidParseError,
    },
    ChangedGuid {
        row_index: usize,
        source: getrandom::Error,
    },
    Addon {
        row_index: usize,
        source: AddonBlockT,
    },
    Insert {
        row_index: usize,
        source: InsertBlockT,
    },
}

#[derive(Debug)]
pub enum GoodsLoadOutcome<AddonBlockT, InsertBlockT> {
    ReturnedTrue {
        row_count: usize,
        created_count: usize,
        skipped_count: usize,
    },
    ReturnedFalse(GoodsLoadFailure),
    BlockedMissingFact(GoodsLoadBlock<AddonBlockT, InsertBlockT>),
}

#[derive(Clone, Copy, Debug)]
pub enum DbGoodsOperation {
    DeleteGoods,
    SaveGoodsFiled,
    SaveGoods,
    SaveGoodsProperties,
}

#[derive(Debug)]
pub struct DbGoodsNotice {
    pub operation: DbGoodsOperation,
    pub error: DbGoodsSaveError,
}

#[derive(Debug)]
pub enum DbGoodsSaveError {
    Database(DbGoodsDatabaseError),
    MissingConnection,
    NestedDeleteFailed,
    NestedPropertiesFailed,
}

impl fmt::Display for DbGoodsSaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Database(error) => error.fmt(formatter),
            Self::MissingConnection => write!(formatter, "не передано соединение World goods DB"),
            Self::NestedDeleteFailed => {
                write!(
                    formatter,
                    "предварительное удаление вещей завершилось ошибкой"
                )
            }
            Self::NestedPropertiesFailed => {
                write!(formatter, "сохранение addon properties завершилось ошибкой")
            }
        }
    }
}

impl Error for DbGoodsSaveError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Database(error) => Some(error),
            Self::MissingConnection | Self::NestedDeleteFailed | Self::NestedPropertiesFailed => {
                None
            }
        }
    }
}

#[derive(Debug)]
pub struct DbGoodsDatabaseError(tiberius::error::Error);

impl fmt::Display for DbGoodsDatabaseError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "ошибка TDS World goods DB: {}", self.0)
    }
}

impl Error for DbGoodsDatabaseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(&self.0)
    }
}

impl From<tiberius::error::Error> for DbGoodsDatabaseError {
    fn from(error: tiberius::error::Error) -> Self {
        Self(error)
    }
}

pub trait DbGoodsOwner<PlayerT> {
    type AddonBlock;
    type InsertBlock;

    fn load_goods(
        &mut self,
        player: &mut PlayerT,
        active_transaction: Option<&mut WorldTdsClient>,
        registry: &GoodsBasePropertiesRegistry,
        changed_goods_indices: &BTreeMap<u32, u32>,
        dakong_addon_types: &BTreeSet<i32>,
    ) -> impl std::future::Future<Output = GoodsLoadOutcome<Self::AddonBlock, Self::InsertBlock>>;

    fn delete_goods(
        &mut self,
        player_id: i32,
        active_transaction: &mut WorldTdsClient,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn save_goods_properties(
        &mut self,
        properties: &[GoodsAddonPropertySnapshot],
        row_id: CGuid,
        active_transaction: &mut WorldTdsClient,
    ) -> impl std::future::Future<Output = bool> + Send;

    fn save_goods(
        &mut self,
        snapshot: &GoodsSaveSnapshot<'_>,
        active_transaction: &mut WorldTdsClient,
    ) -> impl std::future::Future<Output = GoodsSaveOutcome> + Send;

    fn save_goods_filed(
        &mut self,
        snapshot: &PlayerGoodsFiledSnapshot<'_>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> impl std::future::Future<Output = GoodsFiledSaveOutcome> + Send;

    fn pop_notice(&mut self) -> Option<DbGoodsNotice>;
}
