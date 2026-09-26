//! DB-владелец аукционных очередей WorldServer из `dbmisc.cpp/.h`, перенесённый в Realm `persistence/`.
//! Источник контракта — точная пара `worldserver.exe` и `worldserver.pdb`.
//!
//! Две очереди `DbNote` и player FIFO сохраняют раздельные locks, выбор
//! head/tail, правило limit `0 = весь batch`, отрицательный limit `= ничего`
//! и максимум восемь записей в `DoneOutList`. Старый bool queue helpers всегда
//! был `false`; Rust возвращает перемещённый batch, не меняя порядок записей.
//!
//! Terminal dispatch сохраняет opcodes, повторный lookup сервера по player ID
//! и различающиеся A2S/A2B error-типы. Пустой goods payload остаётся typed
//! границей после уже выполненной отправки. Tiberius заменяет ADO/COM и ручное
//! владение, но не прячет async I/O, не объединяет соединения и не меняет SQL,
//! FIFO, partial effects или порядок callback-ов.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::error::Error;
use std::fmt;
use std::sync::Arc;

use encoding_rs::WINDOWS_1251;
use futures_util::TryStreamExt;
use parking_lot::Mutex;
use tiberius::Query;

use crate::persistence::rssetup::{
    WorldDatabaseConnectionError, WorldDatabaseSettings, WorldTdsClient,
};
use crate::content::goodsdb::{GoodsAddonPropertySnapshot, GoodsPropertiesSnapshot};
use crate::app::world_message::CMessage;
use crate::auction::auctionnode::{
    AuctionDatabaseInsertFields, AuctionDatabaseNodeFields, AuctionDatabaseWriteFields, CGoodsNode,
    GoodsNodeSerializeError, GoodsState,
};
use nebokrai_shared::resources::CDaKongXiangQian;
use nebokrai_shared::values::{CGuid, GuidParseError};
use crate::content::cgoods::{
    CGoods, GoodsCodecError, GoodsDbSnapshotBlock, GoodsLoadedAddonBlock,
};
use crate::content::cgoodsfactory::{create_goods, create_goods_no_probability};
use crate::content::goods::GoodsBasePropertiesRegistry;

const OUTPUT_BATCH_LIMIT: i32 = 8;
const AUCTION_MAP_ID: u32 = 5;
const MSG_AUCTION_NODE: i32 = 0x0014_ED01;
const MSG_AUCTION_RETURN: i32 = 0x0008_0404;
const MSG_AUCTION_STATE_CHANGED: i32 = 0x0008_0405;
const LOAD_AUCTION_STATE: i32 = 1;
const LOAD_AUCTION_LIMIT: i32 = 100_000;

/// Внутренний discriminant исходного `CDbMisc::OperatorType`.
///
/// Значения зафиксированы по машинным записям `Nworldserver.exe`: входная
/// запись ModifyState A2B пишется константой `7` в обоих auction-диспетчерах
/// (RVA `0xA5857` — server-auction, RVA `0xA52BF` — misc-auction), входная
/// `OT_IN_MODIFY_STATE_A2S` — `4`, входная `OT_IN_INSERT_NEW_ITEM` — `1`
/// (VERIFIED досверкой диспетчеров той же точной пары). При последовательной
/// семантике MSVC-enum между `OT_OUT_MODIFY_STATE_A2S_OK` (5) и
/// `OT_IN_MODIFY_STATE_A2B` (7) у исходного enum объявлен член 6, которому ни
/// одна машинная запись значения не известна. Имя — INFERRED по симметрии
/// семьи (`OT_OUT_MODIFY_STATE_A2S_ERROR` между двумя triad-группами
/// INSERT 1..3 и A2B 7..9): живой отказ A2S в `DoneListIn` вместо него
/// публикует чужой `OT_OUT_INSERT_NEW_ITEM_ERROR` (ветка ниже в этом файле),
/// поэтому член 6 — мёртвый объявленный discriminant без наблюдаемой записи.
/// Хвост enum (после машинной точки 7) снят досверкой писателей и потребителей
/// `Nworldserver.exe` той же точной пары: 10 = delete-item-успех (producer RVA
/// `0xF2B1D`, consumer → DelItemFromDb `0xF364E`), 13 = delete-item-back
/// (consumer-ветвь `0xF36D7`; producer не наблюдается — имя INFERRED), 16 =
/// read-auction-result (producers `0xF53D9`/`0xF1FC1`, consumer — ветвь idx14
/// bytemap `0xF2BF8`), 19 = modify-money (producer `0xF2B28`, consumer →
/// DelMoneyFromDb `0xF375D`; поля note player/money подтверждены). Членов
/// 11/12/14/15/17/18 машинная запись не знает — UNKNOWN и не объявляются.
/// Фантомный член «read auction» (прежнее значение 10) удалён: машинного
/// note-входа у этой операции нет (обработчик вызывается напрямую), живых
/// write-site в Rust не было; значение 10 занято delete-item.
///
/// Число не выходит в wire и здесь намеренно не переиспользуется как protocol
/// ID: наблюдаемым контрактом являются названные переходы между очередями.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OperatorType {
    OT_NULL = 0,
    OT_IN_INSERT_NEW_ITEM = 1,
    OT_OUT_INSERT_NEW_ITEM_OK = 2,
    OT_OUT_INSERT_NEW_ITEM_ERROR = 3,
    OT_IN_MODIFY_STATE_A2S = 4,
    OT_OUT_MODIFY_STATE_A2S_OK = 5,
 /// Мёртвый член 6 исходного enum; имя — INFERRED (см. doc enum выше).
    OT_OUT_MODIFY_STATE_A2S_ERROR = 6,
 /// Машинная точка 7 исходного enum: RVA `0xA5857`/`0xA52BF` (VERIFIED).
    OT_IN_MODIFY_STATE_A2B = 7,
    OT_OUT_MODIFY_STATE_A2B_OK = 8,
    OT_OUT_MODIFY_STATE_A2B_ERROR = 9,
 /// Машинная точка 10: producer RVA `0xF2B1D`; consumer → DelItemFromDb `0xF364E`.
    OT_IN_DELETE_ITEM_SUCESS = 10,
 /// Число 13 MATCH (consumer-ветвь `0xF36D7`); имя — INFERRED (producer не наблюдается).
    OT_IN_DELETE_ITEM_BACK = 13,
 /// Машинная точка 16: producers `0xF53D9`/`0xF1FC1`; consumer — ветвь bytemap `0xF2BF8`.
    OT_OUT_READ_AUCTION_RESULT = 16,
 /// Машинная точка 19: producer `0xF2B28`; consumer → DelMoneyFromDb `0xF375D`.
    OT_IN_MODIFY_MONEY = 19,
}

pub struct DbNote {
    pub e_type: OperatorType,
    pub goods: CGoodsNode,
    pub player_id: i32,
    pub money: i32,
}

impl Default for DbNote {
    fn default() -> Self {
        Self::new()
    }
}

