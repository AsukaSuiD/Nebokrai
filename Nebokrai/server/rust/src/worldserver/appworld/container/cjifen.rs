//! Wallet-различия, скомпилированные из исторического `cjifen.cpp`.
//!
//! `CWallet::Release/IsFull/GetGoods/GetGoodsAmount/Serialize/Unserialize`
//!
//! и `Clear/QueryGoodsPosition/Find/Remove`
//! traversal
//! а также constructor/destructor-state `CJiFen`
//! собственные query/add-family
//!
//! и его folded container-контракт входят в контракт owner-а. Источник контракта — точная пара WorldServer EXE/PDB.
//!
//! Wallet хранит не коллекцию, а один nullable `CGoods*`. Wire начинается с
//! одного marker-байта: `0` завершает запись, любое ненулевое значение включает
//! полный goods-wire. Encoder передаёт `include_child` дальше без изменений;
//! decoder параметр не читает и вызывает готовый `CGoodsFactory::UnserializeGoods`,
//! который доказанно декодирует с `include_child=true`. Перед marker decoder
//! virtual-вызывает `Release`: owner type/ID становятся `0/0`, прежний товар
//! уничтожается, а короткий source уже не восстанавливает старое состояние.
//! Неизвестный base-properties index даёт null factory-result, который owner
//! сохраняет как пустой slot и всё равно возвращает `true`.
//!
//! `Clear` отличается от `Release`: он уничтожает товар, но сохраняет owner.
//! `IsFull` сравнивает amount с max-stack текущего товара. Встроенные
//! wallet callbacks имеют общий no-op, поэтому в действующих
//! операциях нет скрытой мутации. Безразмерное legacy-чтение marker-а выражено
//! локальной типизированной ошибкой короткого источника.
//! Object-перегрузка `QueryGoodsPosition` не сравнивает pointer identity:
//! ASM передаёт GUID товара в virtual GUID-overload.
//! Поэтому другой объект с тем же полным GUID также получает позицию `0`.
//!
//! Layout сохраняет `CJiFen` как отдельный класс размера `0x28`: bases
//! `CGoodsContainer +0x0`, `CContainerListener +0x20` и единственное поле
//! `m_pGoldCoins: CGoods* +0x24`. Его публичные `Clear/Release/IsFull/GetGoods/`
//! `GetGoodsAmount/Serialize/Unserialize` имеют те же, что `CWallet`;
//! это linker folding, а не наследование. Rust сохраняет отдельный nominal
//! owner и делегирует только доказанному общему состоянию/коду. Обычные
//! `Option<Box<CGoods>>` и `Drop` заменяют nullable pointer, `GarbageCollect`,
//! deleting-destructor и EH cleanup; listener callbacks — общий доказанный
//! no-op.
//!
//! Собственные операции отличаются только resolved JiFen-index.
//! `GetJiFenIndex` получает original name через StringTable ID `WS0110`, но
//! process-global `CGame/StringTable` не переносится: Rust принимает уже
//! разрешённый индекс. Как и wallet, пустой slot принимает товар без проверки
//! currency index, а занятый разрешает stacking только для JiFen. Legacy
//! `GetGoods(index, vector)` снова уничтожает vector-by-value; этот внутренний
//! бесполезный дефект исправлен возвращаемым iterator-ом при том же критерии.

use super::super::goods::cgoods::{CGoods, GoodsCodecError};
use super::super::goods::cgoodsfactory::{GoodsBasePropertiesRegistry, unserialize_goods};
use super::super::listener::ccontainerlistener::{CContainerListener, TraversedContainerObject};
use super::ccontainer::{ContainerGuidStorage, find_by_object_guid};
use super::cwallet::CWallet;
use crate::dbaccess::worlddb::goodslistener::TraversedGoods;
use crate::public::guid::CGuid;

/// Отдельный действующий owner исходного `CJiFen`, не копия его 32-битного ABI.
pub(crate) struct CJiFen {
    wallet_state: CWallet,
}

