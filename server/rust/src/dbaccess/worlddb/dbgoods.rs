//! DB-владелец товаров WorldServer из `dbgoods.cpp`.
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.
//!
//! Контракт охватывает delete/load/save товара, base fields и addon properties.
//! SQL-порядок, signed форматирование legacy `unsigned long`, provider rows и
//! исходные значения отказа сохраняются. Caller-connection остаётся внешним;
//! Tiberius и typed goods snapshots заменяют ADO/COM и vararg buffers без
//! дополнительной транзакции, фильтра или перестановки частичных записей.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;

use encoding_rs::WINDOWS_1251;
use futures_util::TryStreamExt;
use tiberius::Query;

use crate::dbaccess::worlddb::goodslistener::{
    GoodsContainerTraversalSnapshot, GoodsListener, GoodsTraversalBlock,
};
use crate::dbaccess::worlddb::rssetup::{
    WorldDatabaseConnectionError, WorldDatabaseSettings, WorldTdsClient,
};
use crate::public::guid::CGuid;
use crate::worldserver::appworld::goods::cgoods::GoodsLoadedAddonBlock;
use crate::worldserver::appworld::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, create_goods_no_probability,
};
use crate::worldserver::appworld::player::{CPlayer, PlayerLoadedGoodsInsertBlock};

#[derive(Clone, Copy, Debug)]
pub(crate) struct GoodsAddonPropertyValue {
    pub(crate) id: u32,
    pub(crate) base_value: i32,
    pub(crate) modifier: i32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct GoodsAddonValueCountBlock {
    pub(crate) value_count: usize,
}

pub(crate) struct GoodsAddonPropertySnapshot {
    property_type: u32,
    occur_probability: u32,
    values: Vec<GoodsAddonPropertyValue>,
}

impl GoodsAddonPropertySnapshot {
 /// Создаёт только диапазон, в котором исходный `unsigned char` loop
 /// действительно достигал конца vector.
    pub(crate) fn from_legacy_parts(
        property_type: u32,
        occur_probability: u32,
        values: Vec<GoodsAddonPropertyValue>,
    ) -> Result<Self, GoodsAddonValueCountBlock> {
        if values.len() > u8::MAX as usize {
            return Err(GoodsAddonValueCountBlock {
                value_count: values.len(),
            });
        }
        Ok(Self {
            property_type,
            occur_probability,
            values,
        })
    }

 /// Возвращает исходные части property для другого DB-owner-а. Порядок
 /// values и все три 32-битных поля остаются исходным `tagAddonProperty`.
    pub(crate) fn legacy_parts(&self) -> (u32, u32, &[GoodsAddonPropertyValue]) {
        (self.property_type, self.occur_probability, &self.values)
    }
}

pub(crate) enum GoodsPropertiesSnapshot {
    Available(Vec<GoodsAddonPropertySnapshot>),
    MissingBaseProperties,
}

pub(crate) struct GoodsObjectSnapshot {
    pub(crate) goods_id: CGuid,
    pub(crate) base_properties_index: u32,
    pub(crate) name: Vec<u8>,
    pub(crate) price: u32,
    pub(crate) amount: u32,
    pub(crate) properties: GoodsPropertiesSnapshot,
}

pub(crate) struct GoodsSaveSnapshot<'goods> {
    pub(crate) player_id: i32,
    pub(crate) goods: &'goods GoodsObjectSnapshot,
    pub(crate) place: u8,
    pub(crate) position: u8,
}

#[derive(Debug)]
pub(crate) enum GoodsSaveBlock {
    GuidGeneration(getrandom::Error),
    EscapedNameBuffer { required_bytes: usize },
}

#[derive(Debug)]
pub(crate) enum GoodsSaveOutcome {
    Saved,
    Failed,
    BlockedMissingFact(GoodsSaveBlock),
}