impl DbNote {
    pub fn new() -> Self {
        Self {
            e_type: OperatorType::OT_NULL,
            goods: CGoodsNode::new(),
            player_id: -1,
            money: 0,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DbMiscGameServer {
    pub connected: bool,
    pub index: u32,
}

/// Живые World-owner-ы, необходимые только при доставке готового DB-результата.
///
/// Эта граница намеренно отделена от TDS-контекста: `DoneOutList` должен видеть
/// актуальные player/GameServer registries того же `CGame`, но DB-owner не
/// должен удерживать ссылку на game в течение всего `MainLoop`.
pub trait DbMiscDeliveryContext {
    fn player_game_server(&mut self, player_id: u32) -> Option<DbMiscGameServer>;
    fn online_player_id(&mut self, player_id: u32) -> Option<i32>;
    fn send_to_map_id(&mut self, message: &CMessage, map_id: u32);
    fn log_player_not_online_drop_goods(&mut self);
    fn gold_coin_index(&mut self) -> u32;
}

pub trait DbMiscContext {
    fn transfer_money_interval_ms(&mut self) -> i32;
    fn current_tick_ms(&mut self) -> u32;
    fn is_active_connect(&mut self) -> bool;
    fn log_connect_error_and_reconnect(&mut self);
    fn create_normal_connection(&mut self);

    fn insert_item_to_db(&mut self, goods: &CGoodsNode) -> bool;
    fn modify_goods_state_a2s(&mut self, goods: &CGoodsNode) -> bool;
    fn transfer_money(&mut self, goods: &CGoodsNode) -> bool;
    fn modify_goods_state_a2b(&mut self, goods: &CGoodsNode) -> bool;
    fn delete_item_from_db(&mut self, guid: CGuid);
    fn delete_money_from_db(&mut self, player_id: i32, money: i32);

    fn gold_coin_index(&mut self) -> u32;

 /// Заполняет destination в SQL recordset-order из
 /// `SELECT distinct dwowerid FROM Auction (nolock)`.
    fn read_auction_owner_ids(&mut self, destination: &mut VecDeque<i32>);
    fn load_goods_by_owner_id(&mut self, owner_id: i32, state: i32, limit: i32);

 /// Выполняет точный owner-read `Auction` и публикует созданные
 /// `OT_OUT_READ_AUCTION_RESULT` notes в output FIFO. Возвращает число
 /// товаров, а не строк join-а `AuctionGoods`.
    fn load_owner_auction_goods(&mut self, owner_id: i32, state: i32, limit: i32) -> i32;

 /// Выполняет `LoadMoneyById`: создаёт и публикует возврат gold only при
 /// ненулевом `AuctionPlayerMoney.dwmoney`; результат чтения намеренно не
 /// участвует в S2W control-flow оригинала.
    fn load_owner_auction_money(&mut self, owner_id: i32, money_limit: i32);
}

#[derive(Debug)]
pub enum DbMiscDoneOutBlockReason {
    GoodsNodeSerialize(GoodsNodeSerializeError),
    GoodsDecode(GoodsCodecError),
    GoodsSerialize(GoodsCodecError),
    EmptyGoodsBaseIndexAfterDelivery,
}

pub struct DbMiscDoneOutBlock {
    pub reason: DbMiscDoneOutBlockReason,
    pub processed_notes: usize,
    pub pending_notes: VecDeque<Box<DbNote>>,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DbMiscDoneOutReport {
    pub processed_notes: usize,
    pub auction_node_deliveries: usize,
    pub state_change_deliveries: usize,
    pub returned_goods_deliveries: usize,
    pub requeued_input_notes: usize,
    pub dropped_offline_goods: usize,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DbMiscDoneInReport {
    pub processed_notes: usize,
    pub output_notes: usize,
    pub requeued_input_notes: usize,
    pub deleted_items: usize,
    pub modified_money: usize,
    pub unhandled_notes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DbMiscLoadAuctionReport {
    LoadedOwner { owner_id: i32 },
    RefilledOwners { count: usize },
}

pub struct CDbMisc {
    input: Mutex<VecDeque<Box<DbNote>>>,
    output: Arc<Mutex<VecDeque<Box<DbNote>>>>,
    player_ids: Mutex<VecDeque<i32>>,
    auction_batch: VecDeque<i32>,
    start_read_player_list: bool,
    transfer_money_previous_ms: u32,
}

/// Клонируемый producer единственной output FIFO аукциона.
///
/// Concrete Tiberius bridge получает его до MainLoop-вызова и публикует
/// полностью materialized DB batch под тем же mutex. Сам `CDbMisc` остаётся
/// единственным consumer-ом и сохраняет исходный порядок `DoneOutList`.
#[derive(Clone)]
pub struct DbMiscOutputPublisher {
    output: Arc<Mutex<VecDeque<Box<DbNote>>>>,
}

impl DbMiscOutputPublisher {
    fn append(&self, notes: &mut VecDeque<Box<DbNote>>) {
        self.output.lock().append(notes);
    }
}

impl CDbMisc {
    pub fn new(context: &mut impl DbMiscContext) -> Self {
        let owner = Self::with_empty_queues();
        context.create_normal_connection();
        owner
    }

 /// Создаёт queue-owner до process-level сборки concrete context-а.
 /// Connection по-прежнему обязан быть создан в позиции DB-owner-а Init.
    pub fn with_empty_queues() -> Self {
        Self {
            input: Mutex::new(VecDeque::new()),
            output: Arc::new(Mutex::new(VecDeque::new())),
            player_ids: Mutex::new(VecDeque::new()),
            auction_batch: VecDeque::new(),
            start_read_player_list: false,
            transfer_money_previous_ms: 0,
        }
    }

    pub fn initialize_database(&self, context: &mut impl DbMiscContext) {
        context.create_normal_connection();
    }

    pub fn output_publisher(&self) -> DbMiscOutputPublisher {
        DbMiscOutputPublisher {
            output: Arc::clone(&self.output),
        }
    }

    pub fn push_item_to_list_in(&self, note: Box<DbNote>, push_front: bool) -> bool {
        let mut input = self.input.lock();
        if push_front {
            input.push_front(note);
        } else {
            input.push_back(note);
        }
        false
    }

    pub fn push_item_to_list_out(&self, note: Box<DbNote>) -> bool {
        self.output.lock().push_back(note);
        false
    }

    pub fn pop_item_from_list_in(
        &mut self,
        context: &mut impl DbMiscContext,
        limit: i32,
    ) -> VecDeque<Box<DbNote>> {
        let interval = context.transfer_money_interval_ms();
        let should_check_connection = if interval == 0 {
            true
        } else {
            let current = context.current_tick_ms();
            if current.wrapping_sub(self.transfer_money_previous_ms) <= interval as u32 {
                false
            } else {
                self.transfer_money_previous_ms = current;
                true
            }
        };

        if should_check_connection && !context.is_active_connect() {
            context.log_connect_error_and_reconnect();
            context.create_normal_connection();
            return VecDeque::new();
        }
        move_queue_batch(&mut self.input.lock(), limit)
    }

    pub fn pop_item_from_list_out(&self, limit: i32) -> VecDeque<Box<DbNote>> {
        move_queue_batch(&mut self.output.lock(), limit)
    }

    pub fn done_list_in(
        &self,
        context: &mut impl DbMiscContext,
        mut notes: VecDeque<Box<DbNote>>,
    ) -> DbMiscDoneInReport {
        let mut report = DbMiscDoneInReport::default();
        while let Some(mut note) = notes.pop_front() {
            report.processed_notes += 1;
            match note.e_type {
                OperatorType::OT_IN_INSERT_NEW_ITEM => {
                    note.e_type = if context.insert_item_to_db(&note.goods) {
                        OperatorType::OT_OUT_INSERT_NEW_ITEM_OK
                    } else {
                        OperatorType::OT_OUT_INSERT_NEW_ITEM_ERROR
                    };
                    let _ = self.push_item_to_list_out(note);
                    report.output_notes += 1;
                }
                OperatorType::OT_IN_MODIFY_STATE_A2S => {
                    let succeeded = context.modify_goods_state_a2s(&note.goods)
                        && context.transfer_money(&note.goods);
                    note.e_type = if succeeded {
                        OperatorType::OT_OUT_MODIFY_STATE_A2S_OK
                    } else {
 // World действительно использует чужой
                    // ветку ошибки INSERT: собственный член 6
                    // (`OT_OUT_MODIFY_STATE_A2S_ERROR`) при этом объявлен
                    // в исходном enum, но нигде не записывается (см. doc enum).
                        OperatorType::OT_OUT_INSERT_NEW_ITEM_ERROR
                    };
                    let _ = self.push_item_to_list_out(note);
                    report.output_notes += 1;
                }
                OperatorType::OT_IN_MODIFY_STATE_A2B => {
                    if context.modify_goods_state_a2b(&note.goods) {
                        note.e_type = OperatorType::OT_OUT_MODIFY_STATE_A2B_OK;
                        let _ = self.push_item_to_list_out(note);
                        report.output_notes += 1;
                    } else {
                        note.e_type = OperatorType::OT_OUT_MODIFY_STATE_A2B_ERROR;
                        let _ = self.push_item_to_list_in(note, false);
                        report.requeued_input_notes += 1;
                    }
                }
                OperatorType::OT_IN_DELETE_ITEM_SUCESS | OperatorType::OT_IN_DELETE_ITEM_BACK => {
                    context.delete_item_from_db(note.goods.guid());
                    report.deleted_items += 1;
                }
                OperatorType::OT_IN_MODIFY_MONEY => {
                    context.delete_money_from_db(note.player_id, note.money);
                    report.modified_money += 1;
                }
                _ => {
 // Неизвестный discriminant сразу переходит к общей итерации.
                    report.unhandled_notes += 1;
                }
            }
        }
        report
    }

    pub fn done_out_list(
        &self,
        context: &mut impl DbMiscDeliveryContext,
    ) -> Result<DbMiscDoneOutReport, DbMiscDoneOutBlock> {
        let mut notes = self.pop_item_from_list_out(OUTPUT_BATCH_LIMIT);
        let mut report = DbMiscDoneOutReport::default();

        while let Some(mut note) = notes.pop_front() {
            match note.e_type {
                OperatorType::OT_OUT_INSERT_NEW_ITEM_OK => {
                    let bytes = match note.goods.serialize() {
                        Ok(bytes) => bytes,
                        Err(error) => {
                            notes.push_front(note);
                            return Err(output_block(
                                DbMiscDoneOutBlockReason::GoodsNodeSerialize(error),
                                report.processed_notes,
                                notes,
                            ));
                        }
                    };
                    let mut message = CMessage::new(MSG_AUCTION_NODE);
                    message.base_mut().add(&bytes);
                    context.send_to_map_id(&message, AUCTION_MAP_ID);
                    report.auction_node_deliveries += 1;
                }
                OperatorType::OT_OUT_MODIFY_STATE_A2S_OK
                | OperatorType::OT_OUT_MODIFY_STATE_A2B_OK => {
                    if let Some(server) = context.player_game_server(note.goods.owner_id())
                        && server.connected
                    {
                        let mut message = CMessage::new(MSG_AUCTION_STATE_CHANGED);
                        message.base_mut().add_ulong(note.goods.owner_id());
                        context.send_to_map_id(&message, server.index);
                        report.state_change_deliveries += 1;
                    }
                }
                OperatorType::OT_OUT_READ_AUCTION_RESULT => {
                    let state = note.goods.goods_state();
                    if state == GoodsState::AUCTION {
                        let bytes = match note.goods.serialize() {
                            Ok(bytes) => bytes,
                            Err(error) => {
                                notes.push_front(note);
                                return Err(output_block(
                                    DbMiscDoneOutBlockReason::GoodsNodeSerialize(error),
                                    report.processed_notes,
                                    notes,
                                ));
                            }
                        };
                        let mut message = CMessage::new(MSG_AUCTION_NODE);
                        message.base_mut().add(&bytes);
                        context.send_to_map_id(&message, AUCTION_MAP_ID);
                        report.auction_node_deliveries += 1;
                    } else if matches!(
                        state,
                        GoodsState::BACK | GoodsState::SUCESSED | GoodsState::UNDO
                    ) {
                        let Some(first_server) = context.player_game_server(note.goods.owner_id())
                        else {
                            report.processed_notes += 1;
                            continue;
                        };
                        if !first_server.connected {
                            report.processed_notes += 1;
                            continue;
                        }
                        let Some(player_id) = context.online_player_id(note.goods.owner_id())
                        else {
                            context.log_player_not_online_drop_goods();
                            report.dropped_offline_goods += 1;
                            report.processed_notes += 1;
                            continue;
                        };
                        let Some(actual_server) = context.player_game_server(player_id as u32)
                        else {
                            report.processed_notes += 1;
                            continue;
                        };
                        if !actual_server.connected {
                            report.processed_notes += 1;
                            continue;
                        }

                        let mut serialized_goods = Vec::new();
                        let mut base_properties_index = None;
                        if !note.goods.goods_bytes().is_empty() {
                            let mut goods = CGoods::with_constructor_base_and_type();
                            let mut cursor = 0;
                            if let Err(error) =
                                goods.unserialize(note.goods.goods_bytes(), &mut cursor, true)
                            {
                                notes.push_front(note);
                                return Err(output_block(
                                    DbMiscDoneOutBlockReason::GoodsDecode(error),
                                    report.processed_notes,
                                    notes,
                                ));
                            }
                            if let Err(error) = goods.serialize(&mut serialized_goods, true) {
                                notes.push_front(note);
                                return Err(output_block(
                                    DbMiscDoneOutBlockReason::GoodsSerialize(error),
                                    report.processed_notes,
                                    notes,
                                ));
                            }
                            base_properties_index = goods.get_base_properties_index();
                        }

                        let mut message = CMessage::new(MSG_AUCTION_RETURN);
                        message.base_mut().add_long(player_id);
                        message.base_mut().add_guid(note.goods.guid());
                        message.base_mut().add_ulong(note.goods.base_index());
                        message.base_mut().add_long(state.raw());
                        message
                            .base_mut()
                            .add_long(serialized_goods.len() as u32 as i32);
                        message.base_mut().add(&serialized_goods);
                        message.base_mut().update();
                        context.send_to_map_id(&message, actual_server.index);
                        report.returned_goods_deliveries += 1;

                        let Some(base_properties_index) = base_properties_index else {
                            notes.push_front(note);
                            return Err(output_block(
                                DbMiscDoneOutBlockReason::EmptyGoodsBaseIndexAfterDelivery,
                                report.processed_notes,
                                notes,
                            ));
                        };
                        if base_properties_index == context.gold_coin_index() {
                            note.e_type = OperatorType::OT_IN_MODIFY_MONEY;
                            note.player_id = player_id;
                            note.money = note.goods.amount();
                        } else {
                            note.e_type = OperatorType::OT_IN_DELETE_ITEM_SUCESS;
                        }
                        let _ = self.push_item_to_list_in(note, false);
                        report.requeued_input_notes += 1;
                    }
                }
                _ => {}
            }
            report.processed_notes += 1;
        }
        Ok(report)
    }

    pub fn done_ot_in_read_auction(&self, context: &mut impl DbMiscContext) {
        if !context.is_active_connect() {
            return;
        }
        let mut player_ids = self.player_ids.lock();
        player_ids.clear();
        context.read_auction_owner_ids(&mut player_ids);
    }

    pub fn pop_player_list(&self, limit: i32) -> VecDeque<i32> {
        move_queue_batch(&mut self.player_ids.lock(), limit)
    }

    pub fn load_auction(
        &mut self,
        context: &mut impl DbMiscContext,
    ) -> DbMiscLoadAuctionReport {
        if self.start_read_player_list {
            self.done_ot_in_read_auction(context);
            self.start_read_player_list = false;
            self.auction_batch.clear();
        }

        if let Some(owner_id) = self.auction_batch.pop_front() {
            context.load_goods_by_owner_id(owner_id, LOAD_AUCTION_STATE, LOAD_AUCTION_LIMIT);
            return DbMiscLoadAuctionReport::LoadedOwner { owner_id };
        }

        self.auction_batch.clear();
        let mut player_ids = self.pop_player_list(0);
        self.auction_batch.append(&mut player_ids);
        DbMiscLoadAuctionReport::RefilledOwners {
            count: self.auction_batch.len(),
        }
    }

 /// Оригинал общий owner-read, которому принадлежат state и limit аргументы.
 ///
 /// Materialization строк `Auction/AuctionGoods` и публикация output note
 /// остаются в concrete DB context; `CDbMisc` сохраняет сам caller-contract
 /// и число созданных `CGoodsNode`.
    pub fn load_goods_by_owner_id(
        &self,
        context: &mut impl DbMiscContext,
        owner_id: i32,
        state: GoodsState,
        limit: i32,
    ) -> i32 {
        context.load_owner_auction_goods(owner_id, state.raw(), limit)
    }

    pub fn load_owner_back_goods(
        &self,
        context: &mut impl DbMiscContext,
        owner_id: i32,
        limit: i32,
    ) -> i32 {
        self.load_goods_by_owner_id(context, owner_id, GoodsState::BACK, limit)
    }

    pub fn load_owner_undo_goods(
        &self,
        context: &mut impl DbMiscContext,
        owner_id: i32,
        limit: i32,
    ) -> i32 {
        self.load_goods_by_owner_id(context, owner_id, GoodsState::UNDO, limit)
    }

    pub fn load_owner_succ_goods(
        &self,
        context: &mut impl DbMiscContext,
        owner_id: i32,
        limit: i32,
    ) -> i32 {
        self.load_goods_by_owner_id(context, owner_id, GoodsState::SUCESSED, limit)
    }

    pub fn load_money_by_id(
        &self,
        context: &mut impl DbMiscContext,
        owner_id: i32,
        money_limit: i32,
    ) {
        context.load_owner_auction_money(owner_id, money_limit);
    }
}

fn move_queue_batch<T>(source: &mut VecDeque<T>, limit: i32) -> VecDeque<T> {
    if limit == 0 {
        return std::mem::take(source);
    }
    if limit < 0 {
        return VecDeque::new();
    }
    let count = usize::try_from(limit)
        .expect("положительный Windows long помещается в usize")
        .min(source.len());
    source.drain(..count).collect()
}

fn output_block(
    reason: DbMiscDoneOutBlockReason,
    processed_notes: usize,
    pending_notes: VecDeque<Box<DbNote>>,
) -> DbMiscDoneOutBlock {
    DbMiscDoneOutBlock {
        reason,
        processed_notes,
        pending_notes,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuctionWriteOperation {
    CreateNormalConnection,
    CheckNormalConnection,
    InsertItem,
    InsertAddonProperty,
    DeleteItem,
    DeleteMoney,
    ModifyStateAuctionToSucceeded,
    TransferSellerMoney,
    ModifyReturnedGoodsState,
}

/// Технический отказ TDS-границы, который старый owner представлял `false` и
/// `PrintErr`-веткой.
#[derive(Debug)]
pub enum AuctionWriteFailure {
    Connection {
        operation: AuctionWriteOperation,
        source: WorldDatabaseConnectionError,
    },
    Database {
        operation: AuctionWriteOperation,
        source: tiberius::error::Error,
    },
    LegacyUnsignedOutsideSqlInt {
        operation: AuctionWriteOperation,
        field: &'static str,
        value: u32,
    },
    MissingGoodsBaseProperties {
        operation: AuctionWriteOperation,
    },
}

impl fmt::Display for AuctionWriteFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection { source, .. } => source.fmt(formatter),
            Self::Database { operation, source } => {
                write!(formatter, "ошибка Auction DB при {operation:?}: {source}")
            }
            Self::LegacyUnsignedOutsideSqlInt {
                operation,
                field,
                value,
            } => write!(
                formatter,
                "{operation:?}: {field}={value} не представим исходным SQL int"
            ),
            Self::MissingGoodsBaseProperties { operation } => write!(
                formatter,
                "{operation:?}: CGoodsFactory::QueryGoodsBaseProperties вернул null"
            ),
        }
    }
}

impl Error for AuctionWriteFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Connection { source, .. } => Some(source),
            Self::Database { source, .. } => Some(source),
            Self::LegacyUnsignedOutsideSqlInt { .. } | Self::MissingGoodsBaseProperties { .. } => None,
        }
    }
}

/// Безопасная остановка на неопределённой legacy-границе до либо после уже
/// выполненного SQL-перехода. Она не подменяется выдуманным `bool` старого
/// процесса.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuctionWriteBlock {
    BuyerNameWithoutTerminator,
    MissingSellerMoneyAfterFee,
    InsertTextWithoutTerminator { field: &'static str },
    MissingGoodsType,
    MissingLevelLimit,
    GoodsDecode(GoodsCodecError),
    GoodsSnapshot(GoodsDbSnapshotBlock),
    GuidGeneration,
}

/// Результат одного write-перехода без выдумывания проверки rowcount: ADO
/// `ExecuteCn` сообщал только успех выполнения команды.
#[derive(Debug)]
pub enum AuctionWriteOutcome {
    Written,
 /// `ModifyGoodsStateA2S`, `TansferMoney` и `ModifyGoodsStateA2B` печатали
 /// ошибку `ExecuteCn`, но normal return оставался `true`. Это не
 /// технический дефект, который можно исправить локально: `DoneListIn`
 /// публикует из него внешний успешный переход.
    ReturnedTrueAfterDatabaseFailure(AuctionWriteFailure),
    ReturnedFalse(AuctionWriteFailure),
    BlockedMissingFact(AuctionWriteBlock),
}

/// Результат оригинал `IsActiveConnect`: `ExecuteCnEx` возвращал unsigned
/// affected-row count, а `CDbMisc` считал соединение активным только при нуле.
#[derive(Debug)]
pub enum AuctionConnectionState {
    Active,
    Inactive { affected_rows: u64 },
    Failed(AuctionWriteFailure),
}

impl AuctionConnectionState {
    pub const fn legacy_bool(&self) -> bool {
        matches!(self, Self::Active)
    }
}

impl AuctionWriteOutcome {
 /// Точный bool для внешнего async-адаптера `DbMiscContext`; безопасная
 /// блокировка не выдаётся за исходный ответ старого процесса.
    pub const fn legacy_bool(&self) -> Option<bool> {
        match self {
            Self::Written | Self::ReturnedTrueAfterDatabaseFailure(_) => Some(true),
            Self::ReturnedFalse(_) => Some(false),
            Self::BlockedMissingFact(_) => None,
        }
    }
}

/// Linux/TDS-владелец пяти точных DB-переходов записи `CDbMisc`.
///
/// Проверка машинного кода World EXE фиксирует аргументы format strings:
/// `MondifyMoney(money, player_id)` в..,
/// `BuyGoods(guid, buyer_id, buyer_name, buyer_id)` в
///.., `TransferMoney(seller_id, payout)` в
///.. и `UpdateGoodsState(guid, state)` в
///... Первое удаление сохраняет самостоятельное
/// соединение..; четыре прочие команды используют
/// `m_NormalCn`. Параметризованный TDS заменяет только небезопасный `_sprintf` и
/// не объединяет последовательные BuyGoods/TransferMoney в транзакцию.
pub struct TiberiusAuctionWriteOwner {
    settings: WorldDatabaseSettings,
}

impl TiberiusAuctionWriteOwner {
    pub fn new(settings: &WorldDatabaseSettings) -> Self {
        Self {
            settings: settings.clone(),
        }
    }

 /// Заменяет normal connection перед первым DB-batch либо после reconnect.
 /// Старый `CreateNormalCn` сначала освобождал `m_NormalCn`, затем создавал
 /// и открывал новый ADO connection; ошибка любого шага попадала в catch и
 /// возвращала `false`. Владелец старого `WorldTdsClient` в Rust сам решает,
 /// когда отбросить его перед вызовом, а этот метод передаёт только новый
 /// успешно установленный owner.
    pub async fn create_normal_connection(
        &self,
    ) -> Result<WorldTdsClient, AuctionWriteFailure> {
        self.settings
            .connect()
            .await
            .map_err(|source| AuctionWriteFailure::Connection {
                operation: AuctionWriteOperation::CreateNormalConnection,
                source,
            })
    }

 /// Выполняет literal `exec IsActiveConnect` на normal connection.
 ///
 /// Нулевой affected-row count `ExecuteCnEx` означает `true`, любой
 /// ненулевой count или ADO exception — `false`. Tiberius даёт тот же count
 /// через `ExecuteResult::total`; ошибка сохраняется отдельно, но для
 /// legacy caller также означает неактивное соединение.
    pub async fn is_active_connection(
        &self,
        normal_connection: &mut WorldTdsClient,
    ) -> AuctionConnectionState {
        match normal_connection.execute("exec IsActiveConnect", &[]).await {
            Ok(result) => {
                let affected_rows = result.total();
                if affected_rows == 0 {
                    AuctionConnectionState::Active
                } else {
                    AuctionConnectionState::Inactive { affected_rows }
                }
            }
            Err(source) => AuctionConnectionState::Failed(AuctionWriteFailure::Database {
                operation: AuctionWriteOperation::CheckNormalConnection,
                source,
            }),
        }
    }

 /// Вставляет один auction lot точной последовательностью `InsertItemToDb`.
 ///
 /// Вложенный `CGoods` декодируется до SQL, затем отдельный `exec
 /// AddNewGoods` записывает главную строку. Base-properties и каждая
 /// `AddNewGoodsPre` обрабатываются только после этого успеха: ни
 /// отсутствующая база, ни поздний DB-отказ не откатывают уже созданный
 /// лот, поскольку оригинал owner не открывал транзакцию.
    pub async fn insert_item_to_db(
        &self,
        normal_connection: &mut WorldTdsClient,
        node: &CGoodsNode,
        registry: &GoodsBasePropertiesRegistry,
    ) -> AuctionWriteOutcome {
        let mut goods = CGoods::with_constructor_base_and_type();
        let mut cursor = 0;
        if let Err(source) = goods.unserialize(node.goods_bytes(), &mut cursor, true) {
            return AuctionWriteOutcome::BlockedMissingFact(AuctionWriteBlock::GoodsDecode(source));
        }

        let row_id = match CGuid::create() {
            Ok(value) => value,
            Err(_) => {
                return AuctionWriteOutcome::BlockedMissingFact(AuctionWriteBlock::GuidGeneration);
            }
        };
        let fields = node.database_insert_fields();
        let text = match AuctionInsertText::from_fields(&fields) {
            Ok(value) => value,
            Err(source) => return AuctionWriteOutcome::BlockedMissingFact(source),
        };

        let mut query = Query::new(
            "exec addnewGoods @P1,@P2,@P3,@P4,@P5,@P6,@P7,@P8,@P9,@P10,@P11,\
             @P12,@P13,@P14,@P15,@P16,@P17,@P18,@P19,@P20,@P21,@P22,@P23",
        );
        query.bind(row_id.to_string());
        query.bind(goods.get_ex_id().to_string());
        query.bind(i64::from(fields.add_ticket));
        query.bind(i64::from(fields.base_index));
        query.bind(text.account);
        query.bind(i64::from(fields.owner_id));
        query.bind(i64::from(fields.auction_time));
        query.bind(i32::from(fields.money_type));
        query.bind(i32::from(text.goods_type));
        query.bind(i64::from(fields.npc_price as u32));
        query.bind(i64::from(fields.amount as u32));
        query.bind(fields.goods_state.raw());
        query.bind(i32::from(fields.offer_price));
        query.bind(text.goods_name);
        query.bind(text.seller_name);
        query.bind(i64::from(fields.money_seller));
        query.bind(i64::from(fields.time_seller));
        query.bind(i64::from(fields.seller_id));
        query.bind(text.buyer_name);
        query.bind(i64::from(fields.money_buyer));
        query.bind(i64::from(fields.buyer_id));
        query.bind(i64::from(fields.time_buyer));
        query.bind(i64::from(text.level_limit));
        if let Err(source) = query.execute(&mut *normal_connection).await {
            return AuctionWriteOutcome::ReturnedFalse(AuctionWriteFailure::Database {
                operation: AuctionWriteOperation::InsertItem,
                source,
            });
        }

        let snapshot = match goods.db_save_snapshot(registry) {
            Ok(value) => value,
            Err(source) => {
                return AuctionWriteOutcome::BlockedMissingFact(AuctionWriteBlock::GoodsSnapshot(source));
            }
        };
        let properties = match snapshot.properties {
            GoodsPropertiesSnapshot::Available(properties) => properties,
            GoodsPropertiesSnapshot::MissingBaseProperties => {
                return AuctionWriteOutcome::ReturnedFalse(
                    AuctionWriteFailure::MissingGoodsBaseProperties {
                        operation: AuctionWriteOperation::InsertItem,
                    },
                );
            }
        };
        if let Err(source) = save_auction_goods_properties(
            &properties,
            row_id,
            normal_connection,
        )
        .await
        {
            return AuctionWriteOutcome::ReturnedFalse(source);
        }
        AuctionWriteOutcome::Written
    }

 /// Выполняет точный `delete auction where goodsid = '%s'` в отдельном
 /// соединении, как `DelItemFromDb`.
    pub async fn delete_item_from_db(&self, guid: CGuid) -> AuctionWriteOutcome {
        let mut connection = match self.settings.connect().await {
            Ok(connection) => connection,
            Err(source) => {
                return AuctionWriteOutcome::ReturnedFalse(AuctionWriteFailure::Connection {
                    operation: AuctionWriteOperation::DeleteItem,
                    source,
                });
            }
        };
        let mut query = Query::new("delete auction where goodsid = @P1");
        query.bind(guid.to_string());
        execute_auction_write(
            AuctionWriteOperation::DeleteItem,
            &mut connection,
            query,
        )
        .await
    }

 /// Выполняет `exec MondifyMoney money, player_id` через normal connection
 /// от вызывающего кода. Значение денег является абсолютным остатком, а не
 /// суммой для вычитания: именно его передавал `DoneListIn`.
    pub async fn delete_money_from_db(
        &self,
        normal_connection: &mut WorldTdsClient,
        player_id: i32,
        money: i32,
    ) -> AuctionWriteOutcome {
        let mut query = Query::new("exec MondifyMoney @P1, @P2");
        query.bind(money);
        query.bind(player_id);
        execute_auction_write(
            AuctionWriteOperation::DeleteMoney,
            normal_connection,
            query,
        )
        .await
    }

 /// Выполняет первую точную часть A2S: `BuyGoods`. Вызывающий обязан вызвать
 /// `transfer_money` только после `Written`, сохраняя исходную частичную
 /// фиксацию при отказе второй команды.
    pub async fn modify_goods_state_a2s(
        &self,
        normal_connection: &mut WorldTdsClient,
        fields: AuctionDatabaseWriteFields<'_>,
    ) -> AuctionWriteOutcome {
        let buyer_name = match legacy_auction_buyer_name(fields.buyer_name) {
            Some(value) => value,
            None => {
                return AuctionWriteOutcome::BlockedMissingFact(
                    AuctionWriteBlock::BuyerNameWithoutTerminator,
                );
            }
        };
        let mut query = Query::new("exec buygoods @P1, @P2, @P3, @P4");
        query.bind(fields.guid.to_string());
        query.bind(fields.buyer_id as i32);
        query.bind(buyer_name.as_str());
        query.bind(fields.buyer_id as i32);
        execute_auction_write_with_ignored_failure(
            AuctionWriteOperation::ModifyStateAuctionToSucceeded,
            normal_connection,
            query,
        )
        .await
    }

 /// Выполняет вторую A2S-команду отдельно от `BuyGoods`.
 ///
 /// Для money type `1` исходный owner возвращал успех до вычисления
 /// комиссии и SQL. Для остальных значений вызывающий передаёт точный результат
 /// `CGame::GetOptMoneyJin`; отсутствие такого значения не превращается в
 /// нулевое начисление.
    pub async fn transfer_money(
        &self,
        normal_connection: &mut WorldTdsClient,
        fields: AuctionDatabaseWriteFields<'_>,
        seller_money_after_fee: Option<i32>,
    ) -> AuctionWriteOutcome {
        if fields.money_type == 1 {
            return AuctionWriteOutcome::Written;
        }
        let Some(seller_money_after_fee) = seller_money_after_fee else {
            return AuctionWriteOutcome::BlockedMissingFact(
                AuctionWriteBlock::MissingSellerMoneyAfterFee,
            );
        };
        let seller_id = match legacy_unsigned_sql_int(
            AuctionWriteOperation::TransferSellerMoney,
            "dwSellerId",
            fields.seller_id,
        ) {
            Ok(value) => value,
            Err(error) => {
                return AuctionWriteOutcome::ReturnedTrueAfterDatabaseFailure(error);
            }
        };
        let payout = match legacy_unsigned_sql_int(
            AuctionWriteOperation::TransferSellerMoney,
            "seller_money_after_fee",
            seller_money_after_fee as u32,
        ) {
            Ok(value) => value,
            Err(error) => {
                return AuctionWriteOutcome::ReturnedTrueAfterDatabaseFailure(error);
            }
        };
        let mut query = Query::new("exec TransferMoney @P1,@P2");
        query.bind(seller_id);
        query.bind(payout);
        execute_auction_write_with_ignored_failure(
            AuctionWriteOperation::TransferSellerMoney,
            normal_connection,
            query,
        )
        .await
    }

    pub async fn modify_goods_state_a2b(
        &self,
        normal_connection: &mut WorldTdsClient,
        fields: AuctionDatabaseWriteFields<'_>,
    ) -> AuctionWriteOutcome {
        let mut query = Query::new("exec UpdateGoodsState @P1, @P2");
        query.bind(fields.guid.to_string());
        query.bind(fields.goods_state.raw());
        execute_auction_write_with_ignored_failure(
            AuctionWriteOperation::ModifyReturnedGoodsState,
            normal_connection,
            query,
        )
        .await
    }
}

/// Преобразованные строки одного legacy `AddNewGoods` batch. Каждая старый
/// `_sprintf("%s")` останавливался на NUL; отсутствие terminator не получает
/// произвольного чтения соседней памяти.
struct AuctionInsertText {
    account: String,
    goods_type: u8,
    goods_name: String,
    seller_name: String,
    buyer_name: String,
    level_limit: u32,
}

impl AuctionInsertText {
    fn from_fields(fields: &AuctionDatabaseInsertFields<'_>) -> Result<Self, AuctionWriteBlock> {
        let goods_type = fields.goods_type.ok_or(AuctionWriteBlock::MissingGoodsType)?;
        let level_limit = fields
            .level_limit
            .ok_or(AuctionWriteBlock::MissingLevelLimit)?;
        Ok(Self {
            account: legacy_auction_text(fields.account, "m_strAccount")?,
            goods_type,
            goods_name: legacy_auction_text(fields.goods_name, "m_strGoodsName")?,
            seller_name: legacy_auction_text(fields.seller_name, "m_AucInfo.strSellerName")?,
            buyer_name: legacy_auction_text(fields.buyer_name, "m_AucInfo.strBuyerName")?,
            level_limit,
        })
    }
}

/// Сохраняет addon records по оригинал `SaveGoodsProperties`. Четыре
/// accumulator-а намеренно созданы до внешнего цикла и не сбрасываются между
/// properties: это подтверждённая DB-семантика, а не техническая деталь.
async fn save_auction_goods_properties(
    properties: &[GoodsAddonPropertySnapshot],
    row_id: CGuid,
    normal_connection: &mut WorldTdsClient,
) -> Result<(), AuctionWriteFailure> {
    let row_id = row_id.to_string();
    let mut first_base = 0_i32;
    let mut first_modifier = 0_i32;
    let mut second_base = 0_i32;
    let mut second_modifier = 0_i32;

    for property in properties {
        let (property_type, occur_probability, values) = property.legacy_parts();
        for value in values {
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

        let modifier_value_2 = if property_type == 0x25 {
            if first_modifier == 0 && first_base == second_base {
                continue;
            }
            second_base
        } else {
            if occur_probability == 10_000 && first_modifier == 0 && second_modifier == 0 {
                continue;
            }
            second_modifier
        };

        let mut query = Query::new("exec AddNewGoodsPre @P1,@P2,@P3,@P4");
        query.bind(property_type as i32);
        query.bind(first_modifier);
        query.bind(modifier_value_2);
        query.bind(row_id.as_str());
        query
            .execute(&mut *normal_connection)
            .await
            .map_err(|source| AuctionWriteFailure::Database {
                operation: AuctionWriteOperation::InsertAddonProperty,
                source,
            })?;
    }
    Ok(())
}

async fn execute_auction_write(
    operation: AuctionWriteOperation,
    connection: &mut WorldTdsClient,
    query: Query<'_>,
) -> AuctionWriteOutcome {
    match query.execute(connection).await {
        Ok(_) => AuctionWriteOutcome::Written,
        Err(source) => AuctionWriteOutcome::ReturnedFalse(AuctionWriteFailure::Database {
            operation,
            source,
        }),
    }
}

async fn execute_auction_write_with_ignored_failure(
    operation: AuctionWriteOperation,
    connection: &mut WorldTdsClient,
    query: Query<'_>,
) -> AuctionWriteOutcome {
    match query.execute(connection).await {
        Ok(_) => AuctionWriteOutcome::Written,
        Err(source) => AuctionWriteOutcome::ReturnedTrueAfterDatabaseFailure(
            AuctionWriteFailure::Database { operation, source },
        ),
    }
}

fn legacy_auction_text(bytes: &[u8], field: &'static str) -> Result<String, AuctionWriteBlock> {
    let terminator = bytes
        .iter()
        .position(|byte| *byte == 0)
        .ok_or(AuctionWriteBlock::InsertTextWithoutTerminator { field })?;
    let (name, _, _) = WINDOWS_1251.decode(&bytes[..terminator]);
    Ok(name.into_owned())
}

fn legacy_auction_buyer_name(bytes: &[u8]) -> Option<String> {
    legacy_auction_text(bytes, "m_AucInfo.strBuyerName").ok()
}

fn legacy_unsigned_sql_int(
    operation: AuctionWriteOperation,
    field: &'static str,
    value: u32,
) -> Result<i32, AuctionWriteFailure> {
    i32::try_from(value).map_err(|_| AuctionWriteFailure::LegacyUnsignedOutsideSqlInt {
        operation,
        field,
        value,
    })
}

#[derive(Debug)]
pub enum AuctionGoodsLoadFailure {
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

impl fmt::Display for AuctionGoodsLoadFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Connection(source) => source.fmt(formatter),
            Self::Database { row_index, source } => match row_index {
                Some(row_index) => write!(
                    formatter,
                    "ошибка Auction DB в строке {row_index}: {source}"
                ),
                None => write!(formatter, "ошибка запроса Auction DB: {source}"),
            },
            Self::MissingRequiredValue { row_index, column } => {
                write!(
                    formatter,
                    "в строке Auction DB {row_index} отсутствует {column}"
                )
            }
        }
    }
}

impl Error for AuctionGoodsLoadFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Connection(source) => Some(source),
            Self::Database { source, .. } => Some(source),
            Self::MissingRequiredValue { .. } => None,
        }
    }
}

