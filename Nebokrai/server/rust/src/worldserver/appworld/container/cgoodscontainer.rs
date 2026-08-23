//! Владелец общего goods-container исторического `WorldServer`.
//!
//! object `Add`,
//! `AddFromDB`, positional `Add` и positional `Remove`
//! входят в контракт owner-а.
//!
//! Ветка работает только с уже найденным товаром позиции:
//! сначала сравнивает unsigned base-properties index, затем signed particular
//! attribute `GAP_PARTICULAR_ATTRIBUTE/1`, после чего требует max-stack больше
//! `1`. Вычитание свободного места и сложение amount остаются 32-битными
//! wrapping-операциями исходного `unsigned long`. Успех уничтожает incoming
//! товар через Rust ownership, эквивалентно `CGoodsFactory::GarbageCollect`, и
//! возвращает `None`; false возвращает тот же `Box` вызывающему как `Some`.
//!
//! Receiver второго base-index/addon вызова загружается из `pObject`, а не из
//! numeric position. Listener loop после успеха сохранён как доказанный no-op:
//! действующий embedded listener amount-owner-а имеет оба слота
//! `mov eax,1; ret 0xC`, а других override-ов в World owner нет.
//!
//! base-state подтверждает owner type/ID по `+0x14/+0x18`.
//! `Clear` — folded пустой `ret 4`, а `Release` сначала обнуляет owner и затем
//! освобождает только listener-vector `CContainer`. Rust выражает inheritance
//! композицией двух safe state-owner-ов; embedded listeners конкретных goods-
//! контейнеров пока не регистрируются, поскольку оба их callback-а доказанно
//! сведены линкером к no-op.
//! GUID-object и typed-GUID forwarder-ы ниже делегируют concrete storage через
//! общий `ContainerGuidStorage`: null object сохраняет null, а type scalar,
//! как в EXE, не читается. Это заменяет только erased `CBaseObject*` и vtable,
//! не меняя identity либо порядок lookup/remove у concrete контейнеров.
//! `Add(CBaseObject*, void*)` точным direct jump приходит в base
//! `CContainer::Remove`, который всегда возвращает null: Rust materialизует
//! этот внешний ложный результат без mutation. `AddFromDB` сначала virtual
//! читает позицию, на occupied ветви пишет diagnostic callback и возвращает
//! false; на пустой позиции не вставляет goods, а возвращает только non-null
//! status incoming pointer. Устаревший source-pointer null при occupied slot
//! разыменовывался; Rust возвращает typed block без log и без UB.
//! `Remove(position, amount, context)` сначала создаёт split, сообщает о нём
//! listener-у и только затем уменьшает amount; full amount делегирует concrete
//! GUID removal. Общий safe adapter сохраняет этот порядок без RTTI/оригинал pointer.

use super::super::goods::cgoods::{CGoods, GoodsCodecError};
use super::super::goods::cgoodsbaseproperties::GAP_PARTICULAR_ATTRIBUTE;
use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use crate::public::guid::CGuid;
use super::ccontainer::{CContainerState, SharedContainerListener};

/// Наблюдаемый результат base `CGoodsContainer::AddFromDB`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BaseGoodsContainerDbAddOutcome {
 /// Позиция свободна и incoming non-null, но abstract base ничего не вставляет.
    AcceptedWithoutInsert,
 /// Позиция свободна, но legacy incoming pointer был null.
    RejectedMissingIncoming,
 /// Позиция занята: diagnostic уже отправлен callback-у.
    RejectedOccupied,
}

impl BaseGoodsContainerDbAddOutcome {
 /// Возвращает точный `int` normal-result owner-а.
    pub(crate) const fn legacy_result(self) -> i32 {
        match self {
            Self::AcceptedWithoutInsert => 1,
            Self::RejectedMissingIncoming | Self::RejectedOccupied => 0,
        }
    }
}

/// Safe-граница null dereference conflict-ветви base `AddFromDB`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BaseGoodsContainerDbAddBlock {
    MissingIncomingOnOccupiedPosition { position: u32 },
}

/// Concrete position/GUID storage для общего `Remove(position, amount)`.
///
/// `goods_at` и `remove_by_guid` соответствуют двум virtual slot-ам оригинал;
/// external listener остаётся явным callback-ом, а не erased base pointer.
pub(crate) trait GoodsContainerPositionStorage {
    fn goods_at(&mut self, position: u32) -> Option<&mut CGoods>;
    fn remove_by_guid(&mut self, ex_id: &CGuid) -> Option<Box<CGoods>>;
}

/// Действующее base-состояние исходного `CGoodsContainer`, не копия ABI.
pub(crate) struct CGoodsContainerState {
    container_base: CContainerState,
    owner_type: i32,
    owner_id: i32,
}

impl CGoodsContainerState {
 /// Создаёт base defaults constructor-а.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            container_base: CContainerState::with_constructor_defaults(),
            owner_type: 0,
            owner_id: 0,
        }
    }

 /// Base `Clear` доказанно не меняет ни owner, ни listeners.
    pub(crate) const fn clear(&mut self) {}

 /// Сбрасывает owner до очистки non-owning listener registry.
    pub(crate) fn release(&mut self) {
        self.owner_type = 0;
        self.owner_id = 0;
        self.container_base.release();
    }

 /// Сохраняет два signed owner scalar без дополнительных эффектов.
    pub(crate) const fn set_owner(&mut self, owner_type: i32, owner_id: i32) {
        self.owner_type = owner_type;
        self.owner_id = owner_id;
    }

    pub(crate) const fn owner_type(&self) -> i32 {
        self.owner_type
    }

    pub(crate) const fn owner_id(&self) -> i32 {
        self.owner_id
    }

    pub(crate) fn add_listener(&mut self, listener: Option<&SharedContainerListener>) -> i32 {
        self.container_base.add_listener(listener)
    }
}

