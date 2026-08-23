//! Владелец wallet-container исторического `WorldServer`.
//!
//! Состояние конструктора `CWallet::CWallet`,
//! destructor ownership, `GetGoldCoinsAmount`
//! query-family,
//! `AddFromDB` и обеих перегрузок `Add`
//! а также `AddGoldCoinOfLargess`
//! входят в контракт owner-а. Источник контракта — точная пара WorldServer EXE/PDB.
//!
//! Layout сохраняет единственное собственное поле `m_pGoldCoins` по `+0x24`;
//! inherited `CGoodsContainer` хранит signed owner type/ID по `+0x14/+0x18`.
//! Constructor начинает с owner `0/0` и null goods, а destructor сначала
//! выполняет base `Release`, затем `GarbageCollect`. Rust `Option<Box<CGoods>>`
//! заменяет только nullable pointer и deleting-destructor: обычный `Drop`
//! уничтожает товар один раз. Встроенный secondary listener регистрируется
//! constructor-ом, но оба его callbacks сведены линкером к доказанному
//! `mov eax,1; ret 0xC` по; отдельного состояния он не имеет.
//!
//! positional `Add` проверяет gold-coin index только когда slot уже
//! занят; первый товар принимается без currency-validation. Process-global
//! `GetGoldCoinIndex` заменён явно переданным resolved index, а общая stacking-
//! ветка — действующим адаптером `CGoodsContainer::Add`. `AddFromDB` сначала
//! вызывает virtual positional `GetGoods`, поэтому ненулевая позиция способна
//! перезаписать уже занятый единственный slot. Legacy теряет прежний указатель;
//! Rust сохраняет наблюдаемую перезапись, но не воспроизводит внутреннюю утечку
//! и освобождает вытесненный объект обычным ownership. Listener callbacks
//! доказанно no-op и не материализуются.
//!
//! Три base-index query используют resolved gold-coin index, а не индекс
//! фактически сохранённого товара. `GetGoods(index, vector)` принимает
//! vector по значению и уничтожает наполненную копию, поэтому не способен
//! вернуть результат. Этот внутренний дефект сигнатуры без ожидаемого внешнего
//! эффекта исправлен естественным Rust iterator-ом; критерий выбора сохранён.
//!
//! `AddGoldCoinOfLargess` сравнивает base-index двух товаров, но использует
//! отдельный global gold limit вместо обычного max-stack. Вычитание остатка и
//! сложение количества — подтверждённая `u32` wrapping-арифметика; она
//! сохранена, поскольку меняет итоговый баланс. Единственный call site
//! передаёт позицию `0` и статически вызывает `CWallet` на bank-base, обходя
//! bank lock.

use crate::dbaccess::worlddb::goodslistener::TraversedGoods;
use crate::public::guid::CGuid;

use super::super::goods::cgoods::{CGoods, GoodsCodecError, GoodsDbSnapshotBlock};
use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use super::ccontainer::ContainerGuidStorage;
use super::cgoodscontainer::{CGoodsContainerState, add_to_occupied_position};

/// Действующее состояние исходного `CWallet`, не копия его 32-битного ABI.
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
 /// Создаёт точные defaults base-owner-а и пустого wallet slot-а.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            container_base: CGoodsContainerState::with_constructor_defaults(),
            gold_coins: None,
        }
    }

 /// Возвращает количество единственного gold-coins товара либо ноль.
    pub(crate) const fn get_gold_coins_amount(&self) -> u32 {
        match &self.gold_coins {
            Some(goods) => goods.get_amount(),
            None => 0,
        }
    }

 /// Проверяет наличие wallet-slot-а для resolved gold-coin index.
    pub(crate) fn is_goods_existed(
        &self,
        base_properties_index: u32,
        gold_coin_index: u32,
    ) -> bool {
        self.gold_coins.is_some() && base_properties_index == gold_coin_index
    }

 /// Возвращает единственный slot только для resolved gold-coin index.
    pub(crate) fn get_the_first_goods(
        &self,
        base_properties_index: u32,
        gold_coin_index: u32,
    ) -> Option<&CGoods> {
        (base_properties_index == gold_coin_index)
            .then_some(self.gold_coins.as_deref())
            .flatten()
    }

 /// Возвращает usable Rust-view вместо бесполезной legacy vector-by-value копии.
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

 /// Вставляет товар в позицию wallet-а; `Some` сохраняет ownership при false.
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

 /// Делегирует object-перегрузку позиции `0` после уже выполненного cast-а.
    pub(crate) fn add(
        &mut self,
        goods: Box<CGoods>,
        gold_coin_index: u32,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        self.add_at(0, goods, gold_coin_index, registry)
    }

 /// Начисляет largess с отдельным global limit и wrapping-арифметикой.
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

 /// Вставляет DB-товар после positional collision-check без factory-validation.
    pub(crate) fn add_from_db(&mut self, position: u32, goods: Box<CGoods>) -> Option<Box<CGoods>> {
        if self.get_goods(position).is_some() {
            return Some(goods);
        }
        self.gold_coins = Some(goods);
        None
    }

 /// Замораживает nullable wallet-slot для DB traversal с позицией `0`.
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