/// Безопасная граница данных, для которой исходный ADO owner не задавал
/// наблюдаемого нормального продолжения.
#[derive(Debug)]
pub enum AuctionGoodsLoadBlock {
    GoodsGuid {
        row_index: usize,
        source: GuidParseError,
    },
    TextOutsideWindows1251 {
        row_index: usize,
        column: &'static str,
    },
    MissingGoodsFactoryEntry {
        row_index: usize,
        base_index: u32,
    },
    Addon {
        row_index: usize,
        source: GoodsLoadedAddonBlock,
    },
    GoodsSerialize {
        guid: CGuid,
        source: GoodsCodecError,
    },
}

/// Результат одного оригинал read `Auction/AuctionGoods`.
///
/// Успех возвращает уже готовые `OT_OUT_READ_AUCTION_RESULT` notes в порядке
/// сырого `CGUID`; concrete World context обязан append-нуть их в output FIFO
/// без перемешивания с другими producer-ами.
pub enum AuctionGoodsLoadOutcome {
    Loaded(VecDeque<Box<DbNote>>),
    ReturnedFalse(AuctionGoodsLoadFailure),
    BlockedMissingFact(AuctionGoodsLoadBlock),
}

/// Linux/TDS-реализация точного DB read для возврата/публикации товаров
/// аукциона.
///
/// В отличие от старого ADO `OpenRs`, каждый вызов открывает владимое Tiberius
/// connection. SQL сохраняет исходные `NOLOCK`, join и единственный `ORDER BY`;
/// `limit` намеренно не превращён в `TOP`, поскольку оригинал owner ограничивал
/// уже созданные уникальные `GoodsID` после чтения joined rows.
pub struct TiberiusAuctionGoodsReader {
    settings: WorldDatabaseSettings,
}

