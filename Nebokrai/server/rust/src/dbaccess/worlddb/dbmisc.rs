//! Очереди аукционной World DB `CDbMisc` и MainLoop-dispatch их результатов.
//!
//! Статус владельца: `IMPLEMENTED` для `DbNote`, constructor/lifecycle очередей,
//! `PushItemToListIn/Out`, `PopItemFromListIn/Out`, трёх
//! `DoneOT_IN_*`, `DoneListIn`, `DoneOutList`, `PopPlayerList` и достигнутой
//! части `LoadAuction`, caller-контрактов `LoadOwnerBackGoods`,
//! `LoadOwnerUndoGoods`, `LoadOwnerSuccGoods` и `LoadMoneyById`, а также
//! Tiberius materialization `LoadGoodsByOwnerId`. Остальные SQL/load-функции
//! ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).
//!
//! Точная пара: `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`,
//! SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.h/.cpp`.
//! Существенные RVA: `DbNote` `0x000A51C0`, constructor `0x000F2100`,
//! push-in/out `0x000F18C0/0x000F1C00`, pop-in/out
//! `0x000F22F0/0x000F2450`, `DoneOutList` `0x000F2650`, input handlers
//! `0x000F2210/0x000F2270/0x000F3560`, `DoneListIn` `0x000F35C0`,
//! `PopPlayerList` `0x000F2C10` и `LoadAuction` `0x000F56F0`.
//!
//! Две `std::list<DbNote*>` и player `std::deque<long>` заменены owning
//! `VecDeque<Box<DbNote>>` под теми же раздельными locks. Push-in сохраняет
//! выбор head/tail, push-out всегда добавляет в хвост, limit `0` снимает всю
//! очередь, положительный limit — не больше указанного числа с головы, а
//! отрицательный ничего не снимает. Возвращаемый `bool` всех четырёх старых
//! queue helpers всегда был `false`; Rust возвращает сам moved batch и не
//! превращает этот неинформативный флаг в успех.
//!
//! `DoneOutList` снимает не больше восьми notes и сохраняет их порядок. Wire
//! `0x14ED01`, `0x80405` и `0x80404` строится готовыми `CMessage`, `CGoodsNode`
//! и `CGoods`; SQL, registry lookup, лог и фактическая send-граница остаются
//! узкому контексту. Terminal auction-result сначала проверяет server по
//! owner, затем online-player и повторно server уже по фактическому inherited
//! player ID. После `0x80404` gold-coin возвращается во входную очередь как
//! `OT_IN_MODIFY_MONEY`, иной товар — как `OT_IN_DELETE_ITEM_SUCESS`.
//!
//! Сохранены две наблюдаемые странности: ошибка A2S маркируется именно
//! `OT_OUT_INSERT_NEW_ITEM_ERROR`, а ошибка A2B получает
//! `OT_OUT_MODIFY_STATE_A2B_ERROR` и возвращается во входной хвост. Нарисованные
//! декомпилятором возвраты из STL-node cleanup не материализованы: иначе batch
//! limit `8` и list/deque loops физически обрабатывали бы только один элемент.
//! Для `DoneListIn` это имеет статус `VERIFIED_DISASSEMBLY`: exact EXE
//! `0x004F36D2/0x004F375B/0x004F3780` сходится в `0x004F3783`, берёт следующий
//! list-node и возвращается к dispatch; единственный normal `ret` расположен
//! после сравнения с sentinel по `0x004F37AD`. После ответа reverse прекращён.
//!
//! Если terminal note содержит пустой `m_vecGoodsByte`, оригинал сначала
//! отправляет `0x80404`, затем читает неинициализированный constructor-ом
//! `CGoods::m_dwBasePropertiesIndex`. Rust сохраняет уже выполненную отправку и
//! возвращает pending owner с `BLOCKED_MISSING_FACT`, не выбирая money/delete,
//! случайное значение либо fail-closed реакцию. Ошибки безопасных goods codecs
//! аналогично возвращают весь ещё не обработанный batch вызывающему. Lock,
//! allocation, COM AddRef, deleting destructors и STL cleanup выражены
//! владением Rust и узкими callbacks, а не отдельной имитацией библиотек.
//! `LoadOwner*Goods` остаются тонкими переходами к одному concrete DB context:
//! они не переупорядочивают SQL read, не интерпретируют join-строки и передают
//! назад именно число `CGoodsNode`, которое необходимо caller-у для следующего
//! остатка лимита. `LoadMoneyById` намеренно не возвращает значение, поскольку
//! его исходный caller его не использует.
//! `TiberiusAuctionGoodsReader` открывает отдельное connection и возвращает
//! полностью собранный batch вместо ADO/COM recordset; async bridge, который
//! append-ит этот batch в общий output FIFO из World MainLoop, остаётся у
//! concrete context и не подменяется блокирующим вызовом драйвера.