pub(crate) struct PlayerGoodsFiledSnapshot<'snapshot> {
    pub(crate) player_id: i32,
    pub(crate) packet: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) equipment: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) hand: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) wallet: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) yuan_bao: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) ji_fen: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) bank: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) depot: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) fairy: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) battle_fairy: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) auction_goods: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) auction_wallet: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) auction: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) ci_qing: GoodsContainerTraversalSnapshot<'snapshot>,
    pub(crate) compose_ci_qing: GoodsContainerTraversalSnapshot<'snapshot>,
}

#[derive(Debug)]
pub(crate) enum GoodsFiledSaveOutcome {
    ReturnedTrue,
    ReturnedFalse,
    BlockedMissingFact(GoodsTraversalBlock),
}

#[derive(Debug)]
pub(crate) enum GoodsLoadFailure {
    Connection(WorldDatabaseConnectionError),
    Database {
        row_index: Option<usize>,
        source: tiberius::error::Error,
    },
    MissingRequiredValue {
        row_index: usize,
        column: &'static str,
    },
}

#[derive(Debug)]
pub(crate) enum GoodsLoadBlock {
    NameOutsideWindows1251 {
        row_index: usize,
    },
    GoodsGuid {
        row_index: usize,
        source: uuid::Error,
    },
    ChangedGuid {
        row_index: usize,
        source: getrandom::Error,
    },
    Addon {
        row_index: usize,
        source: GoodsLoadedAddonBlock,
    },
    Insert {
        row_index: usize,
        source: PlayerLoadedGoodsInsertBlock,
    },
}

#[derive(Debug)]
pub(crate) enum GoodsLoadOutcome {
    ReturnedTrue {
        row_count: usize,
        created_count: usize,
        skipped_count: usize,
    },
    ReturnedFalse(GoodsLoadFailure),
    BlockedMissingFact(GoodsLoadBlock),
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum DbGoodsOperation {
    DeleteGoods,
    SaveGoodsFiled,
    SaveGoods,
    SaveGoodsProperties,
}

#[derive(Debug)]
pub(crate) struct DbGoodsNotice {
    pub(crate) operation: DbGoodsOperation,
    pub(crate) error: DbGoodsSaveError,
}

#[derive(Debug)]
pub(crate) enum DbGoodsSaveError {
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
pub(crate) struct DbGoodsDatabaseError(tiberius::error::Error);

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

pub(crate) trait DbGoodsOwner {
    async fn load_goods(
        &mut self,
        player: &mut CPlayer,
        active_transaction: Option<&mut WorldTdsClient>,
        registry: &GoodsBasePropertiesRegistry,
        changed_goods_indices: &BTreeMap<u32, u32>,
        dakong_addon_types: &BTreeSet<i32>,
    ) -> GoodsLoadOutcome;

    async fn delete_goods(
        &mut self,
        player_id: i32,
        active_transaction: &mut WorldTdsClient,
    ) -> bool;

    async fn save_goods_properties(
        &mut self,
        properties: &[GoodsAddonPropertySnapshot],
        row_id: CGuid,
        active_transaction: &mut WorldTdsClient,
    ) -> bool;

    async fn save_goods(
        &mut self,
        snapshot: &GoodsSaveSnapshot<'_>,
        active_transaction: &mut WorldTdsClient,
    ) -> GoodsSaveOutcome;