impl TiberiusAuctionGoodsReader {
    pub fn new(settings: &WorldDatabaseSettings) -> Self {
        Self {
            settings: settings.clone(),
        }
    }

 /// Читает оригинал owner-page seed через уже проверенное normal connection.
 /// Порядок без `ORDER BY` намеренно остаётся порядком SQL recordset-а.
    pub async fn read_owner_ids(
        &self,
        normal_connection: &mut WorldTdsClient,
    ) -> Result<VecDeque<i32>, AuctionGoodsLoadFailure> {
        let mut rows = normal_connection
            .simple_query("SELECT DISTINCT dwOwerId FROM Auction WITH (NOLOCK)")
            .await
            .map_err(|source| AuctionGoodsLoadFailure::Database {
                row_index: None,
                source,
            })?;
        let mut owners = VecDeque::new();
        let mut row_index = 0usize;
        while let Some(item) = rows.try_next().await.map_err(|source| {
            AuctionGoodsLoadFailure::Database {
                row_index: Some(row_index),
                source,
            }
        })? {
            let Some(row) = item.into_row() else {
                continue;
            };
            let owner_id = row
                .try_get::<i32, _>("dwOwerId")
                .map_err(|source| AuctionGoodsLoadFailure::Database {
                    row_index: Some(row_index),
                    source,
                })?
                .ok_or(AuctionGoodsLoadFailure::MissingRequiredValue {
                    row_index,
                    column: "dwOwerId",
                })?;
            owners.push_back(owner_id);
            row_index += 1;
        }
        Ok(owners)
    }

