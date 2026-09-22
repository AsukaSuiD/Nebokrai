//! Общая часть `CGoodsContainer` из `cgoodscontainer.cpp/.h`, подтверждённая
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Stacking проверяет base index, particular attribute и max-stack именно в
//! исходном порядке; amount и свободное место используют unsigned wrapping.
//! Успех поглощает incoming товар, отказ возвращает владение вызывающему.
//!
//! Встроенные listeners фактически no-op. `Clear` не меняет base-state,
//! `Release` обнуляет owner и освобождает listener storage. Общие GUID-wrapper-ы
//! передают lookup/remove конкретному контейнеру без нового порядка.
//!
//! Base `AddFromDB` лишь проверяет позицию и не вставляет товар. Частичный
//! `Remove` создаёт split и уведомляет listener до уменьшения исходного amount.

use super::super::goods::cgoods::{CGoods, GoodsCodecError};
use super::super::goods::cgoodsbaseproperties::GAP_PARTICULAR_ATTRIBUTE;
use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use nebokrai_shared::values::CGuid;
use super::ccontainer::{CContainerState, SharedContainerListener};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BaseGoodsContainerDbAddOutcome {
    AcceptedWithoutInsert,
    RejectedMissingIncoming,
    RejectedOccupied,
}

impl BaseGoodsContainerDbAddOutcome {
    pub(crate) const fn legacy_result(self) -> i32 {
        match self {
            Self::AcceptedWithoutInsert => 1,
            Self::RejectedMissingIncoming | Self::RejectedOccupied => 0,
        }
    }
}

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

pub(crate) struct CGoodsContainerState {
    container_base: CContainerState,
    owner_type: i32,
    owner_id: i32,
}

impl CGoodsContainerState {
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            container_base: CContainerState::with_constructor_defaults(),
            owner_type: 0,
            owner_id: 0,
        }
    }

    pub(crate) const fn clear(&mut self) {}

    pub(crate) fn release(&mut self) {
        self.owner_type = 0;
        self.owner_id = 0;
        self.container_base.release();
    }

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

/// Создаёт base `Add(CBaseObject*, void*)`.
///
/// Функция делает direct jump в `CContainer::Remove(GUID, void*)`, а base
/// implementation всегда возвращает null. Ни object, ни context не читаются.
pub(crate) const fn add_base_object() -> bool {
    false
}

/// Создаёт abstract base `AddFromDB` без ложной вставки goods.
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

/// Создаёт `CGoodsContainer::Remove(position, amount, context)`.
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
