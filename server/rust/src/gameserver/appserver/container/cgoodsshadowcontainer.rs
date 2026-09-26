//! Metadata/query core `CGoodsShadowContainer` исторического GameServer.
//!
//! Точная пара `gameserver.exe + GameServer.pdb`; исходный owner
//! `server/gameserver/appserver/container/cgoodsshadowcontainer.cpp`. Shadow не
//! владеет `CGoods`: он хранит 40-байтовую metadata исходного player container
//! и на каждом read заново разрешает живой source owner. `BTreeMap` заменяет
//! legacy `std::map` и использует уже восстановленный raw-byte `CGuid::Ord`,
//! поэтому position/traversal остаются в точном GUID-order.
//!
//! Source player lookup/move/remove передаются явным resolver-ом и не
//! подменяются pointer cache. Storage, queries, amount callbacks, clear/release
//! и typed reports для listener/message dispatcher-а материализованы. Реальная
//! packet/equipment межконтейнерная move-транзакция и packet assembly замкнуты
//! player/container-message owner-ами; wallet/yuanbao и прочие общие routes
//! ещё требуют реконструкции; полный декомпилят хранится локально.

use std::collections::BTreeMap;

use super::ccontainer::{ContainerListenerHandle, PreviousContainer};
use super::cgoodscontainer::CGoodsContainer;
use crate::gameserver::appserver::goods::cgoods::CGoods;
use crate::gameserver::appserver::goods::cgoodsfactory::CGoodsFactory;
use nebokrai_shared::values::CGuid;

const PLAYER_CONTAINER_OWNER_TYPE: i32 = 400;
const INVALID_GOODS_POSITION: u32 = u32::MAX;

#[repr(C, align(4))]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct GoodsShadow {
    pub(crate) original_container_type: i32,
    pub(crate) original_container_id: i32,
    pub(crate) original_container_extend_id: i32,
    pub(crate) original_goods_position: u32,
    pub(crate) goods_id: CGuid,
    pub(crate) goods_base_properties_index: u32,
    pub(crate) goods_amount: u32,
}

const _: () = {
    assert!(std::mem::size_of::<GoodsShadow>() == 40);
    assert!(std::mem::align_of::<GoodsShadow>() == 4);
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlacedShadowGoods {
    pub(crate) identity: CGuid,
    pub(crate) position: u32,
    pub(crate) base_properties_index: u32,
    pub(crate) amount: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ShadowRecordBlock {
    Full,
    InvalidSourceOwner,
    UnsupportedSourceContainer,
}

#[must_use = "record определяет shadow identity и replacement semantics"]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ShadowRecorded {
    pub(crate) record: GoodsShadow,
    pub(crate) replaced: Option<GoodsShadow>,
}

#[must_use = "report содержит обязательные shadow message/listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShadowPresenceReport {
    pub(crate) owner_type: i32,
    pub(crate) owner_id: i32,
    pub(crate) container_extend_id: i32,
    pub(crate) position: u32,
    pub(crate) record: GoodsShadow,
    pub(crate) listeners: Vec<ContainerListenerHandle>,
}

#[must_use = "report содержит удалённую metadata и listener-эффекты"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShadowRemovedReport {
    pub(crate) presence: ShadowPresenceReport,
}

