//! Владелец amount-limited goods-container исторического `WorldServer`.
//!
//! Статус constructor RVA `0x000DC700`, `Find(long, GUID)` RVA `0x000D5F50`,
//! `Find(GUID/object)/IsLocked/TraversingContainer` RVA
//! `0x000DC180/0x000DBD40/0x000DBDC0/0x000DBE10`, query-family RVA
//! `0x000DBE70/0x000DBEC0/0x000DBF20/0x000DBF60/0x000DBFE0/0x000DC5B0`,
//! `GetContentsWeight` RVA `0x000DBE40`,
//! `Lock/Unlock` RVA `0x000DC640/0x000DC260`, `IsFull/Set/GetLimit` RVA
//! `0x000DBCC0/0x000DBCE0/0x000DBCF0`, `SetOwner` RVA `0x000DBD00`,
//! positional `Remove` RVA `0x000D5E20`, `Remove` wrapper-ы и GUID-owner RVA
//! `0x000DBD20/0x000DBD30/0x000DC1C0`,
//! `Unserialize/Serialize` RVA `0x000DBD50/0x000DC070`, `GetGoodsAmount` RVA
//! `0x000DC030`, основного `Add/AddFromDB` RVA `0x000DC790/0x000DC820`,
//! `Clear/Release` RVA
//! `0x000DC9A0/0x000DCAB0`, `Clone` RVA `0x000DCBA0` и destructor RVA
//! `0x000DCC10` — `IMPLEMENTED`;
//! остальные операции ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.h`
//! и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:19,42,114,139,176,218,247,267,289,316,336,355,415,464,526,563,602,608,629`.
//!
//! Exact PDB задаёт старый размер `0x60`: `CGoodsContainer`/secondary
//! `CContainerListener` prefix, `stdext::hash_map<CGUID, CGoods*>` по `+0x24`,
//! unsigned limit по `+0x4C` и vector locked GUID по `+0x50`. Constructor
//! создаёт пустые map/vector, limit `1`, owner type/ID `0` и регистрирует
//! embedded listener. Exact constructor `0x004DC721..0x004DC76F` подтвердил
//! secondary subobject `+0x20` и vtable `0x00549410`; оба added/removed slots
//! ведут в `0x004DBD10`, чьё полное тело — `mov eax,1; ret 0xC`. Поэтому этот
//! listener не имеет побочных эффектов. Во всём World PDB/raw-корпусе других
//! `OnObjectAdded/OnObjectRemoved` override-ов нет; после ответа reverse
//! прекращён.
//!
//! Rust `BTreeMap<CGuid, Box<CGoods>>` заменяет только STL hash storage и raw
//! ownership. GUID остаётся ключом. Старый duplicate overwrite, `Clear` и
//! проигнорированный false `Add` теряли прежний `CGoods*` без
//! `GarbageCollect`. Это внутренний lifetime-дефект без наблюдаемого контракта:
//! доказанный `CGoods` destructor не имеет внешних callback-эффектов, поэтому
//! вытесненные и отклонённые товары теперь уничтожаются обычным Rust `Drop`.
//!
//! Старый hash-list traversal не является игровым порядком: каждая wire-запись
//! содержит свой GUID внутри `CGoods`, decoder вставляет её обратно по GUID, а
//! оба достигнутых callbacks no-op. `BTreeMap` даёт воспроизводимый raw-GUID
//! order без ручного восстановления внутренностей MSVC `stdext::hash_map`.
//!
//! `Serialize` пишет unsigned число только тех товаров, чей base-properties
//! lookup non-null, затем те же полные `CGoods` records. `Unserialize` сначала
//! выполняет точный `Clear`, читает unsigned count, вызывает готовый
//! `CGoodsFactory::UnserializeGoods` и игнорирует bool основного `Add`, как
//! оригинал. Limit и owner при `Clear` сохраняются; `Release` сбрасывает owner,
//! limit в `1`, все товары и locked GUID. Non-null/dynamic-cast границы старого
//! `Add(CBaseObject*)` выражены `Box<CGoods>`, поэтому Rust API не принимает
//! объект другого типа или null.
//!
//! Товар с ещё не назначенным base-properties index не получает значения из
//! старого неинициализированного DWORD: безопасная граница возвращает typed
//! `MissingBasePropertiesIndex`. Короткий source также останавливается typed
//! ошибкой, сохраняя ранний `Clear` и уже добавленные records; legacy helper не
//! получал длину и мог читать за buffer; короткий источник возвращает
//! типизированную ошибку после уже выполненных изменений.
//!
//! Exact `Find(GUID)` сначала ищет hash-node, затем вызывает virtual
//! `IsLocked`; совпадение любого из четырёх DWORD GUID в locked-vector
//! недостаточно — `0x004DBDE0..0x004DBDED` сравнивает все 16 байт. Typed
//! `Find(long, GUID)` через thunks `0x004D5F50/0x004E0660` приходит в
//! `0x004E0A00`, читает только GUID из второго аргумента и делегирует этому
//! методу, поэтому safe Rust не носит лишний type `700` рядом с уже typed
//! `CGoods`. Object-overload через `0x004E0A40` проверяет null, берёт тот же
//! embedded GUID по `+0x0C` и делегирует GUID-slot. Traversal при
//! null listener ничего не делает, иначе посещает все map-values и игнорирует
//! callback-result. `BTreeMap` сохраняет уже принятую storage-замену; для
//! достигнутого `CheckGoodsInPacket` порядок ненаблюдаем, поскольку итоговая
//! 32-битная wrapping-сумма коммутативна.
//! `GetContentsWeight` также обходит все map-values без lock-фильтра и складывает
//! exact wrapping-вес товаров; storage-order на коммутативный результат не
//! влияет.
//!
//! Query-family намеренно различает locked-состояние. `GetGoods(position)`,
//! `GetTheFirstGoods` и `GetGoods(base index, vector)` скрывают locked-товар;
//! `IsGoodsExisted` и обе формы `QueryGoodsPosition` locked-состояние не
//! проверяют. Позиция остаётся ordinal позиции map traversal, включая locked-
//! элементы. Object-overload сравнивает именно pointer identity, GUID-overload
//! — все 16 байт идентификатора. Safe Rust выражает false без изменения out-
//! параметра как `None` и использует `std::ptr::eq` без `unsafe`.
//!
//! PDB/mangled symbol для `GetGoods(base index, vector)` подтверждает передачу
//! vector по значению, а exact `RET 0x14` и destructor временной копии — что
//! собранный список не возвращается вызывающему. Rust сохраняет этот странный
//! контракт принимаемым по значению `Vec<&CGoods>` и unit-result, не исправляя
//! историческую сигнатуру в reference-output API.
//!
//! Exact `Lock` не проверяет принадлежность переданного товара контейнеру: он
//! отвергает только null и уже locked GUID, затем копирует GUID. `Unlock`
//! находит и удаляет первое совпадение, сдвигая хвост на один элемент. Новый
//! C++ reference принимает GUID, требует `Find` и использует удаление всех
//! совпадений; Rust сохраняет EXE-контракт через `Option<&CGoods>` и `Vec::remove`.
//!
//! Exact GUID-`Remove` ищет map-node, отклоняет locked товар, вызывает только
//! доказанные no-op listeners и лишь затем вынимает pointer из map; locked-
//! vector при успехе не чистится. `BTreeMap::remove` заменяет legacy hash erase,
//! а `Box<CGoods>` явно переносит возвращаемое ownership вызывающему.
//! Positional `Remove(position, amount)` сначала использует locked-aware
//! `GetGoods`. При частичном stack-remove он требует max-stack больше `1`,
//! создаёт новый товар через exact factory, назначает requested amount и только
//! затем вычитает его из исходного stack; равный amount делегирует GUID-remove.
//! Точный ASM исправляет перепутанные raw-аргументы и сохраняет unsigned SUB.
//!
//! `AddFromDB` отдельно от обычного Add сначала проверяет full, затем ordinal
//! `GetGoods(position)`: unlocked существующий товар даёт false и debug-log,
//! locked товар скрывается и не блокирует последующий hash overwrite. Non-null
//! incoming после этого вставляется без factory-validation и без listener-
//! callback. Debug-file заменён отсутствием технического log sink; success/
//! rejection и state-переходы сохранены, вытесненный pointer безопасно
//! уничтожается как внутренний lifetime-дефект.
//!
//! Exact `Clone` присваивает target только limit и map сырых `CGoods*`; owner и
//! locked-vector target сохраняются. World-корпус не содержит caller-а этого
//! virtual slot-а, а shallow alias создаёт только double-free/use-after-free
//! lifetime-дефект. Rust исправляет внутреннее владение deep clone-ом каждого
//! товара через его подтверждённый wire-roundtrip, сохраняя набор полей и
//! порядок target-эффектов. `AI` всё ещё требует child-graph `CBaseObject` и
//! остаётся RAW.

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