impl CJiFen {
 /// Создаёт owner `0/0` и пустой единственный slot исходного constructor-а.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            wallet_state: CWallet::with_constructor_defaults(),
        }
    }

 /// Очищает slot, не меняя inherited owner type/ID.
    pub(crate) fn clear(&mut self) {
        self.wallet_state.clear();
    }

 /// Сбрасывает inherited owner type/ID и уничтожает slot.
    pub(crate) fn release(&mut self) {
        self.wallet_state.release();
    }

 /// Проверяет достижение max-stack единственного товара.
    pub(crate) fn is_full(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, GoodsCodecError> {
        self.wallet_state.is_full(registry)
    }

 /// Возвращает товар только для единственной позиции `0`.
    pub(crate) fn get_goods(&self, position: u32) -> Option<&CGoods> {
        self.wallet_state.get_goods(position)
    }

 /// Возвращает mutable DB-view folded wallet-slot-а.
    pub(crate) fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        self.wallet_state.get_goods_mut(position)
    }

 /// Возвращает число занятых slot-ов: ноль либо один.
    pub(crate) const fn get_goods_amount(&self) -> u32 {
        self.wallet_state.get_goods_amount()
    }

 /// Проверяет наличие slot-а для resolved JiFen-index.
    pub(crate) fn is_goods_existed(&self, base_properties_index: u32, ji_fen_index: u32) -> bool {
        self.wallet_state
            .is_goods_existed(base_properties_index, ji_fen_index)
    }

 /// Возвращает единственный slot только для resolved JiFen-index.
    pub(crate) fn get_the_first_goods(
        &self,
        base_properties_index: u32,
        ji_fen_index: u32,
    ) -> Option<&CGoods> {
        self.wallet_state
            .get_the_first_goods(base_properties_index, ji_fen_index)
    }

 /// Возвращает usable iterator вместо legacy vector-by-value копии.
    pub(crate) fn get_goods_by_base_index(
        &self,
        base_properties_index: u32,
        ji_fen_index: u32,
    ) -> impl Iterator<Item = &CGoods> {
        self.wallet_state
            .get_goods_by_base_index(base_properties_index, ji_fen_index)
    }

 /// Делегирует positional `Add` с отдельным resolved JiFen-index.
    pub(crate) fn add_at(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        ji_fen_index: u32,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        self.wallet_state
            .add_at(position, goods, ji_fen_index, registry)
    }

 /// Делегирует object-перегрузку позиции `0`.
    pub(crate) fn add(
        &mut self,
        goods: Box<CGoods>,
        ji_fen_index: u32,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        self.wallet_state.add(goods, ji_fen_index, registry)
    }

 /// Вставляет DB-товар через positional collision-check без factory-validation.
    pub(crate) fn add_from_db(&mut self, position: u32, goods: Box<CGoods>) -> Option<Box<CGoods>> {
        self.wallet_state.add_from_db(position, goods)
    }

 /// Ищет товар по полному GUID в отдельном nominal owner-е.
    pub(crate) fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        self.wallet_state.find(ex_id)
    }

 /// Передаёт ownership совпавшего товара вызывающему.
    pub(crate) fn remove(&mut self, ex_id: &CGuid) -> Option<Box<CGoods>> {
        self.wallet_state.remove(ex_id)
    }

 /// Запрашивает позицию non-null объекта через его GUID.
    pub(crate) fn query_goods_position_by_object(&self, goods: Option<&CGoods>) -> Option<u32> {
        self.wallet_state.query_goods_position_by_object(goods)
    }

 /// Возвращает позицию `0` при полном GUID-совпадении.
    pub(crate) fn query_goods_position(&self, ex_id: &CGuid) -> Option<u32> {
        self.wallet_state.query_goods_position(ex_id)
    }

 /// Делегирует folded traversal общему safe listener trait-у.
    pub(crate) fn traversing_container<L: CContainerListener>(&self, listener: Option<&mut L>) {
        self.wallet_state.traversing_container(listener);
    }

 /// Замораживает единственный folded wallet-slot для DB traversal.
    pub(crate) fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, super::super::goods::cgoods::GoodsDbSnapshotBlock> {
        self.wallet_state.db_save_entries(registry)
    }

 /// Кодирует folded marker/goods-wire исходного `CJiFen`.
    pub(crate) fn serialize(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, GoodsCodecError> {
        self.wallet_state.serialize(destination, include_child)
    }

 /// Декодирует folded marker/goods-wire после обязательного раннего `Release`.
    pub(crate) fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, GoodsCodecError> {
        self.wallet_state.unserialize_with_marker_field(
            source,
            cursor,
            include_child,
            registry,
            "CJiFen marker",
        )
    }
}