    async fn save_goods_filed(
        &mut self,
        snapshot: &PlayerGoodsFiledSnapshot<'_>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> GoodsFiledSaveOutcome;

    fn pop_notice(&mut self) -> Option<DbGoodsNotice>;
}

pub(crate) struct TiberiusDbGoods {
    settings: WorldDatabaseSettings,
    notices: VecDeque<DbGoodsNotice>,
}

impl TiberiusDbGoods {
    pub(crate) fn new(settings: &WorldDatabaseSettings) -> Self {
        Self {
            settings: settings.clone(),
            notices: VecDeque::new(),
        }
    }
}

impl DbGoodsOwner for TiberiusDbGoods {
    async fn load_goods(
        &mut self,
        player: &mut CPlayer,
        active_transaction: Option<&mut WorldTdsClient>,
        registry: &GoodsBasePropertiesRegistry,
        changed_goods_indices: &BTreeMap<u32, u32>,
        dakong_addon_types: &BTreeSet<i32>,
    ) -> GoodsLoadOutcome {
        player.reset_goods_for_db_load();
        let mut standalone_connection;
        let active_transaction = match active_transaction {
            Some(active_transaction) => active_transaction,
            None => {
                standalone_connection = match self.settings.connect().await {
                    Ok(connection) => connection,
                    Err(source) => {
                        return GoodsLoadOutcome::ReturnedFalse(
                            GoodsLoadFailure::Connection(source),
                        );
                    }
                };
                &mut standalone_connection
            }
        };

        let mut query = Query::new(
            "SELECT CONVERT(varchar(38),a.GoodsID) AS GoodsID, a.goodsIndex, \
             a.name, a.price, a.amount, a.place, a.position, \
             CONVERT(int,b.type) AS type, b.modifierValue1, b.modifierValue2 \
             FROM player_goods AS a LEFT JOIN extend_properties AS b ON a.id=b.id \
             WHERE playerid=@P1 ORDER BY playerid",
        );
        query.bind(player.get_id());
        let mut rows = match query.query(&mut *active_transaction).await {
            Ok(rows) => rows,
            Err(source) => {
                return GoodsLoadOutcome::ReturnedFalse(GoodsLoadFailure::Database {
                    row_index: None,
                    source,
                });
            }
        };

        let mut row_index = 0usize;
        let mut created_count = 0usize;
        let mut skipped_count = 0usize;
        loop {
            let item = match rows.try_next().await {
                Ok(Some(item)) => item,
                Ok(None) => break,
                Err(source) => {
                    return GoodsLoadOutcome::ReturnedFalse(GoodsLoadFailure::Database {
                        row_index: Some(row_index),
                        source,
                    });
                }
            };
            let Some(row) = item.into_row() else {
                continue;
            };

            macro_rules! required {
                ($type:ty, $column:literal) => {
                    match row.try_get::<$type, _>($column) {
                        Ok(Some(value)) => value,
                        Ok(None) => {
                            return GoodsLoadOutcome::ReturnedFalse(
                                GoodsLoadFailure::MissingRequiredValue {
                                    row_index,
                                    column: $column,
                                },
                            );
                        }
                        Err(source) => {
                            return GoodsLoadOutcome::ReturnedFalse(
                                GoodsLoadFailure::Database {
                                    row_index: Some(row_index),
                                    source,
                                },
                            );
                        }
                    }
                };
            }

            let goods_guid_text = required!(&str, "GoodsID");
            let mut goods_guid = match CGuid::from_legacy_text(Some(goods_guid_text)) {
                Ok(guid) => guid,
                Err(source) => {
                    return GoodsLoadOutcome::BlockedMissingFact(GoodsLoadBlock::GoodsGuid {
                        row_index, source,
                    });
                }
            };
            let mut goods_index = required!(i32, "goodsIndex") as u32;
            let name = required!(&str, "name");
            let (name, _, name_had_errors) = WINDOWS_1251.encode(name);
            if name_had_errors {
                return GoodsLoadOutcome::BlockedMissingFact(
                    GoodsLoadBlock::NameOutsideWindows1251 { row_index },
                );
            }
            let price = required!(i32, "price") as u32;
            let amount = required!(i32, "amount") as u32;
            let place = required!(i32, "place");
            let position = required!(i32, "position") as u32;
            let addon_type = match row.try_get::<i32, _>("type") {
                Ok(value) => value,
                Err(source) => {
                    return GoodsLoadOutcome::ReturnedFalse(GoodsLoadFailure::Database {
                        row_index: Some(row_index),
                        source,
                    });
                }
            };
            let addon = if let Some(addon_type) = addon_type {
                Some((
                    addon_type,
                    required!(i32, "modifierValue1"),
                    required!(i32, "modifierValue2"),
                ))
            } else {
                None
            };

            if let Some(&replacement) = changed_goods_indices.get(&goods_index) {
                goods_index = replacement;
                goods_guid = match CGuid::create() {
                    Ok(guid) => guid,
                    Err(source) => {
                        return GoodsLoadOutcome::BlockedMissingFact(
                            GoodsLoadBlock::ChangedGuid { row_index, source },
                        );
                    }
                };
            }

            if let Some(goods) = player.loaded_goods_mut(place, position) {
                if let Some((property_type, first_modifier, second_modifier)) = addon
                    && let Err(source) = goods.apply_loaded_addon(
                        property_type,
                        first_modifier,
                        second_modifier,
                        registry,
                        dakong_addon_types,
                    )
                {
                    return GoodsLoadOutcome::BlockedMissingFact(GoodsLoadBlock::Addon {
                        row_index,
                        source,
                    });
                }
                row_index += 1;
                continue;
            }

            let Some(mut goods) = create_goods_no_probability(registry, goods_index) else {
                skipped_count += 1;
                row_index += 1;
                continue;
            };
            if let Some((property_type, first_modifier, second_modifier)) = addon
                && let Err(source) = goods.apply_loaded_addon(
                    property_type,
                    first_modifier,
                    second_modifier,
                    registry,
                    dakong_addon_types,
                )
            {
                return GoodsLoadOutcome::BlockedMissingFact(GoodsLoadBlock::Addon {
                    row_index,
                    source,
                });
            }
            goods.set_ex_id(&goods_guid);
            goods.set_name(&name);
            goods.set_price(price);
            goods.set_amount(amount);
            match player.insert_loaded_goods(place, position, goods, registry) {
                Ok(None) => created_count += 1,
                Ok(Some(_)) => skipped_count += 1,
                Err(source) => {
                    return GoodsLoadOutcome::BlockedMissingFact(GoodsLoadBlock::Insert {
                        row_index,
                        source,
                    });
                }
            }
            row_index += 1;
        }

        GoodsLoadOutcome::ReturnedTrue {
            row_count: row_index,
            created_count,
            skipped_count,
        }
    }