use crate::dbaccess::worlddb::goodslistener::TraversedGoods;
use crate::public::guid::CGuid;
use crate::worldserver::appworld::listener::ccontainerlistener::{
    CContainerListener, TraversedContainerObject,
};

use super::super::goods::cgoods::{CGoods, GoodsCodecError};
use super::super::goods::cgoodsfactory::{
    GoodsBasePropertiesRegistry, create_goods, query_goods_base_properties, unserialize_goods,
};
use super::cgoodscontainer::{
    CGoodsContainerState, GoodsContainerPositionStorage, remove_from_position,
};
use super::ccontainer::{ContainerGuidStorage, find_by_object_guid};

/// Ошибка безопасной границы amount-container codec-а.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AmountContainerCodecError {
    Goods(GoodsCodecError),
    ValidGoodsCountOutsideLegacyRange {
        count: usize,
    },
    UnexpectedEnd {
        field: &'static str,
        offset: usize,
        needed: usize,
        available: usize,
    },
}

impl fmt::Display for AmountContainerCodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Goods(error) => error.fmt(formatter),
            Self::ValidGoodsCountOutsideLegacyRange { count } => write!(
                formatter,
                "amount-container содержит {count} валидных товаров вне 32-битного legacy-диапазона"
            ),
            Self::UnexpectedEnd {
                field,
                offset,
                needed,
                available,
            } => write!(
                formatter,
                "поле {field} с offset {offset} требует {needed} байт, доступно {available}"
            ),
        }
    }
}

