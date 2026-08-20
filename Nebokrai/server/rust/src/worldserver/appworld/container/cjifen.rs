//! Wallet-различия, скомпилированные из исторического `cjifen.cpp`.
//!
//! Статус `CWallet::Release/IsFull/GetGoods/GetGoodsAmount/Serialize/Unserialize`
//! RVA `0x000D5E30/0x000D5E50/0x000D5EC0/0x000D5EE0/0x000D5EF0/0x000D6030`
//! и `Clear/QueryGoodsPosition/Find/Remove` RVA
//! `0x000D8670/0x000D8350/0x000D8370/0x000D87C0/0x000D8A10`, traversal RVA
//! `0x000D8690`, а также constructor/destructor-state `CJiFen`
//! RVA `0x000D8280/0x000D82E0` и его folded container-контракт —
//! `IMPLEMENTED`; остальные wallet-операции ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально).
//! Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:18,34,124,133,142,180,193,236,261,273,289,322,336`.
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
//! `IsFull` сравнивает amount с exact max-stack текущего товара. Встроенные
//! wallet callbacks имеют общий no-op RVA `0x000DBD10`, поэтому в достигнутых
//! операциях нет скрытой мутации. Безразмерное legacy-чтение marker-а выражено
//! локальным `BLOCKED_MISSING_FACT` через typed short-source ошибку.
//! Object-перегрузка `QueryGoodsPosition` не сравнивает pointer identity: exact
//! ASM `0x004D835A..0x004D8366` передаёт GUID товара в virtual GUID-overload.
//! Поэтому другой объект с тем же полным GUID также получает позицию `0`.
//!
//! Exact PDB задаёт `CJiFen` как отдельный класс размера `0x28`: bases
//! `CGoodsContainer +0x0`, `CContainerListener +0x20` и единственное поле
//! `m_pGoldCoins: CGoods* +0x24`. Его публичные `Clear/Release/IsFull/GetGoods/`
//! `GetGoodsAmount/Serialize/Unserialize` имеют те же RVA, что `CWallet`;
//! это linker folding, а не наследование. Rust сохраняет отдельный nominal
//! owner и делегирует только доказанному общему состоянию/коду. Обычные
//! `Option<Box<CGoods>>` и `Drop` заменяют nullable pointer, `GarbageCollect`,
//! deleting-destructor и EH cleanup; listener callbacks — общий доказанный
//! no-op RVA `0x000DBD10`.

use super::super::goods::cgoods::{CGoods, GoodsCodecError};
use super::super::goods::cgoodsfactory::{GoodsBasePropertiesRegistry, unserialize_goods};
use super::super::listener::ccontainerlistener::{CContainerListener, TraversedContainerObject};
use super::cwallet::CWallet;
use crate::dbaccess::worlddb::goodslistener::TraversedGoods;
use crate::public::guid::CGuid;