    async fn delete_goods(
        &mut self,
        player_id: i32,
        active_transaction: &mut WorldTdsClient,
    ) -> bool {
        let mut query = Query::new("DELETE FROM player_goods WHERE playerID=@P1");
        query.bind(player_id);
        match query.execute(active_transaction).await {
            Ok(_) => true,
            Err(error) => {
                self.notices.push_back(DbGoodsNotice {
                    operation: DbGoodsOperation::DeleteGoods,
                    error: DbGoodsSaveError::Database(error.into()),
                });
                false
            }
        }
    }

    async fn save_goods_properties(
        &mut self,
        properties: &[GoodsAddonPropertySnapshot],
        row_id: CGuid,
        active_transaction: &mut WorldTdsClient,
    ) -> bool {
        let row_id = row_id.to_string();
        let mut first_base = 0_i32;
        let mut first_modifier = 0_i32;
        let mut second_base = 0_i32;
        let mut second_modifier = 0_i32;

        for property in properties {
            for value in &property.values {
                match value.id {
                    1 => {
                        first_base = value.base_value;
                        first_modifier = value.modifier;
                    }
                    2 => {
                        second_base = value.base_value;
                        second_modifier = value.modifier;
                    }
                    _ => {}
                }
            }

            let modifier_value_2 = if property.property_type == 0x25 {
                if first_modifier == 0 && first_base == second_base {
                    continue;
                }
                second_base
            } else {
                if property.occur_probability == 10_000
                    && first_modifier == 0
                    && second_modifier == 0
                {
                    continue;
                }
                second_modifier
            };

            let mut query = Query::new(
                "INSERT INTO extend_properties(type,modifierValue1,modifierValue2,id) \
                 VALUES(@P1,@P2,@P3,@P4)",
            );
            query.bind(property.property_type as i32);
            query.bind(first_modifier);
            query.bind(modifier_value_2);
            query.bind(row_id.as_str());
            if let Err(error) = query.execute(&mut *active_transaction).await {
                self.notices.push_back(DbGoodsNotice {
                    operation: DbGoodsOperation::SaveGoodsProperties,
                    error: DbGoodsSaveError::Database(error.into()),
                });
                return false;
            }
        }
        true
    }

