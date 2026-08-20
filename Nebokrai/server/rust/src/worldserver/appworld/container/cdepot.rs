//! Владелец depot-container исторического `WorldServer`.
//!
//! Статус constructor/destructor-state RVA `0x000D7D70/0x000D7E00`,
//! `Clear/Release` RVA `0x000D7D90/0x000D7DB0` и унаследованного volume-codec
//! RVA `0x000DA7D0/0x000D8DA0` — `IMPLEMENTED`; lock-gated игровые операции
//! `Add/Find/Remove/AddFromDB` ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cdepot.cpp:17,30,37,46`.
//!
//! Exact PDB задаёт класс размером `0x78`: base
//! `CVolumeLimitGoodsContainer` по `+0x0` и единственное собственное поле
//! `bool m_bIsLocked` по `+0x74`. Constructor создаёт base с volume `0` и
//! unlocked-флагом. `Clear` и `Release` сначала снимают lock, затем вызывают
//! различающиеся base-операции. Destructor повторяет `Release`; Rust ownership
//! и обычный `Drop` заменяют deleting-destructor, vtable/EH и STL cleanup.
//!
//! Собственных `CDepot::Serialize/Unserialize` в PDB нет. Wire наследуется от
//! volume-owner-а: unsigned count, затем cell index и полный `CGoods` для
//! каждой записи. Virtual `Clear` внутри decoder-а должен видеть override
//! `CDepot`, поэтому Rust снимает lock перед делегированием готовому base-
//! decoder-у. Lock в wire не входит; короткий source оставляет depot
//! разблокированным и очищенным, сохраняя cursor и уже добавленные records до
//! локальной typed `BLOCKED_MISSING_FACT` ошибки.
//!
//! Volume `0xA1` не является constructor-состоянием `CDepot`. Его задаёт
//! точный `CPlayer::DecordFromByteArray` после отдельного virtual `Release` и
//! до унаследованного decoder-а; будущая композиция player-codec обязана
//! сохранить этот порядок. `Lock/Unlock`, `IsExtentionItemPos`,
//! `IsActivedOrOldPos` и lock-gated игровые пути не требуются clone-границе и
//! не получают реализации только по именам PDB.

use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use super::cvolumelimitgoodscontainer::{CVolumeLimitGoodsContainer, VolumeContainerCodecError};
use crate::dbaccess::worlddb::goodslistener::TraversedGoods;

/// Достигнутое состояние исходного `CDepot`, не копия его 32-битного ABI.
pub(crate) struct CDepot {
    volume_state: CVolumeLimitGoodsContainer,
    locked: bool,
}

impl CDepot {
    /// Создаёт unlocked depot с нулевым volume и inherited owner `0/0`.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            volume_state: CVolumeLimitGoodsContainer::with_constructor_defaults(),
            locked: false,
        }
    }

    /// Снимает lock, сбрасывает base-state и задаёт точное unsigned число cells.
    pub(crate) fn set_container_volume(&mut self, size: u32) {
        self.locked = false;
        self.volume_state.set_container_volume(size);
    }

    /// Снимает lock и очищает товары, сохраняя текущий volume.
    pub(crate) fn clear(&mut self) {
        self.locked = false;
        self.volume_state.clear();
    }

    /// Снимает lock и сбрасывает inherited owner, товары и volume.
    pub(crate) fn release(&mut self) {
        self.locked = false;
        self.volume_state.release();
    }

    /// Проверяет точные amount/cell границы унаследованного container-а.
    pub(crate) fn is_full(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        self.volume_state.is_full(registry)
    }

    /// Возвращает число валидных товаров, связанных с cells.
    pub(crate) fn get_goods_amount(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<u32, VolumeContainerCodecError> {
        self.volume_state.get_goods_amount(registry)
    }

    /// Замораживает inherited volume traversal без изменения lock-state.
    pub(crate) fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, super::super::goods::cgoods::GoodsDbSnapshotBlock> {
        self.volume_state.db_save_entries(registry)
    }

    /// Кодирует унаследованный volume-wire без сериализации lock-а.
    pub(crate) fn serialize(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        self.volume_state
            .serialize(destination, include_child, registry)
    }

    /// Снимает lock и декодирует records после virtual `Clear` base-codec-а.
    pub(crate) fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        self.locked = false;
        self.volume_state.unserialize(source, cursor, registry)
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cdepot.cpp

// ============================================================================
// FUNCTION: CDepot::CDepot
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cdepot.cpp:17
// RVA: 0x000D7D70
// ADDRESS: 004d7d70
// PROTOTYPE: undefined __thiscall CDepot(void)
//
// IMPLEMENTED выше; base volume равен нулю, lock снят.

// ============================================================================
// FUNCTION: CDepot::Clear
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cdepot.cpp:37
// RVA: 0x000D7D90
// ADDRESS: 004d7d90
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// IMPLEMENTED выше; входной listener-аргумент исходно заменяется `nullptr`.

// ============================================================================
// FUNCTION: CDepot::Release
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cdepot.cpp:46
// RVA: 0x000D7DB0
// ADDRESS: 004d7db0
// PROTOTYPE: void __thiscall Release(void)
//
// IMPLEMENTED выше; lock снимается раньше base `Release`.

// ============================================================================
// FUNCTION: CDepot::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cdepot.cpp:55
// RVA: 0x000D7DC0
// ADDRESS: 004d7dc0
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.

// ============================================================================
// FUNCTION: CDepot::Add
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cdepot.cpp:68
// RVA: 0x000D7DD0
// ADDRESS: 004d7dd0
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.

// ============================================================================
// FUNCTION: CDepot::Find
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cdepot.cpp:145
// RVA: 0x000D7DE0
// ADDRESS: 004d7de0
// PROTOTYPE: CBaseObject * __thiscall Find(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.

// ============================================================================
// FUNCTION: CDepot::Remove
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cdepot.cpp:157
// RVA: 0x000D7DF0
// ADDRESS: 004d7df0
// PROTOTYPE: CBaseObject * __thiscall Remove(CGUID * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.

// ============================================================================
// FUNCTION: CDepot::~CDepot
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cdepot.cpp:30
// RVA: 0x000D7E00
// ADDRESS: 004d7e00
// PROTOTYPE: void __thiscall ~CDepot(void)
//
// IMPLEMENTED обычным Rust ownership/Drop; vtable/EH cleanup удалён.

// ============================================================================
// FUNCTION: CDepot::AddFromDB
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cdepot.cpp:81
// RVA: 0x000D7E80
// ADDRESS: 004d7e80
// PROTOTYPE: int __thiscall AddFromDB(CGoods * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
