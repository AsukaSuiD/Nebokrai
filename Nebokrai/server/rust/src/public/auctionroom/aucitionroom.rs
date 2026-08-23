//! Комната аукциона `CAuctionRoom` MiscServer и GameServer.
//! Источник контракта — точные пары MiscServer/GameServer EXE/PDB;
//! общий goods wire согласован с WorldServer owner-ом.
//!
//! Owner хранит основные и вторичные ordered списки, player search state,
//! opt/del/success/back queues и выполняет `AI` в исходном порядке. Add/delete
//! ветви сохраняют последовательность мутаций, return flags, duplicate rules
//! и partial effects; дополнительные rollback, сортировка и retry не вводятся.
//! `BTreeMap`, `VecDeque` и owned nodes заменяют MSVC containers и ручной
//! lifetime без изменения auction opcodes или page/filter semantics.

use std::collections::{BTreeMap, VecDeque};

use rustix::time::{ClockId, clock_gettime};

use crate::nets::netmisc::message::{CMessage, MessageSender, SendMessageError};

use super::auctionnode::{CGoodsNode, GoodsNodeSerializeError, GoodsState};
use super::auctionroom::PlayerOptNode;
use super::guid::CGuid;

/// Неопределённая граница старого неинициализированного `m_btGoodsType`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct AddAuctionItemMissingGoodsType {
    pub(crate) guid: CGuid,
}

/// Неопределённая type-граница `ClearRecond` после доказанных time/owner эффектов.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ClearRecondMissingGoodsType {
    pub(crate) guid: CGuid,
}

/// Неопределённая duplicate-граница вставки owned-узла в конечный список.
pub(crate) struct AddTerminalGoodsDuplicate {
    pub(crate) guid: CGuid,
    pub(crate) item: Box<CGoodsNode>,
}

/// Локальные safe-границы полного прохода `DoneDelList`.
pub(crate) enum DoneDelListError {
    MissingGoodsType(ClearRecondMissingGoodsType),
    DuplicateSucessed(AddTerminalGoodsDuplicate),
    DuplicateBack(AddTerminalGoodsDuplicate),
    DefaultStateDanglingPointer { guid: CGuid, state: i32 },
}

/// Результат сериализации и исходно игнорировавшейся отправки terminal-узла.
#[derive(Debug)]
pub(crate) enum TerminalGoodsDelivery {
    /// Safe-сериализация остановилась на недоказанном старом поле или размере.
    SerializeBlocked(GoodsNodeSerializeError),
    /// Пакет построен; результат nullable client/send сохранён для владельца.
    Sent(Result<i32, SendMessageError>),
}

/// Итог одного уже удалённого terminal-узла в исходном GUID-порядке.
#[derive(Debug)]
pub(crate) struct TerminalGoodsDispatch {
    pub(crate) guid: CGuid,
    pub(crate) delivery: TerminalGoodsDelivery,
}

/// Итог одного полного прохода исходного `CAuctionRoom::AI`.
pub(crate) struct AuctionAiOutcome {
    /// Результат полного deletion drain после уже выполненного `DoneAuction`.
    pub(crate) deletion: Result<(), DoneDelListError>,
    /// Все sucessed-узлы, снятые текущим проходом в GUID-порядке.
    pub(crate) sucessed: Vec<TerminalGoodsDispatch>,
    /// Не более одного минимального back-узла текущего прохода.
    pub(crate) back: Option<TerminalGoodsDispatch>,
}

const AUCTION_UNITY_ITEM: i32 = 0x0015_EB03;
const AUCTION_UNITY_INDEX: i32 = 0x0015_EB04;
const AUCTION_SUCCESSED_RESULT: i32 = 0x0015_EB01;
const AUCTION_BACK_RESULT: i32 = 0x0015_EB02;
const AUCTION_UNITY_DETAIL_LIMIT: usize = 20;

/// Построенный в исходном порядке batch `UnityGoods` и локальная safe-граница.
pub(crate) struct UnityGoodsBuild {
    pub(crate) messages: Vec<CMessage>,
    pub(crate) blocked: Option<GoodsNodeSerializeError>,
}

/// Safe-граница построения одной страницы старого аукционного поиска.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AuctionPageBuildError {
    GoodsCountOutsideLegacyRange { length: usize },
    MissingGoodsType,
    MissingLevelLimit,
    LegacyStringWithoutTerminator { field: &'static str },
    Serialize(GoodsNodeSerializeError),
}