    async fn save_goods(
        &mut self,
        snapshot: &GoodsSaveSnapshot<'_>,
        active_transaction: &mut WorldTdsClient,
    ) -> GoodsSaveOutcome {
        let row_id = match CGuid::create() {
            Ok(row_id) => row_id,
            Err(error) => {
                return GoodsSaveOutcome::BlockedMissingFact(GoodsSaveBlock::GuidGeneration(error));
            }
        };
        let visible_name = visible_c_string(&snapshot.goods.name);
        let escaped_name = escape_single_quotes(visible_name);
        let escaped_required_bytes = escaped_name.len().saturating_add(1);
        if escaped_required_bytes > 256 {
            return GoodsSaveOutcome::BlockedMissingFact(GoodsSaveBlock::EscapedNameBuffer {
                required_bytes: escaped_required_bytes,
            });
        }
        let sql_required_bytes = legacy_goods_sql_required_bytes(snapshot, row_id, &escaped_name);
        debug_assert!(sql_required_bytes <= 499);

        if let Err(error) =
            insert_goods_row(snapshot, row_id, visible_name, active_transaction).await
        {
            self.notices.push_back(DbGoodsNotice {
                operation: DbGoodsOperation::SaveGoods,
                error: DbGoodsSaveError::Database(error.into()),
            });
            return GoodsSaveOutcome::Failed;
        }

        let properties = match &snapshot.goods.properties {
            GoodsPropertiesSnapshot::Available(properties) => properties,
            GoodsPropertiesSnapshot::MissingBaseProperties => {
                return GoodsSaveOutcome::Failed;
            }
        };
        if !properties.is_empty()
            && !self
                .save_goods_properties(properties, row_id, active_transaction)
                .await
        {
            self.notices.push_back(DbGoodsNotice {
                operation: DbGoodsOperation::SaveGoods,
                error: DbGoodsSaveError::NestedPropertiesFailed,
            });
            return GoodsSaveOutcome::Failed;
        }
        GoodsSaveOutcome::Saved
    }

    async fn save_goods_filed(
        &mut self,
        snapshot: &PlayerGoodsFiledSnapshot<'_>,
        active_transaction: Option<&mut WorldTdsClient>,
    ) -> GoodsFiledSaveOutcome {
        let Some(active_transaction) = active_transaction else {
            self.notices.push_back(DbGoodsNotice {
                operation: DbGoodsOperation::SaveGoodsFiled,
                error: DbGoodsSaveError::MissingConnection,
            });
            return GoodsFiledSaveOutcome::ReturnedFalse;
        };
        if !self
            .delete_goods(snapshot.player_id, &mut *active_transaction)
            .await
        {
 // WorldServer: `__snprintf(buf, 4,
 // "CDBGoods::SaveGoodsFiled():%d", player_id)` обрезал строку и
 // мог оставить её без NUL перед `AddErrorLogText`.
 // typed boundary: какие байты старый logger читал после
 // первых четырёх, не определёно; typed notice сохраняет сам факт
 // outer-ошибки без воспроизведения чтения за stack-buffer.
            self.notices.push_back(DbGoodsNotice {
                operation: DbGoodsOperation::SaveGoodsFiled,
                error: DbGoodsSaveError::NestedDeleteFailed,
            });
            return GoodsFiledSaveOutcome::ReturnedFalse;
        }

        let containers = [
            (1_u8, &snapshot.packet),
            (2, &snapshot.equipment),
            (3, &snapshot.hand),
            (4, &snapshot.wallet),
            (5, &snapshot.yuan_bao),
            (6, &snapshot.ji_fen),
            (7, &snapshot.bank),
            (8, &snapshot.depot),
            (9, &snapshot.fairy),
            (10, &snapshot.battle_fairy),
            (11, &snapshot.auction_goods),
            (12, &snapshot.auction_wallet),
            (13, &snapshot.auction),
            (14, &snapshot.ci_qing),
            (15, &snapshot.compose_ci_qing),
        ];
        let mut listener = GoodsListener::new(Some(snapshot.player_id), self, active_transaction);
        for (place, container) in containers {
            listener.set_place(place);
            if let Err(block) = container.traverse(&mut listener).await {
                return GoodsFiledSaveOutcome::BlockedMissingFact(block);
            }
        }
        GoodsFiledSaveOutcome::ReturnedTrue
    }