 /// Materializes `Auction` + `AuctionGoods` в внешний output batch.
 ///
 /// Набор DaKong addon property types получает здесь сам DB owner через
 /// статический `CDaKongXiangQian::GetAddType`: оригинал EXE не читает для
 /// этого setup state и не требует mutable global singleton.
    pub async fn load_goods_by_owner_id(
        &self,
        owner_id: i32,
        state: GoodsState,
        limit: i32,
        registry: &GoodsBasePropertiesRegistry,
    ) -> AuctionGoodsLoadOutcome {
        let mut connection = match self.settings.connect().await {
            Ok(connection) => connection,
            Err(source) => {
                return AuctionGoodsLoadOutcome::ReturnedFalse(
                    AuctionGoodsLoadFailure::Connection(source),
                );
            }
        };
        let mut query = Query::new(
            "SELECT CONVERT(varchar(38),a.GoodsID) AS GoodsID, \
             a.dwAddTicket,a.GoodsIndex,a.strAccount,a.dwOwerId,a.dwauctiontime, \
             a.btMoneyType,a.btGoodsType,a.NpcPrice,a.Amount,a.GoodsState,a.bOfferPrice, \
             a.strGoodsName,a.strSellerName,a.dwMoneySeller,a.dwTimeSeller,a.dwSellerId, \
             a.strBuyerName,a.dwMoneyBuyer,a.BuyerId,a.timebuyer,a.dwLvLimit, \
             CONVERT(int,b.type) AS type, b.modifierValue1, b.modifierValue2 \
             FROM Auction AS a WITH (NOLOCK) \
             LEFT JOIN AuctionGoods AS b WITH (NOLOCK) ON a.id=b.id \
             WHERE dwOwerId=@P1 AND GoodsState=@P2 ORDER BY dwOwerId",
        );
        query.bind(owner_id);
        query.bind(state.raw());
        let mut rows = match query.query(&mut connection).await {
            Ok(rows) => rows,
            Err(source) => {
                return AuctionGoodsLoadOutcome::ReturnedFalse(AuctionGoodsLoadFailure::Database {
                    row_index: None,
                    source,
                });
            }
        };

        let mut pending = BTreeMap::<CGuid, PendingAuctionGoods>::new();
        let mut dakong_addon_types = BTreeSet::new();
        CDaKongXiangQian::get_add_type(&mut dakong_addon_types);
        let mut row_index = 0usize;
        loop {
            let item = match rows.try_next().await {
                Ok(Some(item)) => item,
                Ok(None) => break,
                Err(source) => {
                    return AuctionGoodsLoadOutcome::ReturnedFalse(
                        AuctionGoodsLoadFailure::Database {
                            row_index: Some(row_index),
                            source,
                        },
                    );
                }
            };
            let Some(row) = item.into_row() else {
                continue;
            };
            let record = match AuctionGoodsRecord::read(&row, row_index) {
                Ok(record) => record,
                Err(ReadAuctionGoodsRecordError::Failure(failure)) => {
                    return AuctionGoodsLoadOutcome::ReturnedFalse(failure);
                }
                Err(ReadAuctionGoodsRecordError::Block(block)) => {
                    return AuctionGoodsLoadOutcome::BlockedMissingFact(block);
                }
            };

            if !pending.contains_key(&record.guid) {
                // `if (param_3 <= map._Mysize) break`: строка с новым GUID не
                // превращается в note после достижения лимита. Неположительный
                // legacy-limit даёт пустую страницу после того же DB-запроса.
                if limit <= i32::try_from(pending.len()).unwrap_or(i32::MAX) {
                    break;
                }
                let Some(mut goods) = create_goods_no_probability(registry, record.base_index)
                else {
                    return AuctionGoodsLoadOutcome::BlockedMissingFact(
                        AuctionGoodsLoadBlock::MissingGoodsFactoryEntry {
                            row_index,
                            base_index: record.base_index,
                        },
                    );
                };
                goods.set_ex_id(&record.guid);
                goods.set_amount(record.amount as u32);
                goods.set_price(record.npc_price as u32);
                pending.insert(
                    record.guid,
                    PendingAuctionGoods {
                        fields: record.node_fields(),
                        goods,
                    },
                );
            }

            if let Some((property_type, first_modifier, second_modifier)) = record.addon
                && let Some(item) = pending.get_mut(&record.guid)
                && let Err(source) = item.goods.apply_loaded_addon(
                    property_type,
                    first_modifier,
                    second_modifier,
                    registry,
                    &dakong_addon_types,
                )
            {
                return AuctionGoodsLoadOutcome::BlockedMissingFact(AuctionGoodsLoadBlock::Addon {
                    row_index,
                    source,
                });
            }
            row_index += 1;
        }

        let mut notes = VecDeque::with_capacity(pending.len());
        for (guid, item) in pending {
            let mut goods_bytes = Vec::new();
            if let Err(source) = item.goods.serialize(&mut goods_bytes, true) {
                return AuctionGoodsLoadOutcome::BlockedMissingFact(
                    AuctionGoodsLoadBlock::GoodsSerialize { guid, source },
                );
            }
            notes.push_back(Box::new(DbNote {
                e_type: OperatorType::OT_OUT_READ_AUCTION_RESULT,
                goods: CGoodsNode::from_auction_database(item.fields, goods_bytes),
                player_id: -1,
                money: 0,
            }));
        }
        AuctionGoodsLoadOutcome::Loaded(notes)
    }