impl CWallet {
 /// Очищает единственный slot, сохраняя inherited owner type/ID.
    pub(crate) fn clear(&mut self) {
        self.gold_coins = None;
    }

 /// Сбрасывает inherited owner type/ID и уничтожает единственный товар.
    pub(crate) fn release(&mut self) {
        self.container_base.release();
        self.gold_coins = None;
    }

 /// Проверяет достижение max-stack единственного товара.
    pub(crate) fn is_full(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, GoodsCodecError> {
        match &self.gold_coins {
            Some(goods) => Ok(goods.get_max_stack_number(registry)? <= goods.get_amount()),
            None => Ok(false),
        }
    }

 /// Возвращает товар только для единственной позиции `0`.
    pub(crate) fn get_goods(&self, position: u32) -> Option<&CGoods> {
        (position == 0)
            .then_some(self.gold_coins.as_deref())
            .flatten()
    }

 /// Возвращает mutable DB-view единственного wallet-slot-а.
    pub(crate) fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        (position == 0)
            .then_some(self.gold_coins.as_deref_mut())
            .flatten()
    }

 /// Возвращает число занятых wallet-slot-ов: ноль либо один.
    pub(crate) const fn get_goods_amount(&self) -> u32 {
        self.gold_coins.is_some() as u32
    }

 /// Ищет единственный товар по полному 16-байтовому legacy GUID.
    pub(crate) fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        <Self as ContainerGuidStorage>::find_by_guid(self, ex_id)
    }

 /// Передаёт ownership единственного совпавшего товара вызывающему.
    pub(crate) fn remove(&mut self, ex_id: &CGuid) -> Option<Box<CGoods>> {
        <Self as ContainerGuidStorage>::remove_by_guid(self, ex_id)
    }

 /// Запрашивает позицию non-null объекта через его GUID, как virtual call.
    pub(crate) fn query_goods_position_by_object(&self, goods: Option<&CGoods>) -> Option<u32> {
        find_by_object_guid(self, goods.map(CGoods::get_ex_id)).map(|_| 0)
    }

 /// Возвращает единственную позицию `0` при полном GUID-совпадении.
    pub(crate) fn query_goods_position(&self, ex_id: &CGuid) -> Option<u32> {
        self.find(ex_id).map(|_| 0)
    }

 /// Передаёт единственный non-null slot listener-у и игнорирует его `int` result.
    pub(crate) fn traversing_container<L: CContainerListener>(&self, listener: Option<&mut L>) {
        let (Some(listener), Some(goods)) = (listener, self.gold_coins.as_deref()) else {
            return;
        };
        let _ = listener.on_traversing_container(TraversedContainerObject::Goods(goods));
    }

 /// Кодирует marker и возможный полный goods-wire.
    pub(crate) fn serialize(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, GoodsCodecError> {
        let Some(goods) = &self.gold_coins else {
            destination.push(0);
            return Ok(true);
        };
        destination.push(1);
        let _ = goods.serialize(destination, include_child)?;
        Ok(true)
    }

 /// Декодирует marker после обязательного раннего `Release`.
    pub(crate) fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, GoodsCodecError> {
        self.unserialize_with_marker_field(
            source,
            cursor,
            _include_child,
            registry,
            "CWallet marker",
        )
    }

    pub(super) fn unserialize_with_marker_field(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
        marker_field: &'static str,
    ) -> Result<bool, GoodsCodecError> {
        self.release();
        let offset = *cursor;
        let Some(&marker) = source.get(offset) else {
 // Старый владелец не получал длину источника. При коротком буфере
 // он выходил за его границы; Rust не воспроизводит это UB.
            return Err(GoodsCodecError::UnexpectedEnd {
                field: marker_field,
                offset,
                needed: 1,
                available: source.len().saturating_sub(offset),
            });
        };
        *cursor = offset + 1;
        if marker != 0 {
            self.gold_coins = unserialize_goods(source, cursor, registry)?;
        }
        Ok(true)
    }
}