impl Error for AmountContainerCodecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Goods(error) => Some(error),
            _ => None,
        }
    }
}

impl From<GoodsCodecError> for AmountContainerCodecError {
    fn from(error: GoodsCodecError) -> Self {
        Self::Goods(error)
    }
}

/// Достигнутая owning-часть `CAmountLimitGoodsContainer`.
pub(crate) struct CAmountLimitGoodsContainer {
    container_base: CGoodsContainerState,
    goods: BTreeMap<CGuid, Box<CGoods>>,
    goods_amount_limit: u32,
    locked_goods: Vec<CGuid>,
}

impl ContainerGuidStorage for CAmountLimitGoodsContainer {
    type Object = CGoods;
    type Removed = Box<CGoods>;

    fn find_by_guid(&self, ex_id: &CGuid) -> Option<&Self::Object> {
        self.find(ex_id)
    }

    fn remove_by_guid(&mut self, ex_id: &CGuid) -> Option<Self::Removed> {
        self.remove(ex_id)
    }
}

impl GoodsContainerPositionStorage for CAmountLimitGoodsContainer {
    fn goods_at(&mut self, position: u32) -> Option<&mut CGoods> {
        self.get_goods_mut(position)
    }

    fn remove_by_guid(&mut self, ex_id: &CGuid) -> Option<Box<CGoods>> {
        self.remove(ex_id)
    }
}