 /// Читает `AuctionPlayerMoney` и создаёт оригинал gold-return notes.
 ///
 /// `random` принадлежит World owner-у: исходный `CGoodsFactory::CreateGoods`
 /// выполнял свой probability-roll до проверки `dwmoney != 0`. Поэтому
 /// callback вызывается и для нулевой строки, а не заменяется созданием
 /// deterministic `CreateGoodsNoProbability`.
    pub async fn load_money_by_id<Random>(
        &self,
        owner_id: i32,
        money_limit: i32,
        gold_coin_index: u32,
        registry: &GoodsBasePropertiesRegistry,
        random: &mut Random,
    ) -> AuctionMoneyLoadOutcome
    where
        Random: FnMut(i32) -> i32 + ?Sized,
    {
        let mut connection = match self.settings.connect().await {
            Ok(connection) => connection,
            Err(source) => {
                return AuctionMoneyLoadOutcome::ReturnedFalse(
                    AuctionGoodsLoadFailure::Connection(source),
                );
            }
        };
        let mut query = Query::new(
            "SELECT CONVERT(bigint,dwmoney) AS dwmoney FROM AuctionPlayerMoney \
             WHERE dwplayerid=@P1",
        );
        query.bind(owner_id);
        let mut rows = match query.query(&mut connection).await {
            Ok(rows) => rows,
            Err(source) => {
                return AuctionMoneyLoadOutcome::ReturnedFalse(AuctionGoodsLoadFailure::Database {
                    row_index: None,
                    source,
                });
            }
        };

        let mut row_index = 0usize;
        let mut returned_amount = 0_i32;
        let mut notes = VecDeque::new();
        loop {
            let item = match rows.try_next().await {
                Ok(Some(item)) => item,
                Ok(None) => break,
                Err(source) => {
                    return AuctionMoneyLoadOutcome::ReturnedFalse(
                        AuctionGoodsLoadFailure::Database {
                            row_index: Some(row_index),
                            source,
                        },
                    );
                }
            };
            let Some(row) = item.into_row() else {
                continue;
            };
            let money = match row.try_get::<i64, _>("dwmoney") {
                Ok(Some(money)) => money as u64,
                Ok(None) => {
                    return AuctionMoneyLoadOutcome::ReturnedFalse(
                        AuctionGoodsLoadFailure::MissingRequiredValue {
                            row_index,
                            column: "dwmoney",
                        },
                    );
                }
                Err(source) => {
                    return AuctionMoneyLoadOutcome::ReturnedFalse(
                        AuctionGoodsLoadFailure::Database {
                            row_index: Some(row_index),
                            source,
                        },
                    );
                }
            };

 // В оригинал EXE `CreateGoods` расположен перед `dwmoney != 0`.
            let created_goods = create_goods(registry, gold_coin_index, random);
            if let Some(mut goods) = created_goods
                && money != 0
            {
                let limit_as_u64 = (money_limit as i64) as u64;
                let (note_amount, goods_amount) = if limit_as_u64 < money {
 // 004F1F3E: note получает остаток, а вложенный CGoods —
 // именно запрошенную порцию. Эта странность наблюдаема
 // через последующий `OT_IN_MODIFY_MONEY` и сохранена.
                    (money.wrapping_sub(limit_as_u64) as i32, money_limit as u32)
                } else {
                    (0, money as u32)
                };
                let guid = match CGuid::create() {
                    Ok(guid) => guid,
                    Err(source) => {
                        return AuctionMoneyLoadOutcome::BlockedMissingFact(
                            AuctionMoneyLoadBlock::GuidGeneration { row_index, source },
                        );
                    }
                };
                goods.set_amount(goods_amount);
                goods.set_ex_id(&guid);
                let mut goods_bytes = Vec::new();
                if let Err(source) = goods.serialize(&mut goods_bytes, true) {
                    return AuctionMoneyLoadOutcome::BlockedMissingFact(
                        AuctionMoneyLoadBlock::GoodsSerialize { row_index, source },
                    );
                }
                returned_amount = goods_amount as i32;
                notes.push_back(Box::new(DbNote {
                    e_type: OperatorType::OT_OUT_READ_AUCTION_RESULT,
                    goods: CGoodsNode::from_auction_money_return(
                        note_amount,
                        guid,
                        gold_coin_index,
                        goods_bytes,
                    ),
                    player_id: -1,
                    money: 0,
                }));
            }
            row_index += 1;
        }
        AuctionMoneyLoadOutcome::Loaded {
            notes,
            returned_amount,
        }
    }
}