/// Owned-состояние исходного `CAuctionRoom`.
///
/// Параметр сохраняет действующие constructor/clear call sites. Доменное
/// добавление и индексы определены для доказанного `CGoodsNode`;
/// GameServer использует тот же primary ordered map для `CountGoods`.
pub(crate) struct CAuctionRoom<GoodsNode> {
    auction_goods_list: BTreeMap<CGuid, Box<GoodsNode>>,
    time_ticket: BTreeMap<u32, Vec<CGuid>>,
    owner_list: BTreeMap<i32, Vec<CGuid>>,
    type_list: BTreeMap<u8, Vec<CGuid>>,
    del_from_goods_list: VecDeque<Vec<CGuid>>,
    sucessed_goods_list: BTreeMap<CGuid, Box<GoodsNode>>,
    back_goods_list: BTreeMap<CGuid, Box<GoodsNode>>,
    player_search_list: BTreeMap<u32, PlayerOptNode>,
    opt_list: BTreeMap<CGuid, u32>,
}

impl<GoodsNode> Default for CAuctionRoom<GoodsNode> {
    fn default() -> Self {
        Self::new()
    }
}

impl<GoodsNode> CAuctionRoom<GoodsNode> {
    /// Создаёт все исходные ordered-контейнеры и очередь пустыми.
    pub(crate) fn new() -> Self {
        Self {
            auction_goods_list: BTreeMap::new(),
            time_ticket: BTreeMap::new(),
            owner_list: BTreeMap::new(),
            type_list: BTreeMap::new(),
            del_from_goods_list: VecDeque::new(),
            sucessed_goods_list: BTreeMap::new(),
            back_goods_list: BTreeMap::new(),
            player_search_list: BTreeMap::new(),
            opt_list: BTreeMap::new(),
        }
    }

    /// Копирует GUID primary map в исходном `std::map`-порядке.
    /// Это exact `CAuctionRoom::CountGoods`, вызываемый `CGame::RunAuction`.
    pub(crate) fn count_goods(&self) -> Vec<CGuid> {
        self.auction_goods_list.keys().copied().collect()
    }

    /// Очищает семь owned-контейнеров в точном порядке `Clear`.
    ///
    /// Незавершённые операции и поиски игроков сохраняются: исходный `Clear`
    /// оставлял `m_mapOptList` и `m_mapPlayerSearchList` нетронутыми.
    pub(crate) fn clear(&mut self) {
        self.clear_auction_list();
        self.clear_del_list();
        self.clear_sucessed_list();
        self.clear_back_list();
        self.clear_time_list();
        self.clear_owner_list();
        self.clear_type_list();

        // MiscServer не очищает `m_mapOptList`: его пары
        // `CGUID -> unsigned long operation` сохраняются буквально до
        // `DoneOptList` либо destructor комнаты.
    }

    fn clear_auction_list(&mut self) {
        self.auction_goods_list.clear();
    }

    fn clear_sucessed_list(&mut self) {
        self.sucessed_goods_list.clear();
    }

    fn clear_back_list(&mut self) {
        self.back_goods_list.clear();
    }

    fn clear_time_list(&mut self) {
        self.time_ticket.clear();
    }

    fn clear_owner_list(&mut self) {
        self.owner_list.clear();
    }

    fn clear_type_list(&mut self) {
        self.type_list.clear();
    }

    fn clear_del_list(&mut self) {
        self.del_from_goods_list.clear();
    }
}

impl CAuctionRoom<CGoodsNode> {
    /// Добавляет owned MiscServer-узел и обновляет три вторичных индекса.
    ///
    /// Invalid либо уже существующий GUID дают исходный `false`, уничтожают
    /// новый узел и wrapping увеличивают `m_dwDelNewCount`. Ошибка означает
    /// только старое неинициализированное поле типа: main/time/owner эффекты к
    /// этой границе уже совершены, как требует доказанный порядок оригинала.
    pub(crate) fn add_item_to_auction_room(
        &mut self,
        item: Box<CGoodsNode>,
        deleted_new_count: &mut u32,
    ) -> Result<bool, AddAuctionItemMissingGoodsType> {
        let guid = item.guid();
        if guid == CGuid::GUID_INVALID || self.auction_goods_list.contains_key(&guid) {
            *deleted_new_count = deleted_new_count.wrapping_add(1);
            return Ok(false);
        }

        self.auction_goods_list.insert(guid, item);
        let item = self
            .auction_goods_list
            .get_mut(&guid)
            .expect("только что вставленный auction-узел должен существовать");
        item.mark_as_auction();
        let add_ticket = item.add_ticket();
        let owner_id = item.owner_id() as i32;
        let goods_type = item.goods_type();

        self.time_ticket.entry(add_ticket).or_default().push(guid);
        self.owner_list.entry(owner_id).or_default().push(guid);
        let Some(goods_type) = goods_type else {
            // typed boundary: MiscServer читал
            // `m_btGoodsType`, который constructor и `Clear` не задавали:
            // Тип товара берётся из m_btGoodsType и передаётся в PushItemToTypeList.
            // Какой byte наблюдался для такого не-UnSerialize узла, неизвестно.
            return Err(AddAuctionItemMissingGoodsType { guid });
        };
        self.type_list.entry(goods_type).or_default().push(guid);
        Ok(true)
    }