/// Выполняет stacking-ветку для уже занятой позиции.
pub(crate) fn add_to_occupied_position(
    existing: &mut CGoods,
    incoming: Box<CGoods>,
    registry: &GoodsBasePropertiesRegistry,
) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
    let existing_index = existing
        .get_base_properties_index()
        .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
    let incoming_index = incoming
        .get_base_properties_index()
        .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
    if existing_index != incoming_index
        || existing.get_addon_property_value(GAP_PARTICULAR_ATTRIBUTE, 1)
            != incoming.get_addon_property_value(GAP_PARTICULAR_ATTRIBUTE, 1)
    {
        return Ok(Some(incoming));
    }

    let max_stack = existing.get_max_stack_number(registry)?;
    if max_stack <= 1 || incoming.get_amount() > max_stack.wrapping_sub(existing.get_amount()) {
        return Ok(Some(incoming));
    }

    existing.set_amount(existing.get_amount().wrapping_add(incoming.get_amount()));
    drop(incoming);
    Ok(None)
}

/// Материализует base `Add(CBaseObject*, void*)`.
///
/// Функция делает direct jump в `CContainer::Remove(GUID, void*)`, а base
/// implementation всегда возвращает null. Ни object, ни context не читаются.
pub(crate) const fn add_base_object() -> bool {
    false
}

/// Материализует abstract base `AddFromDB` без ложной вставки goods.
///
/// Lookup вызывается до проверки incoming pointer. Occupied-ветвь вызывает
/// callback ровно один раз и возвращает `0`; пустая позиция возвращает `1`
/// только для non-null incoming, сохраняя ownership у caller-а.
pub(crate) fn add_from_db_base<'incoming, 'stored, Lookup, Log>(
    incoming: Option<&'incoming CGoods>,
    position: u32,
    lookup: Lookup,
    log_collision: &mut Log,
) -> Result<BaseGoodsContainerDbAddOutcome, BaseGoodsContainerDbAddBlock>
where
    Lookup: FnOnce(u32) -> Option<&'stored CGoods>,
    Log: FnMut(&'incoming CGoods, &'stored CGoods, u32),
{
    let existing = lookup(position);
    let Some(existing) = existing else {
        return Ok(match incoming {
            Some(_) => BaseGoodsContainerDbAddOutcome::AcceptedWithoutInsert,
            None => BaseGoodsContainerDbAddOutcome::RejectedMissingIncoming,
        });
    };
    let incoming = incoming.ok_or(
        BaseGoodsContainerDbAddBlock::MissingIncomingOnOccupiedPosition { position },
    )?;
    log_collision(incoming, existing, position);
    Ok(BaseGoodsContainerDbAddOutcome::RejectedOccupied)
}

/// Материализует `CGoodsContainer::Remove(position, amount, context)`.
///
/// Partial remove создаёт новый goods, назначает amount, вызывает listener и
/// лишь затем уменьшает original с 32-bit wrapping subtraction. Full remove
/// делегируется concrete GUID owner-у; invalid position/amount и factory miss
/// остаются null result original-а.
pub(crate) fn remove_from_position<Storage, Create, Notify>(
    storage: &mut Storage,
    position: u32,
    amount: u32,
    registry: &GoodsBasePropertiesRegistry,
    create_goods: &mut Create,
    notify_split: &mut Notify,
) -> Result<Option<Box<CGoods>>, GoodsCodecError>
where
    Storage: GoodsContainerPositionStorage,
    Create: FnMut(u32) -> Option<Box<CGoods>>,
    Notify: FnMut(&CGoods),
{
    let Some(goods) = storage.goods_at(position) else {
        return Ok(None);
    };
    if amount == 0 {
        return Ok(None);
    }

    let current_amount = goods.get_amount();
    if amount < current_amount {
        let max_stack = goods.get_max_stack_number(registry)?;
        if max_stack <= 1 {
            return Ok(None);
        }
        let base_properties_index = goods
            .get_base_properties_index()
            .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
        let Some(mut split) = create_goods(base_properties_index) else {
            return Ok(None);
        };
        split.set_amount(amount);
        notify_split(&split);

 // Reentrant callback мог удалить original, что в old оригинал оставляло
 // dangling pointer. Safe adapter не записывает в пропавший slot и
 // возвращает split caller-у вместо internal lifetime defect.
        let Some(goods) = storage.goods_at(position) else {
            return Ok(Some(split));
        };
 // Оригинал повторно читает amount после listener-callback: callback может
 // менять тот же stack, но не обязан его удалять.
        goods.set_amount(goods.get_amount().wrapping_sub(amount));
        return Ok(Some(split));
    }

    if amount == current_amount {
        let ex_id = *goods.get_ex_id();
        return Ok(storage.remove_by_guid(&ex_id));
    }
    Ok(None)
}
