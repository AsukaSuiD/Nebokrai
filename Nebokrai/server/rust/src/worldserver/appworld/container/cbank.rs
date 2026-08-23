//! Владелец bank-container исторического `WorldServer`.
//!
//! Состояние конструктора и деструктора,
//! `Release/Clear` и унаследованного wallet-codec
//! а также lock-gated `Find/Remove/Add/AddFromDB`
//!
//! входят в контракт owner-а. Источник контракта — точная пара WorldServer EXE/PDB.
//!
//! Layout сохраняет класс размером `0x2C`: base `CWallet` по `+0x0` и
//! единственное собственное поле `bool m_bLocked` по `+0x28`. Constructor
//! создаёт обычный wallet-state и `false`; `Clear` и `Release` сначала также
//! снимают lock, затем делегируют различающимся wallet-операциям. Destructor
//! повторяет `Release`, после чего base ownership уничтожается обычным Rust
//! `Drop`; vtable/EH/deleting-destructor cleanup отдельно не восстанавливается.
//!
//! PDB не содержит собственных `CBank::Serialize/Unserialize`: clone-wire
//! наследуется от `CWallet` и остаётся marker-байтом с возможным полным
//! `CGoods`. В исходном virtual-вызове `Release` wallet-decoder видит override
//! `CBank`, поэтому перед чтением marker lock обязательно становится `false`;
//! Rust выполняет ту же мутацию перед готовым wallet-helper-ом. Сам lock в wire
//! не входит. Короткий source оставляет bank уже разблокированным и очищенным,
//! затем возвращает локальную типизированную ошибку короткого источника.
//!
//! `CWallet` служит узким compatibility-layer для доказанных base-state и
//! codec-а. `Option<Box<CGoods>>` заменяет nullable pointer и
//! `GarbageCollect`, а встроенный listener остаётся ранее доказанным no-op
//!
//!
//! `Find`, `Remove` и обе перегрузки `Add` проверяют lock до wallet-вызова.
//! `AddFromDB` отличается: сначала inherited positional `GetGoods` проверяет
//! collision даже у locked bank, и лишь для пустого результата проверяется
//! lock. DB diagnostic logging исключён как техническая замена, не влияющая на
//! ownership или результат. Process-global gold index передаётся wallet-слою
//! явно уже разрешённым значением.
//! `CLargess::AddGoldCoin` делает статически qualified base-вызов и тем самым
//! намеренно обходит bank lock; Rust compatibility-adapter ниже делегирует без
//! собственной lock-проверки.

use super::super::goods::cgoods::{CGoods, GoodsCodecError};
use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use super::cwallet::CWallet;
use crate::dbaccess::worlddb::goodslistener::TraversedGoods;
use crate::public::guid::CGuid;

/// Действующее состояние исходного `CBank`, не копия его 32-битного ABI.
pub(crate) struct CBank {
    wallet_state: CWallet,
    locked: bool,
}

impl CBank {
 /// Создаёт пустой unlocked bank с inherited owner `0/0`.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            wallet_state: CWallet::with_constructor_defaults(),
            locked: false,
        }
    }

 /// Снимает lock и очищает slot, сохраняя inherited owner type/ID.
    pub(crate) fn clear(&mut self) {
        self.locked = false;
        self.wallet_state.clear();
    }

 /// Снимает lock, сбрасывает inherited owner type/ID и уничтожает slot.
    pub(crate) fn release(&mut self) {
        self.locked = false;
        self.wallet_state.release();
    }

 /// Проверяет достижение max-stack единственного товара.
    pub(crate) fn is_full(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, GoodsCodecError> {
        self.wallet_state.is_full(registry)
    }

 /// Возвращает товар только для унаследованной единственной позиции `0`.
    pub(crate) fn get_goods(&self, position: u32) -> Option<&CGoods> {
        self.wallet_state.get_goods(position)
    }

 /// Возвращает mutable DB-view slot-а независимо от lock, как collision lookup.
    pub(crate) fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        self.wallet_state.get_goods_mut(position)
    }

 /// Возвращает число занятых bank-slot-ов: ноль либо один.
    pub(crate) const fn get_goods_amount(&self) -> u32 {
        self.wallet_state.get_goods_amount()
    }

 /// Ищет товар только в unlocked bank.
    pub(crate) fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        (!self.locked)
            .then(|| self.wallet_state.find(ex_id))
            .flatten()
    }

 /// Вынимает товар только из unlocked bank.
    pub(crate) fn remove(&mut self, ex_id: &CGuid) -> Option<Box<CGoods>> {
        if self.locked {
            return None;
        }
        self.wallet_state.remove(ex_id)
    }

 /// Делегирует object-перегрузку `Add` только из unlocked state.
    pub(crate) fn add(
        &mut self,
        goods: Box<CGoods>,
        gold_coin_index: u32,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        if self.locked {
            return Ok(Some(goods));
        }
        self.wallet_state.add(goods, gold_coin_index, registry)
    }

 /// Делегирует positional `Add` только из unlocked state.
    pub(crate) fn add_at(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        gold_coin_index: u32,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        if self.locked {
            return Ok(Some(goods));
        }
        self.wallet_state
            .add_at(position, goods, gold_coin_index, registry)
    }

 /// Проверяет positional collision раньше lock и затем делегирует DB-вставку.
    pub(crate) fn add_from_db(&mut self, position: u32, goods: Box<CGoods>) -> Option<Box<CGoods>> {
        if self.wallet_state.get_goods(position).is_some() || self.locked {
            return Some(goods);
        }
        self.wallet_state.add_from_db(position, goods)
    }

 /// Сохраняет qualified `CWallet`-вызов largess, обходящий bank lock.
    pub(crate) fn add_gold_coin_of_largess(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        gold_coin_limit: u32,
    ) -> Result<Option<Box<CGoods>>, GoodsCodecError> {
        self.wallet_state
            .add_gold_coin_of_largess(position, goods, gold_coin_limit)
    }

 /// Замораживает единственный bank-slot без изменения lock-state.
    pub(crate) fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, super::super::goods::cgoods::GoodsDbSnapshotBlock> {
        self.wallet_state.db_save_entries(registry)
    }

 /// Кодирует унаследованный marker/goods-wire без сериализации lock-а.
    pub(crate) fn serialize(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
    ) -> Result<bool, GoodsCodecError> {
        self.wallet_state.serialize(destination, include_child)
    }

 /// Снимает lock и декодирует унаследованный marker/goods-wire после `Release`.
    pub(crate) fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, GoodsCodecError> {
        self.locked = false;
        self.wallet_state.unserialize_with_marker_field(
            source,
            cursor,
            include_child,
            registry,
            "CBank marker",
        )
    }
}