    /// Возвращает Misc-узел primary auction map либо `None`.
    pub(crate) fn query_goods_node_info(&self, guid: CGuid) -> Option<&CGoodsNode> {
        if guid == CGuid::GUID_INVALID {
            return None;
        }
        self.auction_goods_list.get(&guid).map(Box::as_ref)
    }

    /// Возвращает 32-битный `_Mysize` primary map для Misc diagnostic-owner.
    pub(crate) fn legacy_auction_goods_count(&self) -> u32 {
        self.auction_goods_list.len() as u32
    }

    /// Ставит GUID в очередь операции и обновляет buyer исходного узла.
    ///
    /// Возвращаемое значение всегда `false`, включая успешную мутацию: это
    /// доказанный контракт оригинал MiscServer, на который опирается handler.
    pub(crate) fn push_item_to_opt_list(
        &mut self,
        guid: CGuid,
        operation: u32,
        player_id: i32,
    ) -> bool {
        if self.opt_list.contains_key(&guid) {
            return false;
        }

        if guid == CGuid::GUID_INVALID {
            return false;
        }
        let Some(item) = self.auction_goods_list.get_mut(&guid).map(Box::as_mut) else {
            return false;
        };
        if item.auction_buyer_id() != 0 {
            return false;
        }

        let buyer_id = if operation == 1 {
            item.owner_id()
        } else {
            player_id as u32
        };
        item.set_auction_buyer_id(buyer_id);
        self.opt_list.insert(guid, operation);

        // Контракт: Misc выполняет мутации выше,
        // но по завершает все пути `xor al, al; ret 0x18`.
        false
    }

    /// Добавляет живой GUID в первый batch очереди удаления.
    fn add_item_to_del_list(&mut self, guid: CGuid) -> bool {
        if guid == CGuid::GUID_INVALID {
            return false;
        }

        if let Some(front) = self.del_from_goods_list.front_mut() {
            front.push(guid);
        } else {
            self.del_from_goods_list.push_back(vec![guid]);
        }
        true
    }

    /// Ставит auction-товар в обычную очередь отмены.
    fn del_item_from_auction_room(&mut self, guid: CGuid) -> bool {
        if guid == CGuid::GUID_INVALID {
            return false;
        }
        let Some(item) = self.auction_goods_list.get_mut(&guid).map(Box::as_mut) else {
            return false;
        };
        if !item.is_auction() {
            return false;
        }

        item.mark_as_undo();
        let _ = self.add_item_to_del_list(guid);
        true
    }

    /// Ставит auction-товар в очередь успешной покупки.
    fn del_item_from_auction_room_for_sucessed(&mut self, guid: CGuid) -> bool {
        if guid == CGuid::GUID_INVALID {
            return false;
        }
        let Some(item) = self.auction_goods_list.get_mut(&guid).map(Box::as_mut) else {
            return false;
        };
        if !item.is_auction() {
            return false;
        }

        item.mark_as_sucessed();
        let _ = self.add_item_to_del_list(guid);
        true
    }

    /// Ставит auction-товар в очередь предварительной покупки.
    fn del_item_from_auction_room_by_pre_buy(&mut self, guid: CGuid) -> bool {
        if guid == CGuid::GUID_INVALID {
            return false;
        }
        let Some(item) = self.auction_goods_list.get_mut(&guid).map(Box::as_mut) else {
            return false;
        };
        if !item.is_auction() {
            return false;
        }

        item.mark_as_pre_buy();
        let _ = self.add_item_to_del_list(guid);
        true
    }