use std::collections::{BTreeMap, VecDeque};
use std::error::Error;
use std::fmt;

use encoding_rs::WINDOWS_1251;
use futures_util::TryStreamExt;
use parking_lot::Mutex;
use tiberius::Query;

use crate::dbaccess::worlddb::rssetup::{WorldDatabaseConnectionError, WorldDatabaseSettings};
use crate::nets::networld::message::CMessage;
use crate::public::auctionnode::{
    AuctionDatabaseNodeFields, CGoodsNode, GoodsNodeSerializeError, GoodsState,
};
use crate::public::guid::CGuid;
use crate::worldserver::appworld::goods::cgoods::{CGoods, GoodsCodecError, GoodsLoadedAddonBlock};
use crate::worldserver::appworld::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, create_goods_no_probability,
};

const OUTPUT_BATCH_LIMIT: i32 = 8;
const AUCTION_MAP_ID: u32 = 5;
const MSG_AUCTION_NODE: i32 = 0x0014_ED01;
const MSG_AUCTION_RETURN: i32 = 0x0008_0404;
const MSG_AUCTION_STATE_CHANGED: i32 = 0x0008_0405;
const LOAD_AUCTION_STATE: i32 = 1;
const LOAD_AUCTION_LIMIT: i32 = 100_000;

/// Внутренний discriminant исходного `CDbMisc::OperatorType`.
///
/// Число не выходит в wire и здесь намеренно не переиспользуется как protocol
/// ID: наблюдаемым контрактом являются названные переходы между очередями.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OperatorType {
    OT_NULL,
    OT_IN_INSERT_NEW_ITEM,
    OT_OUT_INSERT_NEW_ITEM_OK,
    OT_OUT_INSERT_NEW_ITEM_ERROR,
    OT_IN_MODIFY_STATE_A2S,
    OT_OUT_MODIFY_STATE_A2S_OK,
    OT_IN_MODIFY_STATE_A2B,
    OT_OUT_MODIFY_STATE_A2B_OK,
    OT_OUT_MODIFY_STATE_A2B_ERROR,
    OT_IN_READ_AUCTION,
    OT_OUT_READ_AUCTION_RESULT,
    OT_IN_DELETE_ITEM_SUCESS,
    OT_IN_DELETE_ITEM_BACK,
    OT_IN_MODIFY_MONEY,
}

/// Владеющая форма старого heap-узла между DB и World MainLoop.
pub(crate) struct DbNote {
    pub(crate) e_type: OperatorType,
    pub(crate) goods: CGoodsNode,
    pub(crate) player_id: i32,
    pub(crate) money: i32,
}

impl Default for DbNote {
    fn default() -> Self {
        Self::new()
    }
}

impl DbNote {
    /// Повторяет `CGoodsNode` constructor + `Clear`, `OT_NULL`, `-1` и `0`.
    pub(crate) fn new() -> Self {
        Self {
            e_type: OperatorType::OT_NULL,
            goods: CGoodsNode::new(),
            player_id: -1,
            money: 0,
        }
    }
}

/// Достигнутые поля результата `CGame::GetPlayerGameServer`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DbMiscGameServer {
    pub(crate) connected: bool,
    pub(crate) index: u32,
}

/// Технические и соседние owner-границы полного `CDbMisc` queue-dispatch.
pub(crate) trait DbMiscContext {
    /// Исходный `CGlobeSetup::m_stSetup.lTransferMoneyTime`.
    fn transfer_money_interval_ms(&self) -> i32;
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

