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
//! получал длину и мог читать за buffer (`BLOCKED_MISSING_FACT`).
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
use super::cgoodscontainer::CGoodsContainerState;

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
        let Some(goods) = self.get_goods(position) else {
            return Ok(None);
        };
        if amount == 0 {
            return Ok(None);
        }

        let current_amount = goods.get_amount();
        if current_amount > amount {
            if goods.get_max_stack_number(registry)? <= 1 {
                return Ok(None);
            }
            let index = goods
                .get_base_properties_index()
                .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
            let ex_id = *goods.get_ex_id();
            let Some(mut removed) = create_goods(registry, index, random) else {
                return Ok(None);
            };
            removed.set_amount(amount);
            let Some(stored) = self.goods_mut(&ex_id) else {
                return Ok(None);
            };
            stored.set_amount(stored.get_amount().wrapping_sub(amount));
            return Ok(Some(removed));
        }

        if current_amount == amount {
            let ex_id = *goods.get_ex_id();
            return Ok(self.remove(&ex_id));
        }
        Ok(None)
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
        self.find(goods?.get_ex_id())
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
        // BLOCKED_MISSING_FACT: legacy helper не получал длину source.
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

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Remove
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:648
// RVA: 0x000D5E20
// ADDRESS: 004d5e20
// PROTOTYPE: CBaseObject * __thiscall Remove(ulong param_1, ulong param_2, void * param_3)
//
// IMPLEMENTED выше как `remove_at`; exact base ASM `0x004E0910..0x004E09F6`
// исправляет raw: первый аргумент — position, второй — amount. Embedded
// listener no-op, factory и обе amount-ветки сохранены.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Find
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:602
// RVA: 0x000D5F50
// ADDRESS: 004d5f50
// PROTOTYPE: CBaseObject * __thiscall Find(long param_1, CGUID * param_2)
//
// IMPLEMENTED выше как typed `find`; legacy type `700` не читался.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::IsFull
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:386
// RVA: 0x000DBCC0
// ADDRESS: 004dbcc0
// PROTOTYPE: int __thiscall IsFull(void)
//
// IMPLEMENTED выше; сравнение остаётся unsigned `limit <= valid-count`.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::SetGoodsAmountLimit
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:398
// RVA: 0x000DBCE0
// ADDRESS: 004dbce0
// PROTOTYPE: void __thiscall SetGoodsAmountLimit(ulong param_1)
//
// IMPLEMENTED выше без изменения unsigned bit-pattern.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::GetGoodsAmountLimit
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:406
// RVA: 0x000DBCF0
// ADDRESS: 004dbcf0
// PROTOTYPE: ulong __thiscall GetGoodsAmountLimit(void)
//
// IMPLEMENTED выше.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::SetOwner
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:517
// RVA: 0x000DBD00
// ADDRESS: 004dbd00
// PROTOTYPE: void __thiscall SetOwner(long param_1, long param_2)
//
// IMPLEMENTED выше; PDB owner fields заменяют потерянные raw-offset-ы.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Remove
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:584
// RVA: 0x000DBD20
// ADDRESS: 004dbd20
// PROTOTYPE: CBaseObject * __thiscall Remove(CBaseObject * param_1, void * param_2)
//
// Exact tail-chain проверяет null, извлекает GUID объекта по `+0x0c` и
// вызывает GUID-slot. Typed Rust call sites передают GUID прямо в `remove`.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Remove
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:590
// RVA: 0x000DBD30
// ADDRESS: 004dbd30
// PROTOTYPE: CBaseObject * __thiscall Remove(long param_1, CGUID * param_2, void * param_3)
//
// Exact tail-chain игнорирует type scalar и делегирует GUID-slot; typed owner
// устраняет параметр, не влияющий на семантику.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Find
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:596
// RVA: 0x000DBD40
// ADDRESS: 004dbd40
// PROTOTYPE: CBaseObject * __thiscall Find(CBaseObject * param_1)
//
// IMPLEMENTED выше как `find_object`; exact ASM `0x004E0A40..0x004E0A56`
// подтверждает null-check, GUID `object + 0x0C` и virtual GUID-slot `+0x04`.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Unserialize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:629
// RVA: 0x000DBD50
// ADDRESS: 004dbd50
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// IMPLEMENTED выше; `param_3` исходное тело не читает, factory сама передаёт
// подтверждённый `include_child=true` goods-decoder-у.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::IsLocked
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:464
// RVA: 0x000DBDC0
// ADDRESS: 004dbdc0
// PROTOTYPE: int __thiscall IsLocked(CGoods * param_1)
//
// IMPLEMENTED выше; safe receiver исключает исходный null-goods случай.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::TraversingContainer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:176
// RVA: 0x000DBE10
// ADDRESS: 004dbe10
// PROTOTYPE: void __thiscall TraversingContainer(CContainerListener * param_1)
//
// IMPLEMENTED выше; callback-result по exact ASM не управляет обходом.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::GetContentsWeight
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:200
// RVA: 0x000DBE40
// ADDRESS: 004dbe40
// PROTOTYPE: ulong __thiscall GetContentsWeight(void)
//
// IMPLEMENTED выше; map-values и unsigned wrapping sum сохранены, null raw
// pointer исключён owning `Box<CGoods>`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::GetGoods
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:218
// RVA: 0x000DBE70
// ADDRESS: 004dbe70
// PROTOTYPE: CGoods * __thiscall GetGoods(ulong param_1)
//
// IMPLEMENTED выше; ordinal включает locked entries, но выбранный locked
// товар возвращается как `None`.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::GetTheFirstGoods
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:267
// RVA: 0x000DBEC0
// ADDRESS: 004dbec0
// PROTOTYPE: CGoods * __thiscall GetTheFirstGoods(ulong param_1)
//
// IMPLEMENTED выше; matching locked entries пропускаются.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::QueryGoodsPosition
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:289
// RVA: 0x000DBF20
// ADDRESS: 004dbf20
// PROTOTYPE: int __thiscall QueryGoodsPosition(CGoods * param_1, ulong * param_2)
//
// IMPLEMENTED выше; `std::ptr::eq` сохраняет object identity без `unsafe`.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::IsGoodsExisted
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:316
// RVA: 0x000DBF60
// ADDRESS: 004dbf60
// PROTOTYPE: int __thiscall IsGoodsExisted(ulong param_1)
//
// IMPLEMENTED выше; exact тело намеренно не вызывает `IsLocked`.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:501
// RVA: 0x000DBFB0
// ADDRESS: 004dbfb0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::QueryGoodsPosition
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:526
// RVA: 0x000DBFE0
// ADDRESS: 004dbfe0
// PROTOTYPE: int __thiscall QueryGoodsPosition(CGUID * param_1, ulong * param_2)
//
// IMPLEMENTED выше; GUID сравнивается полностью, lock не проверяется.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::GetGoodsAmount
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:563
// RVA: 0x000DC030
// ADDRESS: 004dc030
// PROTOTYPE: ulong __thiscall GetGoodsAmount(void)
//
// IMPLEMENTED выше; null/unknown base-properties не входят в count.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Serialize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:608
// RVA: 0x000DC070
// ADDRESS: 004dc070
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// IMPLEMENTED выше; count и record filter используют один factory lookup.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Find
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:415
// RVA: 0x000DC180
// ADDRESS: 004dc180
// PROTOTYPE: CBaseObject * __thiscall Find(CGUID * param_1)
//
// IMPLEMENTED выше; hash miss и locked result дают один `None`.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Remove
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:433
// RVA: 0x000DC1C0
// ADDRESS: 004dc1c0
// PROTOTYPE: CBaseObject * __thiscall Remove(CGUID * param_1, void * param_2)
//
// IMPLEMENTED выше; exact `0x004DC1E7..0x004DC1F4` отклоняет locked товар,
// no-op listener loop предшествует map erase, а locked-vector не меняется.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Unlock
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:355
// RVA: 0x000DC260
// ADDRESS: 004dc260
// PROTOTYPE: int __thiscall Unlock(CGoods * param_1)
//
// IMPLEMENTED выше; `Vec::remove` удаляет ровно первое совпадение и сдвигает
// оставшийся хвост как exact `0x004DC2A1..0x004DC2DE`.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::GetGoods
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:247
// RVA: 0x000DC5B0
// ADDRESS: 004dc5b0
// PROTOTYPE: void __thiscall GetGoods(ulong param_1, vector<CGoods*,std::allocator<CGoods*>_> param_2)
//
// IMPLEMENTED выше; by-value output наполняется unlocked pointers и затем
// уничтожается, как подтверждают mangled symbol и exact `RET 0x14`.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Lock
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:336
// RVA: 0x000DC640
// ADDRESS: 004dc640
// PROTOTYPE: int __thiscall Lock(CGoods * param_1)
//
// IMPLEMENTED выше; принадлежность `goods` текущему container-у не проверяется.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::CAmountLimitGoodsContainer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:19
// RVA: 0x000DC700
// ADDRESS: 004dc700
// PROTOTYPE: undefined __thiscall CAmountLimitGoodsContainer(void)
//
// IMPLEMENTED выше; exact secondary vtable и оба no-op listener slots
// подтверждены по `0x004DC721..0x004DC76F` и `0x004DBD10`.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Add
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:42
// RVA: 0x000DC790
// ADDRESS: 004dc790
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, void * param_2)
//
// IMPLEMENTED выше; typed `Box<CGoods>` заменяет null/dynamic-cast, а World
// added-listener имеет доказанное no-op тело.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::AddFromDB
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:72
// RVA: 0x000DC820
// ADDRESS: 004dc820
// PROTOTYPE: int __thiscall AddFromDB(CGoods * param_1, ulong param_2)
//
// IMPLEMENTED выше; exact ASM подтверждает ранний full, locked-aware ordinal
// `GetGoods`, direct GUID hash insert и literal `0/1` return. Исторический
// `debug-DB` collision-log не влияет на container state и не материализован.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Clear
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:114
// RVA: 0x000DC9A0
// ADDRESS: 004dc9a0
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// IMPLEMENTED выше; removed-listener no-op, map/locked очищаются, owner/limit
// сохраняются. MSVC hash/vector cleanup заменён Drop.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Release
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:139
// RVA: 0x000DCAB0
// ADDRESS: 004dcab0
// PROTOTYPE: void __thiscall Release(void)
//
// IMPLEMENTED выше; `Box` drop заменяет `GarbageCollect`, base owner становится
// `0/0`, limit `1`, а no-op embedded listener не требует Rust storage.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Clone
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:158
// RVA: 0x000DCBA0
// ADDRESS: 004dcba0
// PROTOTYPE: int __thiscall Clone(CGoodsContainer * param_1)
//
// IMPLEMENTED выше; Rust target type заменяет RTTI, limit назначается первым,
// map получает deep-owned товары. Owner/locked target-а исходно не копируются.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::Add
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:493
// RVA: 0x000DCBF0
// ADDRESS: 004dcbf0
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, void * param_3)
//
// IMPLEMENTED тем же typed `add`; позиционный `param_1` не читается.

// ============================================================================
// FUNCTION: CAmountLimitGoodsContainer::~CAmountLimitGoodsContainer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp:35
// RVA: 0x000DCC10
// ADDRESS: 004dcc10
// PROTOTYPE: void __thiscall ~CAmountLimitGoodsContainer(void)
//
// Rust field-drop повторяет `Release`/map/vector cleanup без vtable/EH/STL
// plumbing; отдельный `Drop` не нужен.

// ============================================================================
// FUNCTION: Unwind@00535580
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp
// RVA: 0x00135580
// ADDRESS: 00535580
// PROTOTYPE: undefined Unwind@00535580()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0053558b
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\camountlimitgoodscontainer.cpp
// RVA: 0x0013558B
// ADDRESS: 0053558b
// PROTOTYPE: undefined Unwind@0053558b()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