    /// Полностью обрабатывает текущий ordered opt-list от минимального GUID.
    pub(crate) fn done_opt_list(&mut self) {
        while let Some((&guid, &operation)) = self.opt_list.first_key_value() {
            match operation {
                1 => {
                    let _ = self.del_item_from_auction_room(guid);
                }
                2 => {
                    let _ = self.del_item_from_auction_room_for_sucessed(guid);
                }
                3 => {
                    let _ = self.del_item_from_auction_room_by_pre_buy(guid);
                }
                _ => {}
            }
            let _ = self.opt_list.remove(&guid);
        }
    }

    /// Обрабатывает opt-list и переносит истёкший time-prefix в очередь удаления.
    pub(crate) fn done_auction(&mut self) {
        self.done_opt_list();
        let now = clock_gettime(ClockId::Realtime).tv_sec as u32;

        while let Some((&ticket, _)) = self.time_ticket.first_key_value() {
            if ticket > now {
                break;
            }

            let batch = {
                let guids = self
                    .time_ticket
                    .get_mut(&ticket)
                    .expect("минимальный time-ticket должен существовать");
                std::mem::take(guids)
            };
            self.del_from_goods_list.push_back(batch);
            let _ = self.time_ticket.remove(&ticket);
        }
    }

    /// Удаляет первое совпадение из временного индекса и удаляет пустой key.
    fn del_item_from_time_list(&mut self, ticket: u32, guid: CGuid) -> bool {
        let (removed, became_empty) = {
            let Some(items) = self.time_ticket.get_mut(&ticket) else {
                return false;
            };
            let removed = items
                .iter()
                .position(|item_guid| *item_guid == guid)
                .map(|position| items.remove(position))
                .is_some();
            (removed, items.is_empty())
        };

        if became_empty {
            let _ = self.time_ticket.remove(&ticket);
        }
        removed
    }

    /// Удаляет первое совпадение из owner-индекса и удаляет пустой key.
    fn del_item_from_owner_list(&mut self, owner_id: i32, guid: CGuid) -> bool {
        let (removed, became_empty) = {
            let Some(items) = self.owner_list.get_mut(&owner_id) else {
                return false;
            };
            let removed = items
                .iter()
                .position(|item_guid| *item_guid == guid)
                .map(|position| items.remove(position))
                .is_some();
            (removed, items.is_empty())
        };

        if became_empty {
            let _ = self.owner_list.remove(&owner_id);
        }
        removed
    }

    /// Удаляет первое совпадение из type-индекса, сохраняя существующий key.
    fn del_item_from_type_list(&mut self, goods_type: u8, guid: CGuid) -> bool {
        let Some(items) = self.type_list.get_mut(&goods_type) else {
            return false;
        };
        if let Some(position) = items.iter().position(|item_guid| *item_guid == guid) {
            items.remove(position);
        }
        true
    }

    /// Удаляет GUID живого primary-узла из трёх вторичных индексов.
    ///
    /// Ошибка означает неинициализированный старый goods type: time и owner к
    /// этой границе уже обработаны в исходном порядке.
    fn clear_recond(&mut self, guid: CGuid) -> Result<(), ClearRecondMissingGoodsType> {
        let Some(item) = self.auction_goods_list.get(&guid).map(Box::as_ref) else {
            return Ok(());
        };
        let add_ticket = item.add_ticket();
        let owner_id = item.owner_id() as i32;
        let goods_type = item.goods_type();

        let _ = self.del_item_from_time_list(add_ticket, guid);
        let _ = self.del_item_from_owner_list(owner_id, guid);
        let Some(goods_type) = goods_type else {
            // typed boundary: MiscServer читал byte
            // `m_btGoodsType` до вызовов, хотя constructor/`Clear` его не
            // задавали. Какой type-key удалялся для такого узла, неизвестно.
            return Err(ClearRecondMissingGoodsType { guid });
        };
        let _ = self.del_item_from_type_list(goods_type, guid);
        Ok(())
    }

    /// Извлекает живой primary-узел после очистки его вторичных индексов.
    ///
    /// Ошибка сохраняет узел в primary map; time/owner эффекты `ClearRecond` к
    /// этой границе уже могли быть выполнены.
    fn pop_item_from_goods_list(
        &mut self,
        guid: CGuid,
    ) -> Result<Option<Box<CGoodsNode>>, ClearRecondMissingGoodsType> {
        if guid == CGuid::GUID_INVALID || !self.auction_goods_list.contains_key(&guid) {
            return Ok(None);
        }

        self.clear_recond(guid)?;
        Ok(self.auction_goods_list.remove(&guid))
    }