    fn player_game_server(&mut self, player_id: u32) -> Option<DbMiscGameServer>;
    fn online_player_id(&mut self, player_id: u32) -> Option<i32>;
    fn send_to_map_id(&mut self, message: &CMessage, map_id: u32);
    fn log_player_not_online_drop_goods(&mut self);
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

/// Причина остановки только на недоказанной безопасной границе output batch.
#[derive(Debug)]
pub(crate) enum DbMiscDoneOutBlockReason {
    GoodsNodeSerialize(GoodsNodeSerializeError),
    GoodsDecode(GoodsCodecError),
    GoodsSerialize(GoodsCodecError),
    EmptyGoodsBaseIndexAfterDelivery,
}

/// Сохраняет владение текущим и всеми следующими notes после safe-block.
pub(crate) struct DbMiscDoneOutBlock {
    pub(crate) reason: DbMiscDoneOutBlockReason,
    pub(crate) processed_notes: usize,
    pub(crate) pending_notes: VecDeque<Box<DbNote>>,
}

/// Наблюдаемые результаты одного output batch без интерпретации send-result.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DbMiscDoneOutReport {
    pub(crate) processed_notes: usize,
    pub(crate) auction_node_deliveries: usize,
    pub(crate) state_change_deliveries: usize,
    pub(crate) returned_goods_deliveries: usize,
    pub(crate) requeued_input_notes: usize,
    pub(crate) dropped_offline_goods: usize,
}

/// Результат DB-dispatch одного полностью снятого input batch.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct DbMiscDoneInReport {
    pub(crate) processed_notes: usize,
    pub(crate) output_notes: usize,
    pub(crate) requeued_input_notes: usize,
    pub(crate) deleted_items: usize,
    pub(crate) modified_money: usize,
    pub(crate) unhandled_notes: usize,
}

/// Один вызов достигнутой части `LoadAuction`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DbMiscLoadAuctionReport {
    LoadedOwner { owner_id: i32 },
    RefilledOwners { count: usize },
}

/// Владельцы двух note-очередей и двух стадий player-page deque.
pub(crate) struct CDbMisc {
    input: Mutex<VecDeque<Box<DbNote>>>,
    output: Mutex<VecDeque<Box<DbNote>>>,
    player_ids: Mutex<VecDeque<i32>>,
    auction_batch: VecDeque<i32>,
    start_read_player_list: bool,
    transfer_money_previous_ms: u32,
}

impl CDbMisc {
    /// Создаёт пустые очереди и вызывает исходный connection initializer.
    pub(crate) fn new(context: &mut impl DbMiscContext) -> Self {
        context.create_normal_connection();
        Self {
            input: Mutex::new(VecDeque::new()),
            output: Mutex::new(VecDeque::new()),
            player_ids: Mutex::new(VecDeque::new()),
            auction_batch: VecDeque::new(),
            start_read_player_list: false,
            transfer_money_previous_ms: 0,
        }
    }

    /// Добавляет non-null note в head либо tail; старый return всегда `false`.
    pub(crate) fn push_item_to_list_in(&self, note: Box<DbNote>, push_front: bool) -> bool {
        let mut input = self.input.lock();
        if push_front {
            input.push_front(note);
        } else {
            input.push_back(note);
        }
        false
    }

    /// Добавляет non-null note только в output tail; return остаётся `false`.
    pub(crate) fn push_item_to_list_out(&self, note: Box<DbNote>) -> bool {
        self.output.lock().push_back(note);
        false
    }