/// `LoadMoneyById` возвращал последнее количество, помещённое во вложенный
/// gold `CGoods`; caller S2W его буквально игнорирует, но typed owner не
/// скрывает значение от следующей интеграционной границы.
pub enum AuctionMoneyLoadOutcome {
    Loaded {
        notes: VecDeque<Box<DbNote>>,
        returned_amount: i32,
    },
    ReturnedFalse(AuctionGoodsLoadFailure),
    BlockedMissingFact(AuctionMoneyLoadBlock),
}

#[derive(Debug)]
pub enum AuctionMoneyLoadBlock {
    GuidGeneration {
        row_index: usize,
        source: getrandom::Error,
    },
    GoodsSerialize {
        row_index: usize,
        source: GoodsCodecError,
    },
}

struct PendingAuctionGoods {
    fields: AuctionDatabaseNodeFields,
    goods: Box<CGoods>,
}

struct AuctionGoodsRecord {
    guid: CGuid,
    add_ticket: u32,
    base_index: u32,
    account: Vec<u8>,
    owner_id: u32,
    auction_time: u32,
    money_type: u8,
    goods_type: u8,
    npc_price: i32,
    amount: i32,
    goods_state: GoodsState,
    offer_price: bool,
    goods_name: Vec<u8>,
    seller_name: Vec<u8>,
    money_seller: u32,
    time_seller: u32,
    seller_id: u32,
    buyer_name: Vec<u8>,
    money_buyer: u32,
    buyer_id: u32,
    time_buyer: u32,
    level_limit: u32,
    addon: Option<(i32, i32, i32)>,
}

impl AuctionGoodsRecord {
    fn read(row: &tiberius::Row, row_index: usize) -> Result<Self, ReadAuctionGoodsRecordError> {
        macro_rules! required {
            ($type:ty, $column:literal) => {
                match row.try_get::<$type, _>($column) {
                    Ok(Some(value)) => value,
                    Ok(None) => {
                        return Err(ReadAuctionGoodsRecordError::Failure(
                            AuctionGoodsLoadFailure::MissingRequiredValue {
                                row_index,
                                column: $column,
                            },
                        ));
                    }
                    Err(source) => {
                        return Err(ReadAuctionGoodsRecordError::Failure(
                            AuctionGoodsLoadFailure::Database {
                                row_index: Some(row_index),
                                source,
                            },
                        ));
                    }
                }
            };
        }
        macro_rules! ansi {
            ($column:literal) => {{
                let source = required!(&str, $column);
                let (encoded, _, had_errors) = WINDOWS_1251.encode(source);
                if had_errors {
                    return Err(ReadAuctionGoodsRecordError::Block(
                        AuctionGoodsLoadBlock::TextOutsideWindows1251 {
                            row_index,
                            column: $column,
                        },
                    ));
                }
                encoded.into_owned()
            }};
        }

        let guid_text = required!(&str, "GoodsID");
        let guid = CGuid::from_legacy_text(Some(guid_text)).map_err(|source| {
            ReadAuctionGoodsRecordError::Block(AuctionGoodsLoadBlock::GoodsGuid {
                row_index,
                source,
            })
        })?;
        let addon = match row.try_get::<i32, _>("type") {
            Ok(Some(property_type)) => Some((
                property_type,
                required!(i32, "modifierValue1"),
                required!(i32, "modifierValue2"),
            )),
            Ok(None) => None,
            Err(source) => {
                return Err(ReadAuctionGoodsRecordError::Failure(
                    AuctionGoodsLoadFailure::Database {
                        row_index: Some(row_index),
                        source,
                    },
                ));
            }
        };
        Ok(Self {
            guid,
            add_ticket: required!(i32, "dwAddTicket") as u32,
            base_index: required!(i32, "GoodsIndex") as u32,
            account: ansi!("strAccount"),
            owner_id: required!(i32, "dwOwerId") as u32,
            auction_time: required!(i64, "dwauctiontime") as u32,
            money_type: required!(i32, "btMoneyType") as u8,
            goods_type: required!(i32, "btGoodsType") as u8,
            npc_price: required!(i32, "NpcPrice"),
            amount: required!(i32, "Amount"),
            goods_state: GoodsState::from_raw(required!(i32, "GoodsState")),
            offer_price: required!(i32, "bOfferPrice") != 0,
            goods_name: ansi!("strGoodsName"),
            seller_name: ansi!("strSellerName"),
            money_seller: required!(i32, "dwMoneySeller") as u32,
            time_seller: required!(i32, "dwTimeSeller") as u32,
            seller_id: required!(i32, "dwSellerId") as u32,
            buyer_name: ansi!("strBuyerName"),
            money_buyer: required!(i32, "dwMoneyBuyer") as u32,
            buyer_id: required!(i32, "BuyerId") as u32,
            time_buyer: required!(i64, "TimeBuyer") as u32,
            level_limit: required!(i32, "dwLvLimit") as u32,
            addon,
        })
    }

    fn node_fields(&self) -> AuctionDatabaseNodeFields {
        AuctionDatabaseNodeFields {
            add_ticket: self.add_ticket,
            account: self.account.clone(),
            owner_id: self.owner_id,
            auction_time: self.auction_time,
            money_type: self.money_type,
            goods_type: self.goods_type,
            npc_price: self.npc_price,
            amount: self.amount,
            goods_state: self.goods_state,
            offer_price: self.offer_price,
            guid: self.guid,
            base_index: self.base_index,
            level_limit: self.level_limit,
            goods_name: self.goods_name.clone(),
            seller_name: self.seller_name.clone(),
            money_seller: self.money_seller,
            time_seller: self.time_seller,
            seller_id: self.seller_id,
            buyer_name: self.buyer_name.clone(),
            money_buyer: self.money_buyer,
            time_buyer: self.time_buyer,
            buyer_id: self.buyer_id,
        }
    }
}

enum ReadAuctionGoodsRecordError {
    Failure(AuctionGoodsLoadFailure),
    Block(AuctionGoodsLoadBlock),
}

/// Долгоживущая DB-часть concrete `DbMiscContext`.
///
/// Normal connection сохраняется между MainLoop-проходами и заменяется ровно
/// в `CreateNormalCn`/reconnect-точках. Отдельные read/delete соединения по-
/// прежнему создают соответствующие Tiberius owner-ы на один вызов.
pub struct TiberiusDbMiscDatabase {
    writer: TiberiusAuctionWriteOwner,
    reader: TiberiusAuctionGoodsReader,
    normal_connection: Option<WorldTdsClient>,
}

impl TiberiusDbMiscDatabase {
    pub fn new(settings: &WorldDatabaseSettings) -> Self {
        Self {
            writer: TiberiusAuctionWriteOwner::new(settings),
            reader: TiberiusAuctionGoodsReader::new(settings),
            normal_connection: None,
        }
    }

 /// Открывает исходное `m_NormalCn` в позиции создания process-owner-а.
 ///
 /// `CDbMisc` не владеет TDS-клиентом: очередь остаётся доменным owner-ом,
 /// а соединение живёт в process DB-контексте и затем переиспользуется всеми
 /// последовательными MainLoop batch-ами.
    pub async fn initialize_normal_connection(
        &mut self,
    ) -> Result<(), AuctionWriteFailure> {
        self.normal_connection = Some(self.writer.create_normal_connection().await?);
        Ok(())
    }
}