    /// Передаёт owned-узел в список успешно завершённых аукционов.
    ///
    /// При duplicate key существующая запись сохраняется, а входной узел
    /// возвращается внутри ошибки: старую утечку safe Rust не воспроизводит.
    fn add_item_to_sucessed_list(
        &mut self,
        item: Box<CGoodsNode>,
    ) -> Result<bool, AddTerminalGoodsDuplicate> {
        let guid = item.guid();
        match self.sucessed_goods_list.entry(guid) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(item);
                Ok(true)
            }
            std::collections::btree_map::Entry::Occupied(_) => {
                // typed boundary: MiscServer игнорировал
                // result `std::map::insert`, возвращал `true` по non-null
                // pointer и оставлял новый узел без доказанного владельца.
                Err(AddTerminalGoodsDuplicate { guid, item })
            }
        }
    }

    /// Передаёт owned-узел в список возврата продавцу.
    ///
    /// При duplicate key существующая запись сохраняется, а входной узел
    /// возвращается внутри ошибки: старую утечку safe Rust не воспроизводит.
    fn add_item_to_back_list(
        &mut self,
        item: Box<CGoodsNode>,
    ) -> Result<bool, AddTerminalGoodsDuplicate> {
        let guid = item.guid();
        match self.back_goods_list.entry(guid) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(item);
                Ok(true)
            }
            std::collections::btree_map::Entry::Occupied(_) => {
                // typed boundary: MiscServer имеет ту же
                // unchecked insert-ветвь, поэтому duplicate ownership не
                // получает придуманного уничтожения, замены либо leak.
                Err(AddTerminalGoodsDuplicate { guid, item })
            }
        }
    }

    /// Полностью дренирует очередь удаления до первой safe-границы.
    ///
    /// Missing goods type сохраняет текущий GUID в начале batch. Ошибки
    /// duplicate/default-state возникают уже после доказанного удаления этого
    /// GUID; содержащийся в ошибке owned-узел не теряется.
    pub(crate) fn done_del_list(&mut self) -> Result<(), DoneDelListError> {
        loop {
            while self.del_from_goods_list.front().is_some_and(Vec::is_empty) {
                let _ = self.del_from_goods_list.pop_front();
            }

            let Some(guid) = self
                .del_from_goods_list
                .front()
                .and_then(|batch| batch.first())
                .copied()
            else {
                return Ok(());
            };

            match self.process_del_guid(guid) {
                Ok(()) => self.consume_first_del_guid(),
                Err(error @ DoneDelListError::MissingGoodsType(_)) => return Err(error),
                Err(error) => {
                    self.consume_first_del_guid();
                    return Err(error);
                }
            }
        }
    }

    /// Полностью дренирует sucessed-map и отправляет по одному `0x15EB01`.
    ///
    /// Любой результат сериализации или отправки относится к уже удалённому
    /// узлу и не останавливает последующие GUID текущего прохода.
    pub(crate) fn done_sucessed_goods_list(
        &mut self,
        sender: Option<&dyn MessageSender>,
    ) -> Vec<TerminalGoodsDispatch> {
        let mut outcomes = Vec::with_capacity(self.sucessed_goods_list.len());
        while let Some((&guid, item)) = self.sucessed_goods_list.first_key_value() {
            let delivery = Self::deliver_terminal_goods(item, AUCTION_SUCCESSED_RESULT, sender);
            let item = self
                .sucessed_goods_list
                .remove(&guid)
                .expect("минимальный sucessed-узел должен существовать");
            drop(item);
            outcomes.push(TerminalGoodsDispatch { guid, delivery });
        }
        outcomes
    }

    /// Обрабатывает только один минимальный back-key через `0x15EB02`.
    ///
    /// Следующие записи намеренно остаются внешнему вызову `AI`; результат
    /// сериализации либо отправки не возвращает уже снятый узел в map.
    pub(crate) fn done_back_list(
        &mut self,
        sender: Option<&dyn MessageSender>,
    ) -> Option<TerminalGoodsDispatch> {
        let (&guid, item) = self.back_goods_list.first_key_value()?;
        let delivery = Self::deliver_terminal_goods(item, AUCTION_BACK_RESULT, sender);
        let item = self
            .back_goods_list
            .remove(&guid)
            .expect("минимальный back-узел должен существовать");
        drop(item);
        Some(TerminalGoodsDispatch { guid, delivery })
    }

    fn deliver_terminal_goods(
        item: &CGoodsNode,
        message_type: i32,
        sender: Option<&dyn MessageSender>,
    ) -> TerminalGoodsDelivery {
        let serialized = match item.serialize() {
            Ok(serialized) => serialized,
            Err(error) => return TerminalGoodsDelivery::SerializeBlocked(error),
        };
        let mut message = CMessage::new(message_type);
        message.base_mut().add(&serialized);
        TerminalGoodsDelivery::Sent(message.send(sender, false))
    }

    /// Выполняет четыре стадии аукционной обработки в исходном порядке.
    ///
    /// Safe-ошибка deletion drain сохраняется в отчёте и не отменяет две
    /// следующие независимые terminal-стадии текущего прохода.
    pub(crate) fn ai(&mut self, sender: Option<&dyn MessageSender>) -> AuctionAiOutcome {
        self.done_auction();
        let deletion = self.done_del_list();
        let sucessed = self.done_sucessed_goods_list(sender);
        let back = self.done_back_list(sender);
        AuctionAiOutcome {
            deletion,
            sucessed,
            back,
        }
    }

    fn process_del_guid(&mut self, guid: CGuid) -> Result<(), DoneDelListError> {
        let Some(item) = self.auction_goods_list.get(&guid).map(Box::as_ref) else {
            return Ok(());
        };
        let state = item.goods_state();
        let offer_price = item.offer_price();

        match state {
            GoodsState::AUCTION => {
                let Some(mut item) = self
                    .pop_item_from_goods_list(guid)
                    .map_err(DoneDelListError::MissingGoodsType)?
                else {
                    return Ok(());
                };
                if offer_price {
                    item.mark_as_sucessed();
                    let _ = self
                        .add_item_to_sucessed_list(item)
                        .map_err(DoneDelListError::DuplicateSucessed)?;
                } else {
                    item.mark_as_back();
                    let _ = self
                        .add_item_to_back_list(item)
                        .map_err(DoneDelListError::DuplicateBack)?;
                }
            }
            GoodsState::SUCESSED => {
                let Some(item) = self
                    .pop_item_from_goods_list(guid)
                    .map_err(DoneDelListError::MissingGoodsType)?
                else {
                    return Ok(());
                };
                let _ = self
                    .add_item_to_sucessed_list(item)
                    .map_err(DoneDelListError::DuplicateSucessed)?;
            }
            GoodsState::UNDO | GoodsState::PRE_BUY => {
                let Some(item) = self
                    .pop_item_from_goods_list(guid)
                    .map_err(DoneDelListError::MissingGoodsType)?
                else {
                    return Ok(());
                };
                let _ = self
                    .add_item_to_back_list(item)
                    .map_err(DoneDelListError::DuplicateBack)?;
            }
            unsupported => {
                self.clear_recond(guid)
                    .map_err(DoneDelListError::MissingGoodsType)?;
                // Контракт: Misc по
                //.. выполнял
                // `ClearRecond(guid); item->~CGoodsNode(); delete item;`
                // без `m_mapAuctionGoodsList.erase`, после чего удалял GUID из
                // batch. Достижимость dangling pointer неизвестна, поэтому
                // Box остаётся в primary map, а внешний выбор не выдумывается.
                return Err(DoneDelListError::DefaultStateDanglingPointer {
                    guid,
                    state: unsupported.raw(),
                });
            }
        }
        Ok(())
    }

    fn consume_first_del_guid(&mut self) {
        let became_empty = {
            let batch = self
                .del_from_goods_list
                .front_mut()
                .expect("обрабатываемый deletion batch должен существовать");
            batch.remove(0);
            batch.is_empty()
        };
        if became_empty {
            let _ = self.del_from_goods_list.pop_front();
        }
    }

    /// Копирует не более первых ста GUID signed owner-а в переданный список.
    fn query_item_from_owner_list(&self, player_id: i32, items: &mut Vec<CGuid>) -> bool {
        if let Some(owner_items) = self.owner_list.get(&player_id) {
            for guid in owner_items {
                items.push(*guid);
                if items.len() > 99 {
                    return true;
                }
            }
        }
        !items.is_empty()
    }

    /// Дописывает в сообщение self-auction список указанного игрока.
    pub(crate) fn add_byte_auction_self_to_client(
        &self,
        message: &mut CMessage,
        player_id: u32,
    ) -> Result<(), GoodsNodeSerializeError> {
        let mut items = Vec::new();
        let _ = self.query_item_from_owner_list(player_id as i32, &mut items);
        message.base_mut().add_long(items.len() as i32);

        for guid in items {
            let Some(item) = self.query_goods_node_info(guid) else {
                message.base_mut().add_long(0);
                continue;
            };

            message.base_mut().add_long(1);
            let serialized = item.serialize()?;
            message.base_mut().add(&serialized);
        }
        Ok(())
    }

    /// Вставляет либо целиком заменяет поисковое условие по его player ID.
    pub(crate) fn modify_player_seach_condition(&mut self, condition: PlayerOptNode) {
        let player_id = condition.player_id();
        let _ = self.player_search_list.insert(player_id, condition);
    }

    /// Изменяет сохранённую страницу игрока по старой операции `0/1/2`.
    pub(crate) fn compute_player_page(
        &mut self,
        player_id: u32,
        operation: u32,
    ) -> Result<(), AuctionPageBuildError> {
        let goods_count = u32::try_from(self.auction_goods_list.len()).map_err(|_| {
            AuctionPageBuildError::GoodsCountOutsideLegacyRange {
                length: self.auction_goods_list.len(),
            }
        })?;
        let page_count = goods_count.wrapping_add(6) / 7;
        let Some(condition) = self.player_search_list.get_mut(&player_id) else {
            return Ok(());
        };

        match operation {
            1 => {
                let next_page = condition.current_page().wrapping_add(1);
                condition.set_current_page(if page_count <= next_page {
                    page_count.wrapping_sub(1)
                } else {
                    next_page
                });
            }
            2 if condition.current_page() != 0 => {
                condition.set_current_page(condition.current_page() - 1);
            }
            2 => {}
            _ => condition.set_current_page(0),
        }
        Ok(())
    }

    /// Дописывает страницу временного индекса в исходном wire-порядке.
    pub(crate) fn add_byte_at_page_by_time(
        &self,
        player_id: u32,
        message: &mut CMessage,
    ) -> Result<(), AuctionPageBuildError> {
        let Some(condition) = self.player_search_list.get(&player_id) else {
            return Ok(());
        };
        let goods_count = u32::try_from(self.auction_goods_list.len()).map_err(|_| {
            AuctionPageBuildError::GoodsCountOutsideLegacyRange {
                length: self.auction_goods_list.len(),
            }
        })?;

        let mut start_ticket = self.time_ticket.keys().next().copied();
        let mut skip_in_start = 0usize;
        let mut visited = 0u32;
        let mut remaining_to_skip = condition.current_page().wrapping_mul(7) as i32;

        if remaining_to_skip > 0 {
            start_ticket = None;
            'tickets: for (ticket, guids) in &self.time_ticket {
                let mut visited_in_ticket = 0usize;
                for guid in guids {
                    visited_in_ticket += 1;
                    visited = visited.wrapping_add(1);
                    if self.is_match_condition(player_id, condition, *guid)? {
                        remaining_to_skip -= 1;
                        if remaining_to_skip == 0 {
                            start_ticket = Some(*ticket);
                            skip_in_start = visited_in_ticket;
                            break 'tickets;
                        }
                    }
                }
            }
        }

        let Some(start_ticket) = start_ticket else {
            message.base_mut().add_long(0);
            return Ok(());
        };

        message
            .base_mut()
            .add_ulong(goods_count.wrapping_sub(visited));
        message.base_mut().add_long(7);

        let mut slots = 7u32;
        let mut first_ticket = true;
        for (_, guids) in self.time_ticket.range(start_ticket..) {
            let skip = if first_ticket { skip_in_start } else { 0 };
            first_ticket = false;
            for guid in guids.iter().skip(skip) {
                if slots == 0 {
                    return Ok(());
                }
                if !self.is_match_condition(player_id, condition, *guid)? {
                    continue;
                }

                let item = self
                    .query_goods_node_info(*guid)
                    .expect("успешный IsMatchCondition гарантирует живой auction-узел");
                slots -= 1;
                message.base_mut().add_long(1);
                let serialized = item.serialize().map_err(AuctionPageBuildError::Serialize)?;
                message.base_mut().add(&serialized);
            }
        }
        Ok(())
    }

    fn is_match_condition(
        &self,
        player_id: u32,
        condition: &PlayerOptNode,
        guid: CGuid,
    ) -> Result<bool, AuctionPageBuildError> {
        let Some(item) = self.query_goods_node_info(guid) else {
            return Ok(false);
        };
        if player_id == item.owner_id() {
            return Ok(false);
        }

        let condition_name =
            legacy_name(condition.goods_name(), "stPlayerOptNode::m_strGoodsName")?;
        let name_matches = if condition_name.is_empty() {
            true
        } else {
            let item_name = legacy_name(item.goods_name(), "CGoodsNode::m_strGoodsName")?;
            item_name
                .windows(condition_name.len())
                .any(|window| window == condition_name)
        };
        let level = item
            .level_limit()
            .ok_or(AuctionPageBuildError::MissingLevelLimit)?;
        let goods_type = item
            .goods_type()
            .ok_or(AuctionPageBuildError::MissingGoodsType)?;

        let level_matches =
            (condition.low_level() as u32) <= level && level <= condition.up_level() as u32;
        let money_matches = u32::from(item.money_type()) == condition.money_type() as u32;
        let weapon_matches = condition.weapon_type() == -1
            || u32::from(goods_type) == condition.weapon_type() as u32;
        Ok(name_matches && level_matches && money_matches && weapon_matches)
    }

    /// Строит исходный ordered batch синхронизации одного GameServer map.
    pub(crate) fn unity_goods(
        &self,
        existing: &BTreeMap<CGuid, bool>,
        map_id: u8,
    ) -> UnityGoodsBuild {
        let mut unity_message = CMessage::new(AUCTION_UNITY_INDEX);
        unity_message.base_mut().add_byte(map_id);

        let mut messages = Vec::new();
        let mut detail_slots = AUCTION_UNITY_DETAIL_LIMIT;
        let mut need_send = false;

        if !self.auction_goods_list.is_empty() {
            let mut existing_iter = existing.keys().peekable();
            let mut primary_iter = self.auction_goods_list.iter().peekable();

            while existing_iter.peek().is_some() {
                let Some((primary_guid, primary_item)) = primary_iter.peek().copied() else {
                    break;
                };
                let existing_guid = *existing_iter
                    .peek()
                    .expect("цикл проверил существующий входной GUID");

                match existing_guid.cmp(primary_guid) {
                    std::cmp::Ordering::Equal => {
                        Self::append_unity_guid(&mut unity_message, *primary_guid);
                        existing_iter.next();
                        primary_iter.next();
                    }
                    std::cmp::Ordering::Less => {
                        need_send = true;
                        existing_iter.next();
                    }
                    std::cmp::Ordering::Greater => {
                        Self::append_unity_guid(&mut unity_message, *primary_guid);
                        if let Err(error) = Self::append_unity_detail(
                            &mut messages,
                            primary_item,
                            map_id,
                            &mut detail_slots,
                            &mut need_send,
                        ) {
                            return UnityGoodsBuild {
                                messages,
                                blocked: Some(error),
                            };
                        }
                        primary_iter.next();
                    }
                }
            }

            for (primary_guid, primary_item) in primary_iter {
                Self::append_unity_guid(&mut unity_message, *primary_guid);
                if let Err(error) = Self::append_unity_detail(
                    &mut messages,
                    primary_item,
                    map_id,
                    &mut detail_slots,
                    &mut need_send,
                ) {
                    return UnityGoodsBuild {
                        messages,
                        blocked: Some(error),
                    };
                }
            }

            if !need_send && existing_iter.peek().is_none() {
                return UnityGoodsBuild {
                    messages,
                    blocked: None,
                };
            }
        }

        messages.push(unity_message);
        UnityGoodsBuild {
            messages,
            blocked: None,
        }
    }

    fn append_unity_guid(message: &mut CMessage, guid: CGuid) {
        message.base_mut().add_byte(1);
        message.base_mut().add_guid(guid);
    }

    fn append_unity_detail(
        messages: &mut Vec<CMessage>,
        item: &CGoodsNode,
        map_id: u8,
        detail_slots: &mut usize,
        need_send: &mut bool,
    ) -> Result<(), GoodsNodeSerializeError> {
        if *detail_slots == 0 {
            return Ok(());
        }

        *detail_slots -= 1;
        *need_send = true;
        let serialized = item.serialize()?;
        let mut detail = CMessage::new(AUCTION_UNITY_ITEM);
        detail.base_mut().add_byte(map_id);
        detail.base_mut().add(&serialized);
        messages.push(detail);
        Ok(())
    }
}

fn legacy_name<'value>(
    value: &'value [u8; 256],
    field: &'static str,
) -> Result<&'value [u8], AuctionPageBuildError> {
    let terminator = value
        .iter()
        .position(|byte| *byte == 0)
        .ok_or(AuctionPageBuildError::LegacyStringWithoutTerminator { field })?;
    Ok(&value[..terminator])
}

impl<GoodsNode> Drop for CAuctionRoom<GoodsNode> {
    fn drop(&mut self) {
        self.clear();
    }
}