#[must_use = "source callback может удалить shadow или изменить публикуемое amount"]
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum ShadowSourceChangeOutcome {
    Missing,
    Unchanged,
    AmountChanged(ShadowPresenceReport),
    Removed(ShadowRemovedReport),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CGoodsShadowContainer {
    base: CGoodsContainer,
    shadows: BTreeMap<CGuid, GoodsShadow>,
    container_extend_id: i32,
}

impl Default for CGoodsShadowContainer {
    fn default() -> Self {
        Self::new()
    }
}

impl CGoodsShadowContainer {
    pub(crate) const fn new() -> Self {
        Self {
            base: CGoodsContainer::new(),
            shadows: BTreeMap::new(),
            container_extend_id: 0,
        }
    }

    pub(crate) const fn base(&self) -> &CGoodsContainer {
        &self.base
    }

    pub(crate) const fn base_mut(&mut self) -> &mut CGoodsContainer {
        &mut self.base
    }

    pub(crate) const fn set_container_extend_id(&mut self, extend_id: i32) {
        self.container_extend_id = extend_id;
    }

    pub(crate) const fn container_extend_id(&self) -> i32 {
        self.container_extend_id
    }

    pub(crate) const fn shadows(&self) -> &BTreeMap<CGuid, GoodsShadow> {
        &self.shadows
    }

    pub(crate) fn goods_amount(&self) -> u32 {
        self.shadows.len() as u32
    }

    pub(crate) const fn is_supported_player_container(extend_id: i32) -> bool {
        matches!(extend_id, 1 | 2 | 4 | 5)
    }

    /// Фактический source add уже выполнен caller-ом; здесь фиксируется его
    /// результат. Auto-position (`u32::MAX`) заменяется реальной позицией.
    pub(crate) fn record_placed_goods(
        &mut self,
        previous: PreviousContainer,
        placed: PlacedShadowGoods,
        has_capacity: bool,
    ) -> Result<ShadowRecorded, ShadowRecordBlock> {
        if !has_capacity {
            return Err(ShadowRecordBlock::Full);
        }
        if previous.container_type != PLAYER_CONTAINER_OWNER_TYPE {
            return Err(ShadowRecordBlock::InvalidSourceOwner);
        }
        if !Self::is_supported_player_container(previous.container_extend_id) {
            return Err(ShadowRecordBlock::UnsupportedSourceContainer);
        }
        let original_goods_position = if previous.goods_position == INVALID_GOODS_POSITION {
            placed.position
        } else {
            previous.goods_position
        };
        let record = GoodsShadow {
            original_container_type: previous.container_type,
            original_container_id: previous.container_id,
            original_container_extend_id: previous.container_extend_id,
            original_goods_position,
            goods_id: placed.identity,
            goods_base_properties_index: placed.base_properties_index,
            goods_amount: placed.amount,
        };
        let replaced = self.shadows.insert(placed.identity, record);
        Ok(ShadowRecorded { record, replaced })
    }

    /// Derived owners с собственным доказанным контрактом прямого `map[]`
    /// используют этот primitive без player/extend/capacity нормализации.
    pub(crate) fn insert_shadow_record(&mut self, record: GoodsShadow) -> ShadowRecorded {
        let replaced = self.shadows.insert(record.goods_id, record);
        ShadowRecorded { record, replaced }
    }

    pub(crate) fn query_goods_position(&self, goods_id: CGuid) -> Option<u32> {
        self.shadows
            .keys()
            .position(|candidate| *candidate == goods_id)
            .map(|position| position as u32)
    }

    pub(crate) fn goods_id_at(&self, position: u32) -> Option<CGuid> {
        self.shadows.keys().nth(position as usize).copied()
    }

    pub(crate) fn original_container_information(
        &self,
        goods_id: CGuid,
    ) -> Option<PreviousContainer> {
        self.shadows.get(&goods_id).map(|shadow| PreviousContainer {
            container_type: shadow.original_container_type,
            container_id: shadow.original_container_id,
            container_extend_id: shadow.original_container_extend_id,
            goods_position: shadow.original_goods_position,
        })
    }

    pub(crate) fn goods_source_container_extend_id(&self, goods_id: CGuid) -> i32 {
        self.shadows
            .get(&goods_id)
            .map_or(0, |shadow| shadow.original_container_extend_id)
    }

    pub(crate) fn shadow_position(&self, previous: &PreviousContainer) -> Option<u32> {
        self.shadows
            .iter()
            .find(|(_, shadow)| {
                shadow.original_container_type == previous.container_type
                    && shadow.original_container_id == previous.container_id
                    && shadow.original_container_extend_id == previous.container_extend_id
                    && shadow.original_goods_position == previous.goods_position
            })
            .and_then(|(goods_id, _)| self.query_goods_position(*goods_id))
    }

    pub(crate) fn find<'a, Resolve>(
        &self,
        goods_id: CGuid,
        mut resolve: Resolve,
    ) -> Option<&'a CGoods>
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&'a CGoods>,
    {
        resolve(self.shadows.get(&goods_id)?)
    }

    pub(crate) fn get_goods<'a, Resolve>(
        &self,
        position: u32,
        mut resolve: Resolve,
    ) -> Option<&'a CGoods>
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&'a CGoods>,
    {
        let (_, shadow) = self.shadows.iter().nth(position as usize)?;
        resolve(shadow)
    }

    pub(crate) fn get_first_goods<'a, Resolve>(
        &self,
        base_properties_index: u32,
        mut resolve: Resolve,
    ) -> Option<&'a CGoods>
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&'a CGoods>,
    {
        self.shadows
            .values()
            .filter(|shadow| shadow.goods_base_properties_index == base_properties_index)
            .find_map(&mut resolve)
    }

    pub(crate) fn get_goods_by_base_properties<'a, Resolve>(
        &self,
        base_properties_index: u32,
        mut resolve: Resolve,
    ) -> Vec<&'a CGoods>
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&'a CGoods>,
    {
        self.shadows
            .values()
            .filter(|shadow| shadow.goods_base_properties_index == base_properties_index)
            .filter_map(&mut resolve)
            .collect()
    }

    pub(crate) fn is_goods_existed<'a, Resolve>(
        &self,
        base_properties_index: u32,
        resolve: Resolve,
    ) -> bool
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&'a CGoods>,
    {
        self.get_first_goods(base_properties_index, resolve)
            .is_some()
    }

    pub(crate) fn traversing_goods<'a, Resolve>(&self, mut resolve: Resolve) -> Vec<&'a CGoods>
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&'a CGoods>,
    {
        self.shadows
            .values()
            .filter_map(|shadow| {
                resolve(shadow).filter(|goods| goods.amount() == shadow.goods_amount)
            })
            .collect()
    }

    pub(crate) fn contents_weight<Resolve>(
        &self,
        factory: &CGoodsFactory,
        mut resolve: Resolve,
    ) -> u32
    where
        Resolve: FnMut(&GoodsShadow) -> Option<&CGoods>,
    {
        self.shadows.values().fold(0u32, |weight, shadow| {
            weight.wrapping_add(resolve(shadow).map_or(0, |goods| goods.weight(factory)))
        })
    }

    pub(crate) fn add_shadow_report(&self, goods_id: CGuid) -> Option<ShadowPresenceReport> {
        self.presence_report(goods_id)
    }

    pub(crate) fn remove_shadow(&mut self, goods_id: CGuid) -> Option<ShadowRemovedReport> {
        let presence = self.presence_report(goods_id)?;
        self.shadows.remove(&goods_id);
        Some(ShadowRemovedReport { presence })
    }

    /// Exact callback пропускает update, когда прежнее shadow amount равно
    /// самому add-delta; это странное сравнение сохранено буквально.
    pub(crate) fn on_source_added(
        &mut self,
        goods_id: CGuid,
        amount: u32,
    ) -> ShadowSourceChangeOutcome {
        let Some(shadow) = self.shadows.get_mut(&goods_id) else {
            return ShadowSourceChangeOutcome::Missing;
        };
        if shadow.goods_amount == amount {
            return ShadowSourceChangeOutcome::Unchanged;
        }
        shadow.goods_amount = shadow.goods_amount.wrapping_add(amount);
        ShadowSourceChangeOutcome::AmountChanged(
            self.presence_report(goods_id)
                .expect("shadow существует после amount add"),
        )
    }

    pub(crate) fn on_source_removed(
        &mut self,
        goods_id: CGuid,
        amount: u32,
    ) -> ShadowSourceChangeOutcome {
        let Some(shadow) = self.shadows.get_mut(&goods_id) else {
            return ShadowSourceChangeOutcome::Missing;
        };
        if shadow.goods_amount < amount {
            return ShadowSourceChangeOutcome::Removed(
                self.remove_shadow(goods_id)
                    .expect("shadow существует до oversized remove"),
            );
        }
        shadow.goods_amount = shadow.goods_amount.wrapping_sub(amount);
        if shadow.goods_amount == 0 {
            return ShadowSourceChangeOutcome::Removed(
                self.remove_shadow(goods_id)
                    .expect("shadow существует до zero remove"),
            );
        }
        ShadowSourceChangeOutcome::AmountChanged(
            self.presence_report(goods_id)
                .expect("shadow существует после partial remove"),
        )
    }

    pub(crate) fn clear(&mut self) -> usize {
        let count = self.shadows.len();
        self.shadows.clear();
        count
    }

    pub(crate) fn release(&mut self) -> usize {
        self.container_extend_id = 0;
        self.base.release();
        self.clear()
    }

    fn presence_report(&self, goods_id: CGuid) -> Option<ShadowPresenceReport> {
        let record = *self.shadows.get(&goods_id)?;
        Some(ShadowPresenceReport {
            owner_type: self.base.owner_type(),
            owner_id: self.base.owner_id(),
            container_extend_id: self.container_extend_id,
            position: self.query_goods_position(goods_id)?,
            record,
            listeners: self.base.base().listener_snapshot(),
        })
    }
}