impl CAmountLimitGoodsContainer {
    /// Создаёт точные defaults constructor-а.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            container_base: CGoodsContainerState::with_constructor_defaults(),
            goods: BTreeMap::new(),
            goods_amount_limit: 1,
            locked_goods: Vec::new(),
        }
    }

    /// Присваивает unsigned предел количества валидных товаров.
    pub(crate) const fn set_goods_amount_limit(&mut self, limit: u32) {
        self.goods_amount_limit = limit;
    }

    /// Возвращает unsigned предел количества валидных товаров.
    pub(crate) const fn get_goods_amount_limit(&self) -> u32 {
        self.goods_amount_limit
    }

    /// Сохраняет два signed owner scalar без дополнительных эффектов.
    pub(crate) const fn set_owner(&mut self, owner_type: i32, owner_id: i32) {
        self.container_base.set_owner(owner_type, owner_id);
    }

    /// Считает только товары с non-null base-properties lookup.
    pub(crate) fn get_goods_amount(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<u32, AmountContainerCodecError> {
        let mut count = 0usize;
        for goods in self.goods.values() {
            let index = goods
                .get_base_properties_index()
                .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
            if query_goods_base_properties(registry, index).is_some() {
                count += 1;
            }
        }
        u32::try_from(count)
            .map_err(|_| AmountContainerCodecError::ValidGoodsCountOutsideLegacyRange { count })
    }

    /// Сравнивает текущий valid-count с unsigned limit как исходный `IsFull`.
    pub(crate) fn is_full(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, AmountContainerCodecError> {
        Ok(self.goods_amount_limit <= self.get_goods_amount(registry)?)
    }

    /// Добавляет typed товар по exact GUID; `Some` сохраняет ownership при false.
    pub(crate) fn add(
        &mut self,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, AmountContainerCodecError> {
        if self.is_full(registry)? {
            return Ok(Some(goods));
        }
        self.insert_unchecked(goods);
        Ok(None)
    }

    /// Воспроизводит DB-вставку: full и unlocked ordinal collision до insert.
    pub(crate) fn add_from_db(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, AmountContainerCodecError> {
        if self.is_full(registry)? || self.get_goods(position).is_some() {
            return Ok(Some(goods));
        }
        self.insert_unchecked(goods);
        Ok(None)
    }

    /// Вставляет raw DB/legacy pointer по GUID и безопасно уничтожает вытесненный.
    pub(super) fn insert_unchecked(&mut self, goods: Box<CGoods>) {
        let ex_id = *goods.get_ex_id();
        let _ = self.goods.insert(ex_id, goods);
    }

    /// Вынимает unlocked товар по GUID и переносит ownership вызывающему.
    pub(super) fn remove(&mut self, ex_id: &CGuid) -> Option<Box<CGoods>> {
        let goods = self.goods.get(ex_id)?;
        if self.is_locked(goods) {
            return None;
        }
        self.goods.remove(ex_id)
    }

    /// Вынимает exact amount из ordinal position, включая split-stack ветку.
    pub(crate) fn remove_at<Random>(
        &mut self,
        position: u32,
        amount: u32,
        registry: &GoodsBasePropertiesRegistry,
        random: &mut Random,
    ) -> Result<Option<Box<CGoods>>, AmountContainerCodecError>
    where
        Random: FnMut(i32) -> i32,
    {
        let mut create = |index| create_goods(registry, index, random);
        let mut no_op_listener = |_: &CGoods| {};
        remove_from_position(
            self,
            position,
            amount,
            registry,
            &mut create,
            &mut no_op_listener,
        )
        .map_err(Into::into)
    }

    /// Очищает goods/locked storage, сохраняя owner и limit.
    pub(crate) fn clear(&mut self) {
        self.goods.clear();
        self.locked_goods.clear();
    }

    /// Выполняет точный достигнутый reset `Release`.
    pub(crate) fn release(&mut self) {
        self.goods_amount_limit = 1;
        self.goods.clear();
        self.locked_goods.clear();
        self.container_base.release();
    }

    /// Обходит все goods в storage-order ровно один раз для virtual `CGoods::AI`.
    /// Сам товарный AI остаётся owner-ом `CGoods`, поэтому callback внедряется
    /// явно вместо прежнего virtual dispatch через raw pointer.
    pub(crate) fn ai(&mut self, mut on_goods_ai: impl FnMut(&mut CGoods)) {
        for goods in self.goods.values_mut() {
            on_goods_ai(goods);
        }
    }

    /// Копирует World-набор `limit + goods`, сохраняя owner/locked target-а.
    pub(crate) fn clone_into(
        &self,
        target: &mut CAmountLimitGoodsContainer,
    ) -> Result<bool, GoodsCodecError> {
        target.goods_amount_limit = self.goods_amount_limit;

        let mut cloned_goods = BTreeMap::new();
        for (&ex_id, goods) in &self.goods {
            let mut cloned = Box::new(CGoods::with_constructor_base_and_type());
            let _ = goods.as_ref().clone_into(cloned.as_mut())?;
            cloned_goods.insert(ex_id, cloned);
        }
        target.goods = cloned_goods;
        Ok(true)
    }

    /// Ищет exact GUID и скрывает товар, если тот присутствует в locked-vector.
    pub(crate) fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        let goods = self.goods.get(ex_id).map(Box::as_ref)?;
        (!self.is_locked(goods)).then_some(goods)
    }

    /// Делегирует object-overload точному GUID-поиску после null-check.
    pub(crate) fn find_object(&self, goods: Option<&CGoods>) -> Option<&CGoods> {
        find_by_object_guid(self, goods.map(CGoods::get_ex_id))
    }

    /// Возвращает ordinal map-элемент, если position ниже limit и он не locked.
    pub(crate) fn get_goods(&self, position: u32) -> Option<&CGoods> {
        if position >= self.goods_amount_limit {
            return None;
        }

        let goods = self
            .goods
            .values()
            .nth(position as usize)
            .map(Box::as_ref)?;
        (!self.is_locked(goods)).then_some(goods)
    }

    /// Возвращает mutable DB-view того же unlocked ordinal-элемента.
    pub(crate) fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        let ex_id = *self.get_goods(position)?.get_ex_id();
        self.goods_mut(&ex_id)
    }

    /// Возвращает первый unlocked товар с exact base-properties index.
    pub(crate) fn get_the_first_goods(&self, base_properties_index: u32) -> Option<&CGoods> {
        self.goods.values().map(Box::as_ref).find(|goods| {
            goods.get_base_properties_index() == Some(base_properties_index)
                && !self.is_locked(goods)
        })
    }

    /// Проверяет наличие base-properties index, намеренно не учитывая lock.
    pub(crate) fn is_goods_existed(&self, base_properties_index: u32) -> bool {
        self.goods
            .values()
            .any(|goods| goods.get_base_properties_index() == Some(base_properties_index))
    }

    /// Возвращает ordinal по exact object identity без lock-фильтра.
    pub(crate) fn query_goods_position(&self, goods: Option<&CGoods>) -> Option<u32> {
        let goods = goods?;
        self.goods
            .values()
            .position(|stored| std::ptr::eq(stored.as_ref(), goods))
            .and_then(|position| u32::try_from(position).ok())
    }

    /// Возвращает ordinal по полному GUID без lock-фильтра.
    pub(crate) fn query_goods_position_by_guid(&self, ex_id: &CGuid) -> Option<u32> {
        self.goods
            .values()
            .position(|goods| goods.get_ex_id() == ex_id)
            .and_then(|position| u32::try_from(position).ok())
    }

    /// Сохраняет исходную by-value vector-сигнатуру: результат будет отброшен.
    pub(crate) fn get_goods_by_base_index<'container>(
        &'container self,
        base_properties_index: u32,
        mut output: Vec<&'container CGoods>,
    ) {
        output.extend(self.goods.values().map(Box::as_ref).filter(|goods| {
            goods.get_base_properties_index() == Some(base_properties_index)
                && !self.is_locked(goods)
        }));
    }

    /// Копирует GUID любого non-null товара, если такой GUID ещё не locked.
    pub(crate) fn lock(&mut self, goods: Option<&CGoods>) -> i32 {
        let Some(goods) = goods else {
            return 0;
        };
        if self.is_locked(goods) {
            return 0;
        }

        self.locked_goods.push(*goods.get_ex_id());
        1
    }

    /// Удаляет первое exact GUID-совпадение либо возвращает исходный `0`.
    pub(crate) fn unlock(&mut self, goods: Option<&CGoods>) -> i32 {
        let Some(goods) = goods else {
            return 0;
        };
        let Some(position) = self
            .locked_goods
            .iter()
            .position(|locked| locked == goods.get_ex_id())
        else {
            return 0;
        };

        self.locked_goods.remove(position);
        1
    }

    /// Проверяет полное 16-байтовое совпадение GUID в linear locked-vector.
    fn is_locked(&self, goods: &CGoods) -> bool {
        self.locked_goods
            .iter()
            .any(|locked| locked == goods.get_ex_id())
    }

    /// Передаёт listener-у все map-values и игнорирует его `int` результат.
    pub(crate) fn traversing_container<L: CContainerListener>(&self, listener: Option<&mut L>) {
        let Some(listener) = listener else {
            return;
        };

        for goods in self.goods.values().map(Box::as_ref) {
            let _ = listener.on_traversing_container(TraversedContainerObject::Goods(goods));
        }
    }

    /// Складывает exact unsigned вес всех товаров без lock-фильтра.
    pub(crate) fn get_contents_weight(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<u32, AmountContainerCodecError> {
        self.goods.values().try_fold(0u32, |weight, goods| {
            Ok(weight.wrapping_add(goods.get_weight(registry)?))
        })
    }

    /// Даёт derived-container-у read-only обход единственного goods-owner-а.
    pub(super) fn goods(&self) -> impl Iterator<Item = &CGoods> {
        self.goods.values().map(Box::as_ref)
    }

    /// Замораживает concrete traversal с ordinal `QueryGoodsPosition`.
    pub(crate) fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, super::super::goods::cgoods::GoodsDbSnapshotBlock> {
        self.goods()
            .map(|goods| {
                Ok(TraversedGoods {
                    goods: goods.db_save_snapshot(registry)?,
                    position: self.query_goods_position(Some(goods)).unwrap_or(0) as u8,
                })
            })
            .collect()
    }

    /// Заимствует товар по exact GUID для derived positional операции.
    pub(super) fn goods_mut(&mut self, ex_id: &CGuid) -> Option<&mut CGoods> {
        self.goods.get_mut(ex_id).map(Box::as_mut)
    }

    /// Кодирует valid-count и полные товары в container wire.
    pub(crate) fn serialize(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, AmountContainerCodecError> {
        let count = self.get_goods_amount(registry)?;
        destination.extend_from_slice(&count.to_le_bytes());
        for goods in self.goods.values() {
            let index = goods
                .get_base_properties_index()
                .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
            if query_goods_base_properties(registry, index).is_some() {
                let _ = goods.serialize(destination, include_child)?;
            }
        }
        Ok(true)
    }

    /// Декодирует container после точного раннего `Clear`.
    pub(crate) fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, AmountContainerCodecError> {
        self.clear();
        let count = read_amount_u32(source, cursor, "goods count")?;
        for _ in 0..count {
            if let Some(goods) = unserialize_goods(source, cursor, registry)? {
                let _ = self.add(goods, registry)?;
            }
        }
        Ok(true)
    }
}

fn read_amount_u32(
    source: &[u8],
    cursor: &mut usize,
    field: &'static str,
) -> Result<u32, AmountContainerCodecError> {
    let offset = *cursor;
    let available = source.len().saturating_sub(offset);
    let Some(end) = offset.checked_add(4) else {
        return Err(AmountContainerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: 4,
            available,
        });
    };
    let Some(bytes) = source.get(offset..end) else {
        // Legacy helper не получал длину source; реакция на короткий буфер
        // определялась выходом за его границы и не имитируется.
        return Err(AmountContainerCodecError::UnexpectedEnd {
            field,
            offset,
            needed: 4,
            available,
        });
    };
    *cursor = end;
    Ok(u32::from_le_bytes(
        bytes
            .try_into()
            .expect("slice содержит ровно четыре байта legacy count"),
    ))
}
