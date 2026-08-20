//! Владелец bank-container исторического `WorldServer`.
//!
//! Статус constructor/destructor-state RVA `0x000D7FE0/0x000D8060`,
//! `Release/Clear` RVA `0x000D8040/0x000D8050` и унаследованного wallet-codec
//! RVA `0x000D5EF0/0x000D6030` — `IMPLEMENTED`; lock-gated игровые операции
//! `Find/Remove/Add/AddFromDB` ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbank.cpp:16,29,150,161`.
//!
//! Exact PDB задаёт класс размером `0x2C`: base `CWallet` по `+0x0` и
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
//! затем возвращает локальную typed `BLOCKED_MISSING_FACT` ошибку.
//!
//! `CWallet` служит узким compatibility-layer для доказанных base-state и
//! codec-а. `Option<Box<CGoods>>` заменяет nullable pointer и
//! `GarbageCollect`, а встроенный listener остаётся ранее доказанным no-op RVA
//! `0x000DBD10`. Не реализованные `Lock/Unlock` и lock-gated игровые пути не
//! нужны для clone-последовательности и не получают поведения по имени метода.

use super::super::goods::cgoods::{CGoods, GoodsCodecError};
use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use super::cwallet::CWallet;
use crate::dbaccess::worlddb::goodslistener::TraversedGoods;

/// Достигнутое состояние исходного `CBank`, не копия его 32-битного ABI.
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

    /// Проверяет достижение exact max-stack единственного товара.
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

    /// Возвращает число занятых bank-slot-ов: ноль либо один.
    pub(crate) const fn get_goods_amount(&self) -> u32 {
        self.wallet_state.get_goods_amount()
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

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbank.cpp

// ============================================================================
// FUNCTION: CBank::CBank
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbank.cpp:16
// RVA: 0x000D7FE0
// ADDRESS: 004d7fe0
// PROTOTYPE: undefined __thiscall CBank(void)
//
// IMPLEMENTED выше; base wallet-state создаётся раньше unlocked-флага.

// ============================================================================
// FUNCTION: CBank::Find
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbank.cpp:36
// RVA: 0x000D8000
// ADDRESS: 004d8000
// PROTOTYPE: CBaseObject * __thiscall Find(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBank::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbank.cpp:48
// RVA: 0x000D8010
// ADDRESS: 004d8010
// PROTOTYPE: CBaseObject * __thiscall Remove(CGUID * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBank::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbank.cpp:62
// RVA: 0x000D8020
// ADDRESS: 004d8020
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBank::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbank.cpp:75
// RVA: 0x000D8030
// ADDRESS: 004d8030
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBank::Release
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbank.cpp:150
// RVA: 0x000D8040
// ADDRESS: 004d8040
// PROTOTYPE: void __thiscall Release(void)
//
// IMPLEMENTED выше; lock снимается раньше base `Release`.

// ============================================================================
// FUNCTION: CBank::Clear
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbank.cpp:161
// RVA: 0x000D8050
// ADDRESS: 004d8050
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// IMPLEMENTED выше; opaque listener-аргумент не меняет доказанное state.

// ============================================================================
// FUNCTION: CBank::~CBank
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbank.cpp:29
// RVA: 0x000D8060
// ADDRESS: 004d8060
// PROTOTYPE: void __thiscall ~CBank(void)
//
// IMPLEMENTED обычным Rust ownership/Drop; vtable/EH cleanup удалён.

// ============================================================================
// FUNCTION: CBank::AddFromDB
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbank.cpp:86
// RVA: 0x000D80E0
// ADDRESS: 004d80e0
// PROTOTYPE: int __thiscall AddFromDB(CGoods * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
