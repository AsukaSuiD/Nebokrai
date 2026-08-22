//! Владелец yuanbao-container исторического `WorldServer`.
//!
//! Статус constructor/destructor-state RVA `0x000D86F0/0x000D8750` и folded
//! `Clear/Release/IsFull/GetGoods/GetGoodsAmount/Serialize/Unserialize` RVA
//! `0x000D8670/0x000D5E30/0x000D5E50/0x000D5EC0/0x000D5EE0/0x000D5EF0/`
//! `0x000D6030` и yuanbao-специфичные query/add-family RVA
//! `0x000D86B0/0x000D86D0/0x000D8810/0x000D8970/0x000D8A80/0x000D8AC0`
//! — `IMPLEMENTED`; остальной корпус ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная
//! пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cyuanbao.cpp:18,34,41,55,82,209,222,245`.
//!
//! Exact PDB задаёт отдельный класс размера `0x28`: `CGoodsContainer +0x0`,
//! `CContainerListener +0x20`, единственный `m_pGoldCoins: CGoods* +0x24`.
//! Публичные container-операции `CYuanBao` указывают на те же RVA, что
//! `CWallet`; это linker folding, а не наследование. Rust поэтому сохраняет
//! отдельный nominal owner, но делегирует его доказанный однослотовый state и
//! wire-кодек готовому compatibility-layer. `Option<Box<CGoods>>` и обычный
//! `Drop` заменяют nullable pointer, `GarbageCollect`, deleting-destructor и EH
//! cleanup. Оба listener callback-а имеют общий no-op RVA `0x000DBD10`.
//!
//! Wire остаётся marker-байтом `0/1` с возможным полным `CGoods`; decoder
//! сначала сбрасывает owner и прежний slot. Короткий source выражает
//! безразмерное legacy-чтение локальным typed `BLOCKED_MISSING_FACT`.
//! YuanBao-index разрешается process-global фабрикой через StringTable ID
//! `WS0109`; Rust принимает уже разрешённое значение и не переносит `CGame`.
//! Пустой slot exact принимает товар без currency-validation, а занятый
//! разрешает stacking только для YuanBao. Бесполезный legacy
//! `GetGoods(index, vector-by-value)` исправлен возвращаемым iterator-ом с тем
//! же YuanBao-фильтром.

use super::super::goods::cgoods::{CGoods, GoodsCodecError};
use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use super::super::listener::ccontainerlistener::CContainerListener;
use super::cwallet::CWallet;
use crate::dbaccess::worlddb::goodslistener::TraversedGoods;
use crate::public::guid::CGuid;

/// Отдельный достигнутый owner исходного `CYuanBao`, не копия его 32-битного ABI.
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

    /// Делегирует object-перегрузку exact позиции `0`.
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

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cyuanbao.cpp

// ============================================================================
// FUNCTION: CYuanBao::IsGoodsExisted
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cyuanbao.cpp:209
// RVA: 0x000D86B0
// ADDRESS: 004d86b0
// PROTOTYPE: int __thiscall IsGoodsExisted(ulong param_1)
//
// IMPLEMENTED выше; resolved YuanBao-index передаётся явно.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYuanBao::GetTheFirstGoods
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cyuanbao.cpp:222
// RVA: 0x000D86D0
// ADDRESS: 004d86d0
// PROTOTYPE: CGoods * __thiscall GetTheFirstGoods(ulong param_1)
//
// IMPLEMENTED выше; пустой slot остаётся `None`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYuanBao::CYuanBao
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cyuanbao.cpp:18
// RVA: 0x000D86F0
// ADDRESS: 004d86f0
// PROTOTYPE: undefined __thiscall CYuanBao(void)
//
// IMPLEMENTED выше; owner `0/0`, slot пуст, listener не хранит Rust-state.
//

// ============================================================================
// FUNCTION: CYuanBao::~CYuanBao
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cyuanbao.cpp:34
// RVA: 0x000D8750
// ADDRESS: 004d8750
// PROTOTYPE: void __thiscall ~CYuanBao(void)
//
// IMPLEMENTED через owned `Option<Box<CGoods>>` и обычный Rust `Drop`.
//

// ============================================================================
// FUNCTION: CYuanBao::AddFromDB
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cyuanbao.cpp:82
// RVA: 0x000D8810
// ADDRESS: 004d8810
// PROTOTYPE: int __thiscall AddFromDB(CGoods * param_1, ulong param_2)
//
// IMPLEMENTED выше через single-slot DB helper; debug logging исключён как
// технический side effect.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYuanBao::Add
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cyuanbao.cpp:55
// RVA: 0x000D8970
// ADDRESS: 004d8970
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, void * param_3)
//
// IMPLEMENTED выше с resolved YuanBao-index и exact stacking helper-ом.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYuanBao::Add
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cyuanbao.cpp:41
// RVA: 0x000D8A80
// ADDRESS: 004d8a80
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, void * param_2)
//
// IMPLEMENTED выше как typed `CGoods` API с позицией `0`.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYuanBao::GetGoods
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cyuanbao.cpp:245
// RVA: 0x000D8AC0
// ADDRESS: 004d8ac0
// PROTOTYPE: void __thiscall GetGoods(ulong param_1, vector<CGoods*,std::allocator<CGoods*>_> param_2)
//
// IMPLEMENTED выше как возвращаемый iterator; внутренний vector-by-value defect
// исправлен без изменения YuanBao-фильтра.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