    /// Снимает input batch после точного transfer-money/reconnect gate.
    pub(crate) fn pop_item_from_list_in(
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

    /// Снимает output batch без DB connection gate.
    pub(crate) fn pop_item_from_list_out(&self, limit: i32) -> VecDeque<Box<DbNote>> {
        move_queue_batch(&mut self.output.lock(), limit)
    }

    /// Выполняет три DB handlers и три terminal input-перехода в list-order.
    pub(crate) fn done_list_in(
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
                        // World RVA 0x000F2210 действительно использует чужой
                        // insert-error discriminant.
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
                    // Exact EXE 0x004F3610..0x004F3620 направляет неизвестный
                    // discriminant прямо к общей итерации 0x004F3783.
                    report.unhandled_notes += 1;
                }
            }
        }
        report
    }

    /// Снимает максимум восемь output notes и исполняет полный World dispatch.
    pub(crate) fn done_out_list(
        &self,
        context: &mut impl DbMiscContext,
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

    /// Выполняет достигнутый direct refresh owner-ID списка.
    pub(crate) fn done_ot_in_read_auction(&self, context: &mut impl DbMiscContext) {
        if !context.is_active_connect() {
            return;
        }
        let mut player_ids = self.player_ids.lock();
        player_ids.clear();
        context.read_auction_owner_ids(&mut player_ids);
    }

    /// Снимает player-ID по тому же limit-контракту отдельной player lock.
    pub(crate) fn pop_player_list(&self, limit: i32) -> VecDeque<i32> {
        move_queue_batch(&mut self.player_ids.lock(), limit)
    }

    /// Читает не больше одного owner-а либо переносит весь новый owner batch.
    pub(crate) fn load_auction(
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

    /// Exact общий owner-read, которому принадлежат state и limit аргументы.
    ///
    /// Materialization строк `Auction/AuctionGoods` и публикация output note
    /// остаются в concrete DB context; `CDbMisc` сохраняет сам caller-contract
    /// и число созданных `CGoodsNode`.
    pub(crate) fn load_goods_by_owner_id(
        &self,
        context: &mut impl DbMiscContext,
        owner_id: i32,
        state: GoodsState,
        limit: i32,
    ) -> i32 {
        context.load_owner_auction_goods(owner_id, state.raw(), limit)
    }

    /// Exact `LoadOwnerBackGoods(owner, limit)`, state `3`.
    pub(crate) fn load_owner_back_goods(
        &self,
        context: &mut impl DbMiscContext,
        owner_id: i32,
        limit: i32,
    ) -> i32 {
        self.load_goods_by_owner_id(context, owner_id, GoodsState::BACK, limit)
    }

    /// Exact `LoadOwnerUndoGoods(owner, limit)`, state `4`.
    pub(crate) fn load_owner_undo_goods(
        &self,
        context: &mut impl DbMiscContext,
        owner_id: i32,
        limit: i32,
    ) -> i32 {
        self.load_goods_by_owner_id(context, owner_id, GoodsState::UNDO, limit)
    }

    /// Exact `LoadOwnerSuccGoods(owner, limit)`, state `2`.
    pub(crate) fn load_owner_succ_goods(
        &self,
        context: &mut impl DbMiscContext,
        owner_id: i32,
        limit: i32,
    ) -> i32 {
        self.load_goods_by_owner_id(context, owner_id, GoodsState::SUCESSED, limit)
    }

    /// Exact `LoadMoneyById`; original caller намеренно игнорировал return.
    pub(crate) fn load_money_by_id(
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

/// Ошибка достигнутой Tiberius-границы `CDbMisc::LoadGoodsByOwnerId`.
#[derive(Debug)]
pub(crate) enum AuctionGoodsLoadFailure {
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
pub(crate) enum AuctionGoodsLoadBlock {
    GoodsGuid {
        row_index: usize,
        source: uuid::Error,
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

/// Результат одного exact read `Auction/AuctionGoods`.
///
/// Успех возвращает уже готовые `OT_OUT_READ_AUCTION_RESULT` notes в порядке
/// сырого `CGUID`; concrete World context обязан append-нуть их в output FIFO
/// без перемешивания с другими producer-ами.
pub(crate) enum AuctionGoodsLoadOutcome {
    Loaded(VecDeque<Box<DbNote>>),
    ReturnedFalse(AuctionGoodsLoadFailure),
    BlockedMissingFact(AuctionGoodsLoadBlock),
}

/// Linux/TDS-реализация точного DB read для возврата/публикации товаров
/// аукциона.
///
/// В отличие от старого ADO `OpenRs`, каждый вызов открывает владимое Tiberius
/// connection. SQL сохраняет исходные `NOLOCK`, join и единственный `ORDER BY`;
/// `limit` намеренно не превращён в `TOP`, поскольку exact owner ограничивал
/// уже созданные уникальные `GoodsID` после чтения joined rows.
pub(crate) struct TiberiusAuctionGoodsReader {
    settings: WorldDatabaseSettings,
}

impl TiberiusAuctionGoodsReader {
    pub(crate) fn new(settings: &WorldDatabaseSettings) -> Self {
        Self {
            settings: settings.clone(),
        }
    }

    /// Materializes `Auction` + `AuctionGoods` в будущий output batch.
    ///
    /// `dakong_addon_types` принадлежит уже загруженному setup owner-у. Его
    /// caller передаёт как snapshot, чтобы DB read не создавал второй mutable
    /// глобальный singleton.
    pub(crate) async fn load_goods_by_owner_id(
        &self,
        owner_id: i32,
        state: GoodsState,
        limit: i32,
        registry: &GoodsBasePropertiesRegistry,
        dakong_addon_types: &std::collections::BTreeSet<i32>,
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
            "SELECT a.*, CONVERT(int,b.type) AS type, b.modifierValue1, b.modifierValue2 \
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

        let mut row_index = 0usize;
        let mut pending = BTreeMap::<CGuid, PendingAuctionGoods>::new();
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
                // Exact `if (param_3 <= map._Mysize) break`: current joined
                // row is not consumed into a new note once the unique-GUID
                // bound has been reached. A non-positive legacy limit is
                // therefore an empty page, after the same connection/query.
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
                    dakong_addon_types,
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
            auction_time: required!(i32, "dwauctiontime") as u32,
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
            time_buyer: required!(i32, "TimeBuyer") as u32,
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

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp

// IMPLEMENTED_OWNER: CDbMisc::DbNote::DbNote находится в typed Rust-владельце выше.

// ============================================================================
// FUNCTION: CDbMisc::CreateNormalCn
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:1799
// RVA: 0x000EF7F0
// ADDRESS: 004ef7f0
// PROTOTYPE: bool __thiscall CreateNormalCn(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004ef893
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:1810
// RVA: 0x000EF893
// ADDRESS: 004ef893
// PROTOTYPE: undefined Catch@004ef893()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDbMisc::DelItemFromDb
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:673
// RVA: 0x000EF930
// ADDRESS: 004ef930
// PROTOTYPE: bool __thiscall DelItemFromDb(CGUID param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004efa3f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:694
// RVA: 0x000EFA3F
// ADDRESS: 004efa3f
// PROTOTYPE: undefined Catch@004efa3f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDbMisc::DelMoneyFromDb
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:703
// RVA: 0x000EFAB0
// ADDRESS: 004efab0
// PROTOTYPE: bool __thiscall DelMoneyFromDb(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004efb3f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:719
// RVA: 0x000EFB3F
// ADDRESS: 004efb3f
// PROTOTYPE: undefined Catch@004efb3f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDbMisc::ModifyGoodsStateA2S
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:922
// RVA: 0x000EFB80
// ADDRESS: 004efb80
// PROTOTYPE: bool __thiscall ModifyGoodsStateA2S(CGoodsNode * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004efc68
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:951
// RVA: 0x000EFC68
// ADDRESS: 004efc68
// PROTOTYPE: undefined Catch@004efc68()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004efc89
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:955
// RVA: 0x000EFC89
// ADDRESS: 004efc89
// PROTOTYPE: undefined FUN_004efc89()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDbMisc::TansferMoney
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:959
// RVA: 0x000EFCC0
// ADDRESS: 004efcc0
// PROTOTYPE: bool __thiscall TansferMoney(CGoodsNode * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004efdb5
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:990
// RVA: 0x000EFDB5
// ADDRESS: 004efdb5
// PROTOTYPE: undefined Catch@004efdb5()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDbMisc::ModifyGoodsStateA2B
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:999
// RVA: 0x000EFDE0
// ADDRESS: 004efde0
// PROTOTYPE: bool __thiscall ModifyGoodsStateA2B(CGoodsNode * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004efec0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:1026
// RVA: 0x000EFEC0
// ADDRESS: 004efec0
// PROTOTYPE: undefined Catch@004efec0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004efee1
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:1030
// RVA: 0x000EFEE1
// ADDRESS: 004efee1
// PROTOTYPE: undefined FUN_004efee1()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDbMisc::IsActiveConnect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:1829
// RVA: 0x000EFF10
// ADDRESS: 004eff10
// PROTOTYPE: bool __thiscall IsActiveConnect(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004eff84
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:1839
// RVA: 0x000EFF84
// ADDRESS: 004eff84
// PROTOTYPE: undefined Catch@004eff84()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004eff9f
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:1844
// RVA: 0x000EFF9F
// ADDRESS: 004eff9f
// PROTOTYPE: undefined FUN_004eff9f()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDbMisc::SaveGoodsProperties
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:1682
// RVA: 0x000F02C0
// ADDRESS: 004f02c0
// PROTOTYPE: bool __thiscall SaveGoodsProperties(vector<CGoods::tagAddonProperty,std::allocator<CGoods::tagAddonProperty>_> * param_1, CGoodsBaseProperties * param_2, char * param_3, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED_OWNER: CDbMisc::~CDbMisc находится в typed Rust-владельце выше.

// IMPLEMENTED_OWNER: CDbMisc::PushItemToListIn находится в typed Rust-владельце выше.

// IMPLEMENTED_OWNER: CDbMisc::DoneOT_IN_READ_AUCTION находится в typed Rust-владельце выше.

// ============================================================================
// FUNCTION: Catch@004f1b7c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:295
// RVA: 0x000F1B7C
// ADDRESS: 004f1b7c
// PROTOTYPE: undefined Catch@004f1b7c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED_OWNER: CDbMisc::PushItemToListOut находится в typed Rust-владельце выше.

// ============================================================================
// FUNCTION: CDbMisc::LoadMoneyById
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:1566
// RVA: 0x000F1C60
// ADDRESS: 004f1c60
// PROTOTYPE: long __thiscall LoadMoneyById(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004f204c
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:1649
// RVA: 0x000F204C
// ADDRESS: 004f204c
// PROTOTYPE: undefined Catch@004f204c()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED_OWNER: CDbMisc::CDbMisc находится в typed Rust-владельце выше.

// IMPLEMENTED_OWNER: CDbMisc::DoneOT_IN_MODIFY_STATE_A2S находится в typed Rust-владельце выше.

// IMPLEMENTED_OWNER: CDbMisc::DoneOT_IN_MODIFY_STATE_A2B находится в typed Rust-владельце выше.

// IMPLEMENTED_OWNER: CDbMisc::PopItemFromListIn находится в typed Rust-владельце выше.

// IMPLEMENTED_OWNER: CDbMisc::PopItemFromListOut находится в typed Rust-владельце выше.

// IMPLEMENTED_OWNER: CDbMisc::DoneOutList находится в typed Rust-владельце выше.

// IMPLEMENTED_OWNER: CDbMisc::PopPlayerList находится в typed Rust-владельце выше.

// ============================================================================
// FUNCTION: CDbMisc::InsertItemToDb
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:729
// RVA: 0x000F2EC0
// ADDRESS: 004f2ec0
// PROTOTYPE: bool __thiscall InsertItemToDb(CGoodsNode * param_1, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004f33db
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:745
// RVA: 0x000F33DB
// ADDRESS: 004f33db
// PROTOTYPE: undefined Catch@004f33db()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED_OWNER: CDbMisc::DoneOT_IN_INSERT_NEW_ITEM находится в typed Rust-владельце выше.

// IMPLEMENTED_OWNER: CDbMisc::DoneListIn находится в typed Rust-владельце выше.

// ============================================================================
// FUNCTION: CDbMisc::LoadGoodsByOwnerId
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:1035
// RVA: 0x000F37E0
// ADDRESS: 004f37e0
// PROTOTYPE: long __thiscall LoadGoodsByOwnerId(long param_1, GoodsState param_2, long param_3, _com_ptr_t<_com_IIID<_Connection,&struct___s_GUID_const__GUID_00000550_0000_0010_8000_00aa006d2ea4>_> param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004f52c8
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:1327
// RVA: 0x000F52C8
// ADDRESS: 004f52c8
// PROTOTYPE: undefined Catch@004f52c8()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: FUN_004f5640
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:1372
// RVA: 0x000F5640
// ADDRESS: 004f5640
// PROTOTYPE: undefined FUN_004f5640()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDbMisc::LoadOwnerBackGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:1661
// RVA: 0x000F5660
// ADDRESS: 004f5660
// PROTOTYPE: long __thiscall LoadOwnerBackGoods(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDbMisc::LoadOwnerUndoGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:1667
// RVA: 0x000F5690
// ADDRESS: 004f5690
// PROTOTYPE: long __thiscall LoadOwnerUndoGoods(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CDbMisc::LoadOwnerSuccGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\dbaccess\worlddb\dbmisc.cpp:1673
// RVA: 0x000F56C0
// ADDRESS: 004f56c0
// PROTOTYPE: long __thiscall LoadOwnerSuccGoods(long param_1, long param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// IMPLEMENTED_OWNER: CDbMisc::LoadGoodsByOwnerId materialized выше как
// TiberiusAuctionGoodsReader; RAW сохранён как доказательство query, limit,
// joined-row и GUID-order семантики. CDbMisc::LoadAuction находится в typed
// Rust-владельце выше.

// COMPONENT_VARIANT_END: WorldServer
