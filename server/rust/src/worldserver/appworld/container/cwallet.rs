//! Однослотовый `CWallet` из `cwallet.cpp/.h`, подтверждённый
//! `worldserver.exe` и `worldserver.pdb`.
//!
//! Пустой wallet принимает первый товар без проверки currency index; занятый
//! складывает только gold coins. `AddFromDB` может перезаписать занятый slot.
//! Rust освобождает вытесненный объект вместо внутренней утечки оригинала.
//!
//! Wire — marker `0/1` и, при единице, полный `CGoods`; decode сначала
//! выполняет `Release`. Query по base index сравнивает разрешённый gold index,
//! а не фактический товар. Largess использует отдельный gold limit и сохраняет
//! unsigned wrapping остатков и суммы.

use crate::dbaccess::worlddb::goodslistener::TraversedGoods;
use nebokrai_shared::values::CGuid;

use super::super::goods::cgoods::{CGoods, GoodsCodecError, GoodsDbSnapshotBlock};
use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use super::ccontainer::ContainerGuidStorage;
use super::cgoodscontainer::{CGoodsContainerState, add_to_occupied_position};

pub(crate) struct CWallet {
    pub(super) container_base: CGoodsContainerState,
    pub(super) gold_coins: Option<Box<CGoods>>,
}

/// Единственный wallet-slot является concrete target-ом inherited GUID slots
/// `CContainer`: полное равенство GUID, без currency-index фильтра.
impl ContainerGuidStorage for CWallet {
    type Object = CGoods;
    type Removed = Box<CGoods>;

    fn find_by_guid(&self, ex_id: &CGuid) -> Option<&Self::Object> {
        self.gold_coins
            .as_deref()
            .filter(|goods| goods.get_ex_id() == ex_id)
    }

    fn remove_by_guid(&mut self, ex_id: &CGuid) -> Option<Self::Removed> {
        if self
            .gold_coins
            .as_deref()
            .is_some_and(|goods| goods.get_ex_id() == ex_id)
        {
            return self.gold_coins.take();
        }
        None
    }
}

impl CWallet {
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            container_base: CGoodsContainerState::with_constructor_defaults(),
            gold_coins: None,
        }
    }

    pub(crate) const fn get_gold_coins_amount(&self) -> u32 {
        match &self.gold_coins {
            Some(goods) => goods.get_amount(),
            None => 0,
        }
    }

    pub(crate) fn is_goods_existed(
        &self,
        base_properties_index: u32,
        gold_coin_index: u32,
    ) -> bool {
        self.gold_coins.is_some() && base_properties_index == gold_coin_index
    }

    pub(crate) fn get_the_first_goods(
        &self,
        base_properties_index: u32,
        gold_coin_index: u32,
    ) -> Option<&CGoods> {
        (base_properties_index == gold_coin_index)
            .then_some(self.gold_coins.as_deref())
            .flatten()
    }

    pub(crate) fn get_goods_by_base_index(
        &self,
        base_properties_index: u32,
        gold_coin_index: u32,
    ) -> impl Iterator<Item = &CGoods> {
        self.gold_coins
            .iter()
            .map(Box::as_ref)
            .filter(move |_| base_properties_index == gold_coin_index)
    }

    pub(crate) fn add_at(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        gold_coin_index: u32,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        let Some(existing) = self.gold_coins.as_deref_mut() else {
            self.gold_coins = Some(goods);
            return Ok(None);
        };

        let incoming_index = goods
            .get_base_properties_index()
            .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
        if incoming_index != gold_coin_index || position != 0 {
            return Ok(Some(goods));
        }

        add_to_occupied_position(existing, goods, registry)
    }

    pub(crate) fn add(
        &mut self,
        goods: Box<CGoods>,
        gold_coin_index: u32,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        self.add_at(0, goods, gold_coin_index, registry)
    }

    pub(crate) fn add_gold_coin_of_largess(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        gold_coin_limit: u32,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        let Some(existing) = ((position == 0).then_some(self.gold_coins.as_deref_mut())).flatten()
        else {
            self.gold_coins = Some(goods);
            return Ok(None);
        };

        let existing_index = existing
            .get_base_properties_index()
            .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
        let incoming_index = goods
            .get_base_properties_index()
            .ok_or(GoodsCodecError::MissingBasePropertiesIndex)?;
        if existing_index != incoming_index
            || goods.get_amount() > gold_coin_limit.wrapping_sub(existing.get_amount())
        {
            return Ok(Some(goods));
        }

        existing.set_amount(existing.get_amount().wrapping_add(goods.get_amount()));
        drop(goods);
        Ok(None)
    }

    pub(crate) fn add_from_db(&mut self, position: u32, goods: Box<CGoods>) -> Option<Box<CGoods>> {
        if self.get_goods(position).is_some() {
            return Some(goods);
        }
        self.gold_coins = Some(goods);
        None
    }

    pub(crate) fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, GoodsDbSnapshotBlock> {
        self.gold_coins
            .iter()
            .map(|goods| {
                Ok(TraversedGoods {
                    goods: goods.db_save_snapshot(registry)?,
                    position: 0,
                })
            })
            .collect()
    }
}