    fn pop_notice(&mut self) -> Option<DbGoodsNotice> {
        self.notices.pop_front()
    }
}

fn visible_c_string(bytes: &[u8]) -> &[u8] {
    bytes
        .iter()
        .position(|byte| *byte == 0)
        .map_or(bytes, |end| &bytes[..end])
}

fn escape_single_quotes(bytes: &[u8]) -> Vec<u8> {
    let mut escaped = Vec::with_capacity(
        bytes
            .len()
            .saturating_add(bytes.iter().filter(|byte| **byte == b'\'').count()),
    );
    for byte in bytes {
        if *byte == b'\'' {
            escaped.push(b'\'');
        }
        escaped.push(*byte);
    }
    escaped
}

fn legacy_goods_sql_required_bytes(
    snapshot: &GoodsSaveSnapshot<'_>,
    row_id: CGuid,
    escaped_name: &[u8],
) -> usize {
    let mut sql = Vec::with_capacity(256 + escaped_name.len());
    sql.extend_from_slice(b"INSERT INTO player_goods(id,GoodsID,goodsIndex,playerID,name,price,amount,place,position) \t\t\t VALUES('");
    sql.extend_from_slice(row_id.to_string().as_bytes());
    sql.extend_from_slice(b"','");
    sql.extend_from_slice(snapshot.goods.goods_id.to_string().as_bytes());
    sql.extend_from_slice(b"',");
    append_i32(&mut sql, snapshot.goods.base_properties_index as i32);
    sql.push(b',');
    append_i32(&mut sql, snapshot.player_id);
    sql.extend_from_slice(b",N'");
    sql.extend_from_slice(escaped_name);
    sql.extend_from_slice(b"',");
    append_i32(&mut sql, snapshot.goods.price as i32);
    sql.push(b',');
    append_i32(&mut sql, snapshot.goods.amount as i32);
    sql.push(b',');
    append_i32(&mut sql, i32::from(snapshot.place));
    sql.push(b',');
    append_i32(&mut sql, i32::from(snapshot.position));
    sql.push(b')');

    sql.len().saturating_add(1)
}

fn append_i32(target: &mut Vec<u8>, value: i32) {
    target.extend_from_slice(value.to_string().as_bytes());
}

async fn insert_goods_row(
    snapshot: &GoodsSaveSnapshot<'_>,
    row_id: CGuid,
    visible_name: &[u8],
    active_transaction: &mut WorldTdsClient,
) -> Result<(), tiberius::error::Error> {
    let (name, _, _) = WINDOWS_1251.decode(visible_name);
    let mut query = Query::new(
        "INSERT INTO player_goods(id,GoodsID,goodsIndex,playerID,name,price,amount,place,position) \
         VALUES(@P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9)",
    );
    query.bind(row_id.to_string());
    query.bind(snapshot.goods.goods_id.to_string());
    query.bind(snapshot.goods.base_properties_index as i32);
    query.bind(snapshot.player_id);
    query.bind(name.into_owned());
    query.bind(snapshot.goods.price as i32);
    query.bind(snapshot.goods.amount as i32);
    query.bind(snapshot.place);
    query.bind(snapshot.position);
    query.execute(active_transaction).await?;
    Ok(())
}