/// Отдельный достигнутый owner исходного `CJiFen`, не копия его 32-битного ABI.
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

    /// Проверяет достижение exact max-stack единственного товара.
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

    /// Возвращает число занятых slot-ов: ноль либо один.
    pub(crate) const fn get_goods_amount(&self) -> u32 {
        self.wallet_state.get_goods_amount()
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
        self.owner_type = 0;
        self.owner_id = 0;
        self.gold_coins = None;
    }

    /// Проверяет достижение exact max-stack единственного товара.
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

    /// Возвращает число занятых wallet-slot-ов: ноль либо один.
    pub(crate) const fn get_goods_amount(&self) -> u32 {
        self.gold_coins.is_some() as u32
    }

    /// Ищет единственный товар по полному 16-байтовому legacy GUID.
    pub(crate) fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        self.gold_coins
            .as_deref()
            .filter(|goods| goods.get_ex_id() == ex_id)
    }

    /// Передаёт ownership единственного совпавшего товара вызывающему.
    pub(crate) fn remove(&mut self, ex_id: &CGuid) -> Option<Box<CGoods>> {
        if self
            .gold_coins
            .as_deref()
            .is_some_and(|goods| goods.get_ex_id() == ex_id)
        {
            return self.gold_coins.take();
        }
        None
    }

    /// Запрашивает позицию non-null объекта через его GUID, как exact virtual call.
    pub(crate) fn query_goods_position_by_object(&self, goods: Option<&CGoods>) -> Option<u32> {
        self.query_goods_position(goods?.get_ex_id())
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
            // BLOCKED_MISSING_FACT: legacy owner не получал длину source.
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

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp

// ============================================================================
// FUNCTION: CWallet::Release
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:133
// RVA: 0x000D5E30
// ADDRESS: 004d5e30
// PROTOTYPE: void __thiscall Release(void)
//
// IMPLEMENTED выше; owner сбрасывается раньше уничтожения slot-а.

// ============================================================================
// FUNCTION: CWallet::IsFull
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:142
// RVA: 0x000D5E50
// ADDRESS: 004d5e50
// PROTOTYPE: int __thiscall IsFull(void)
//
// IMPLEMENTED выше; сравнение остаётся unsigned `max_stack <= amount`.

// ============================================================================
// FUNCTION: CWallet::GetGoods
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:236
// RVA: 0x000D5EC0
// ADDRESS: 004d5ec0
// PROTOTYPE: CGoods * __thiscall GetGoods(ulong param_1)
//
// IMPLEMENTED выше; `nullptr` выражен заимствованным `Option`.

// ============================================================================
// FUNCTION: CWallet::GetGoodsAmount
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:261
// RVA: 0x000D5EE0
// ADDRESS: 004d5ee0
// PROTOTYPE: ulong __thiscall GetGoodsAmount(void)
//
// IMPLEMENTED выше; результат буквально равен `0` либо `1`.

// ============================================================================
// FUNCTION: CWallet::Serialize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:322
// RVA: 0x000D5EF0
// ADDRESS: 004d5ef0
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// IMPLEMENTED выше; goods-result исходно игнорируется после marker `1`.

// ============================================================================
// FUNCTION: CWallet::Unserialize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:336
// RVA: 0x000D6030
// ADDRESS: 004d6030
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// IMPLEMENTED выше; `param_3` не читается, factory decode всегда использует
// доказанный `include_child=true`.

// ============================================================================
// FUNCTION: CJiFen::IsGoodsExisted
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:211
// RVA: 0x000D8240
// ADDRESS: 004d8240
// PROTOTYPE: int __thiscall IsGoodsExisted(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJiFen::GetTheFirstGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:224
// RVA: 0x000D8260
// ADDRESS: 004d8260
// PROTOTYPE: CGoods * __thiscall GetTheFirstGoods(ulong param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJiFen::CJiFen
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:18
// RVA: 0x000D8280
// ADDRESS: 004d8280
// PROTOTYPE: undefined __thiscall CJiFen(void)
//
// IMPLEMENTED выше; owner `0/0`, slot пуст, listener не хранит Rust-state.
//

// ============================================================================
// FUNCTION: CJiFen::~CJiFen
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:34
// RVA: 0x000D82E0
// ADDRESS: 004d82e0
// PROTOTYPE: void __thiscall ~CJiFen(void)
//
// IMPLEMENTED через owned `Option<Box<CGoods>>` и обычный Rust `Drop`.
//

// ============================================================================
// FUNCTION: CWallet::QueryGoodsPosition
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:180
// RVA: 0x000D8350
// ADDRESS: 004d8350
// PROTOTYPE: int __thiscall QueryGoodsPosition(CGoods * param_1, ulong * param_2)
//
// IMPLEMENTED выше как GUID-routing; null и отсутствие дают `None`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWallet::QueryGoodsPosition
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:193
// RVA: 0x000D8370
// ADDRESS: 004d8370
// PROTOTYPE: int __thiscall QueryGoodsPosition(CGUID * param_1, ulong * param_2)
//
// IMPLEMENTED выше; successful out-position `0` выражена `Some(0)`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJiFen::AddFromDB
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:82
// RVA: 0x000D83C0
// ADDRESS: 004d83c0
// PROTOTYPE: int __thiscall AddFromDB(CGoods * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJiFen::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:55
// RVA: 0x000D8520
// ADDRESS: 004d8520
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJiFen::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:41
// RVA: 0x000D85C0
// ADDRESS: 004d85c0
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CJiFen::GetGoods
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:247
// RVA: 0x000D8600
// ADDRESS: 004d8600
// PROTOTYPE: void __thiscall GetGoods(ulong param_1, vector<CGoods*,std::allocator<CGoods*>_> param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWallet::Clear
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:124
// RVA: 0x000D8670
// ADDRESS: 004d8670
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// IMPLEMENTED выше; inherited owner сохраняется, `param_1` не наблюдается.

// ============================================================================
// FUNCTION: CWallet::TraversingContainer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:158
// RVA: 0x000D8690
// ADDRESS: 004d8690
// PROTOTYPE: void __thiscall TraversingContainer(CContainerListener * param_1)
//
// IMPLEMENTED выше через общий safe listener trait; null не вызывает callback.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWallet::Find
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:273
// RVA: 0x000D87C0
// ADDRESS: 004d87c0
// PROTOTYPE: CBaseObject * __thiscall Find(CGUID * param_1)
//
// IMPLEMENTED выше через полное равенство достигнутого `CGuid`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CWallet::Remove
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp:289
// RVA: 0x000D8A10
// ADDRESS: 004d8a10
// PROTOTYPE: CBaseObject * __thiscall Remove(CGUID * param_1, void * param_2)
//
// IMPLEMENTED выше; доказанные no-op listener callbacks не материализуются.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@004db129
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp
// RVA: 0x000DB129
// ADDRESS: 004db129
// PROTOTYPE: undefined Catch@004db129()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@00535750
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp
// RVA: 0x00135750
// ADDRESS: 00535750
// PROTOTYPE: undefined Unwind@00535750()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@005357b0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp
// RVA: 0x001357B0
// ADDRESS: 005357b0
// PROTOTYPE: undefined Unwind@005357b0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@005357d0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp
// RVA: 0x001357D0
// ADDRESS: 005357d0
// PROTOTYPE: undefined Unwind@005357d0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@005357db
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp
// RVA: 0x001357DB
// ADDRESS: 005357db
// PROTOTYPE: undefined Unwind@005357db()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@005357e6
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cjifen.cpp
// RVA: 0x001357E6
// ADDRESS: 005357e6
// PROTOTYPE: undefined Unwind@005357e6()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
