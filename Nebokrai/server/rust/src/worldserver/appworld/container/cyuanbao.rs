//! Владелец yuanbao-container исторического `WorldServer`.
//!
//! Состояние конструктора и деструктора и folded
//! `Clear/Release/IsFull/GetGoods/GetGoodsAmount/Serialize/Unserialize`
//! `/`
//! и yuanbao-специфичные query/add-family
//!
//! входят в контракт owner-а. Источник контракта — точная пара WorldServer EXE/PDB.
//!
//! Layout сохраняет отдельный класс размера `0x28`: `CGoodsContainer +0x0`,
//! `CContainerListener +0x20`, единственный `m_pGoldCoins: CGoods* +0x24`.
//! Публичные container-операции `CYuanBao` указывают на те же, что
//! `CWallet`; это linker folding, а не наследование. Rust поэтому сохраняет
//! отдельный nominal owner, но делегирует его доказанный однослотовый state и
//! wire-кодек готовому compatibility-layer. `Option<Box<CGoods>>` и обычный
//! `Drop` заменяют nullable pointer, `GarbageCollect`, deleting-destructor и EH
//! cleanup. Оба listener callback-а имеют общий no-op.
//!
//! Wire остаётся marker-байтом `0/1` с возможным полным `CGoods`; decoder
//! сначала сбрасывает owner и прежний slot. Короткий source выражает
//! безразмерное legacy-чтение локальной типизированной ошибкой.
//! YuanBao-index разрешается process-global фабрикой через StringTable ID
//! `WS0109`; Rust принимает уже разрешённое значение и не переносит `CGame`.
//! Пустой slot принимает товар без currency-validation, а занятый
//! разрешает stacking только для YuanBao. Бесполезный legacy
//! `GetGoods(index, vector-by-value)` исправлен возвращаемым iterator-ом с тем
//! же YuanBao-фильтром.

use super::super::goods::cgoods::{CGoods, GoodsCodecError};
use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use super::super::listener::ccontainerlistener::CContainerListener;
use super::cwallet::CWallet;
use crate::dbaccess::worlddb::goodslistener::TraversedGoods;
use crate::public::guid::CGuid;

/// Отдельный действующий owner исходного `CYuanBao`, не копия его 32-битного ABI.
pub(crate) struct CYuanBao {
    wallet_state: CWallet,
}

impl CYuanBao {
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

 /// Проверяет наличие slot-а для resolved YuanBao-index.
    pub(crate) fn is_goods_existed(&self, base_properties_index: u32, yuan_bao_index: u32) -> bool {
        self.wallet_state
            .is_goods_existed(base_properties_index, yuan_bao_index)
    }

 /// Возвращает единственный slot только для resolved YuanBao-index.
    pub(crate) fn get_the_first_goods(
        &self,
        base_properties_index: u32,
        yuan_bao_index: u32,
    ) -> Option<&CGoods> {
        self.wallet_state
            .get_the_first_goods(base_properties_index, yuan_bao_index)
    }

 /// Возвращает usable iterator вместо legacy vector-by-value копии.
    pub(crate) fn get_goods_by_base_index(
        &self,
        base_properties_index: u32,
        yuan_bao_index: u32,
    ) -> impl Iterator<Item = &CGoods> {
        self.wallet_state
            .get_goods_by_base_index(base_properties_index, yuan_bao_index)
    }

 /// Делегирует positional `Add` с отдельным resolved YuanBao-index.
    pub(crate) fn add_at(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        yuan_bao_index: u32,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        self.wallet_state
            .add_at(position, goods, yuan_bao_index, registry)
    }

 /// Делегирует object-перегрузку позиции `0`.
    pub(crate) fn add(
        &mut self,
        goods: Box<CGoods>,
        yuan_bao_index: u32,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        self.wallet_state.add(goods, yuan_bao_index, registry)
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

 /// Кодирует folded marker/goods-wire исходного `CYuanBao`.
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
            "CYuanBao marker",
        )
    }
}