pub enum TiberiusDbMiscRuntimeEvent<'a> {
    MissingNormalConnection(AuctionWriteOperation),
    NormalConnectionFailed(&'a AuctionWriteFailure),
    ConnectionCheck(&'a AuctionConnectionState),
    Write(&'a AuctionWriteOutcome),
    GoodsLoad(&'a AuctionGoodsLoadOutcome),
    MoneyLoad(&'a AuctionMoneyLoadOutcome),
    OwnerListFailed(&'a AuctionGoodsLoadFailure),
}

/// Короткоживущие World/domain callbacks одного MainLoop-прохода.
///
/// DB owner не хранит ссылок на `CGame` либо reloadable setup. Поэтому caller
/// каждый проход передаёт актуальные routing/config owners и не создаёт
/// расходящуюся копию состояния после reload.
pub struct TiberiusDbMiscCallbacks {
    pub transfer_money_interval_ms: Box<dyn FnMut() -> i32>,
    pub current_tick_ms: Box<dyn FnMut() -> u32>,
    pub report_reconnect: Box<dyn FnMut()>,
    pub report_runtime_event:
        Box<dyn for<'event> FnMut(TiberiusDbMiscRuntimeEvent<'event>)>,
    pub seller_money_after_fee: Box<dyn FnMut(&CGoodsNode) -> Option<i32>>,
    pub gold_coin_index: Box<dyn FnMut() -> u32>,
    pub random: Box<dyn FnMut(i32) -> i32>,
}

/// Concrete synchronous owner-contract поверх асинхронного Tiberius.
///
/// Старый `CDbMisc` выполнял ADO-вызовы последовательно внутри MainLoop. На
/// multi-thread Tokio runtime `block_in_place + Handle::block_on` сохраняет
/// этот caller-visible порядок, но отдаёт TDS I/O асинхронному драйверу и не
/// создаёт второй module graph либо отдельную копию World owners.
pub struct TiberiusDbMiscContext {
    runtime: tokio::runtime::Handle,
    database: TiberiusDbMiscDatabase,
    registry: GoodsBasePropertiesRegistry,
    output: DbMiscOutputPublisher,
    callbacks: TiberiusDbMiscCallbacks,
}

impl TiberiusDbMiscContext {
    pub fn new(
        runtime: tokio::runtime::Handle,
        database: TiberiusDbMiscDatabase,
        registry: GoodsBasePropertiesRegistry,
        output: DbMiscOutputPublisher,
        callbacks: TiberiusDbMiscCallbacks,
    ) -> Self {
        Self {
            runtime,
            database,
            registry,
            output,
            callbacks,
        }
    }

    fn write_bool(&mut self, outcome: AuctionWriteOutcome) -> bool {
        let result = outcome.legacy_bool().unwrap_or(false);
        if !matches!(outcome, AuctionWriteOutcome::Written) {
            (self.callbacks.report_runtime_event)(TiberiusDbMiscRuntimeEvent::Write(&outcome));
        }
        result
    }

    fn report_write(&mut self, outcome: AuctionWriteOutcome) {
        if !matches!(outcome, AuctionWriteOutcome::Written) {
            (self.callbacks.report_runtime_event)(TiberiusDbMiscRuntimeEvent::Write(&outcome));
        }
    }

    fn report_missing_connection(&mut self, operation: AuctionWriteOperation) {
        (self.callbacks.report_runtime_event)(
            TiberiusDbMiscRuntimeEvent::MissingNormalConnection(operation),
        );
    }
}

impl DbMiscContext for TiberiusDbMiscContext {
    fn transfer_money_interval_ms(&mut self) -> i32 {
        (self.callbacks.transfer_money_interval_ms)()
    }

    fn current_tick_ms(&mut self) -> u32 {
        (self.callbacks.current_tick_ms)()
    }

    fn is_active_connect(&mut self) -> bool {
        let Some(connection) = self.database.normal_connection.as_mut() else {
            return false;
        };
        let runtime = self.runtime.clone();
        let writer = &self.database.writer;
        let state = tokio::task::block_in_place(|| {
            runtime.block_on(writer.is_active_connection(connection))
        });
        let active = state.legacy_bool();
        if !active {
            (self.callbacks.report_runtime_event)(
                TiberiusDbMiscRuntimeEvent::ConnectionCheck(&state),
            );
        }
        active
    }

    fn log_connect_error_and_reconnect(&mut self) {
        (self.callbacks.report_reconnect)();
    }

    fn create_normal_connection(&mut self) {
        self.database.normal_connection.take();
        let runtime = self.runtime.clone();
        let result = tokio::task::block_in_place(|| {
            runtime.block_on(self.database.writer.create_normal_connection())
        });
        match result {
            Ok(connection) => self.database.normal_connection = Some(connection),
            Err(error) => (self.callbacks.report_runtime_event)(
                TiberiusDbMiscRuntimeEvent::NormalConnectionFailed(&error),
            ),
        }
    }

    fn insert_item_to_db(&mut self, goods: &CGoodsNode) -> bool {
        let Some(connection) = self.database.normal_connection.as_mut() else {
            self.report_missing_connection(AuctionWriteOperation::InsertItem);
            return false;
        };
        let runtime = self.runtime.clone();
        let writer = &self.database.writer;
        let registry = &self.registry;
        let outcome = tokio::task::block_in_place(|| {
            runtime.block_on(writer.insert_item_to_db(connection, goods, registry))
        });
        self.write_bool(outcome)
    }

    fn modify_goods_state_a2s(&mut self, goods: &CGoodsNode) -> bool {
        let Some(connection) = self.database.normal_connection.as_mut() else {
            self.report_missing_connection(AuctionWriteOperation::ModifyStateAuctionToSucceeded);
            return false;
        };
        let runtime = self.runtime.clone();
        let writer = &self.database.writer;
        let fields = goods.database_write_fields();
        let outcome = tokio::task::block_in_place(|| {
            runtime.block_on(writer.modify_goods_state_a2s(connection, fields))
        });
        self.write_bool(outcome)
    }

    fn transfer_money(&mut self, goods: &CGoodsNode) -> bool {
        let seller_money_after_fee = (self.callbacks.seller_money_after_fee)(goods);
        let Some(connection) = self.database.normal_connection.as_mut() else {
            self.report_missing_connection(AuctionWriteOperation::TransferSellerMoney);
            return false;
        };
        let runtime = self.runtime.clone();
        let writer = &self.database.writer;
        let fields = goods.database_write_fields();
        let outcome = tokio::task::block_in_place(|| {
            runtime.block_on(writer.transfer_money(
                connection,
                fields,
                seller_money_after_fee,
            ))
        });
        self.write_bool(outcome)
    }

    fn modify_goods_state_a2b(&mut self, goods: &CGoodsNode) -> bool {
        let Some(connection) = self.database.normal_connection.as_mut() else {
            self.report_missing_connection(AuctionWriteOperation::ModifyReturnedGoodsState);
            return false;
        };
        let runtime = self.runtime.clone();
        let writer = &self.database.writer;
        let fields = goods.database_write_fields();
        let outcome = tokio::task::block_in_place(|| {
            runtime.block_on(writer.modify_goods_state_a2b(connection, fields))
        });
        self.write_bool(outcome)
    }

    fn delete_item_from_db(&mut self, guid: CGuid) {
        let runtime = self.runtime.clone();
        let writer = &self.database.writer;
        let outcome = tokio::task::block_in_place(|| {
            runtime.block_on(writer.delete_item_from_db(guid))
        });
        self.report_write(outcome);
    }

    fn delete_money_from_db(&mut self, player_id: i32, money: i32) {
        let Some(connection) = self.database.normal_connection.as_mut() else {
            self.report_missing_connection(AuctionWriteOperation::DeleteMoney);
            return;
        };
        let runtime = self.runtime.clone();
        let writer = &self.database.writer;
        let outcome = tokio::task::block_in_place(|| {
            runtime.block_on(writer.delete_money_from_db(connection, player_id, money))
        });
        self.report_write(outcome);
    }

    fn gold_coin_index(&mut self) -> u32 {
        (self.callbacks.gold_coin_index)()
    }

    fn read_auction_owner_ids(&mut self, destination: &mut VecDeque<i32>) {
        let Some(connection) = self.database.normal_connection.as_mut() else {
            self.report_missing_connection(AuctionWriteOperation::CheckNormalConnection);
            return;
        };
        let runtime = self.runtime.clone();
        let reader = &self.database.reader;
        match tokio::task::block_in_place(|| runtime.block_on(reader.read_owner_ids(connection))) {
            Ok(mut owners) => destination.append(&mut owners),
            Err(error) => (self.callbacks.report_runtime_event)(
                TiberiusDbMiscRuntimeEvent::OwnerListFailed(&error),
            ),
        }
    }

    fn load_goods_by_owner_id(&mut self, owner_id: i32, state: i32, limit: i32) {
        let _ = self.load_owner_auction_goods(owner_id, state, limit);
    }

    fn load_owner_auction_goods(&mut self, owner_id: i32, state: i32, limit: i32) -> i32 {
        let runtime = self.runtime.clone();
        let reader = &self.database.reader;
        let registry = &self.registry;
        let outcome = tokio::task::block_in_place(|| {
            runtime.block_on(reader.load_goods_by_owner_id(
                owner_id,
                GoodsState::from_raw(state),
                limit,
                registry,
            ))
        });
        match outcome {
            AuctionGoodsLoadOutcome::Loaded(mut notes) => {
                let count = i32::try_from(notes.len()).unwrap_or(i32::MAX);
                self.output.append(&mut notes);
                count
            }
            outcome => {
                (self.callbacks.report_runtime_event)(TiberiusDbMiscRuntimeEvent::GoodsLoad(
                    &outcome,
                ));
                0
            }
        }
    }

    fn load_owner_auction_money(&mut self, owner_id: i32, money_limit: i32) {
        let runtime = self.runtime.clone();
        let reader = &self.database.reader;
        let registry = &self.registry;
        let gold_coin_index = (self.callbacks.gold_coin_index)();
        let random = &mut self.callbacks.random;
        let outcome = tokio::task::block_in_place(|| {
            runtime.block_on(reader.load_money_by_id(
                owner_id,
                money_limit,
                gold_coin_index,
                registry,
                random,
            ))
        });
        match outcome {
            AuctionMoneyLoadOutcome::Loaded { mut notes, .. } => self.output.append(&mut notes),
            outcome => (self.callbacks.report_runtime_event)(
                TiberiusDbMiscRuntimeEvent::MoneyLoad(&outcome),
            ),
        }
    }
}
