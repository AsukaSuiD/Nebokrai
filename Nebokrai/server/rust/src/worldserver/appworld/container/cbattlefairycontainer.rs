//! Владелец battle-fairy-container исторического `WorldServer`.
//!
//! Статус constructor/destructor RVA `0x000D77F0/0x000D7810`, собственных
//! `Serialize/Unserialize` RVA `0x000D7860/0x000D7870` и folded
//! `Clear/Release` RVA `0x000D7AA0/0x000D7AB0`,
//! `Add/Add(position)/Find/Remove/AddFromDB` RVA
//! `0x000D7AC0/0x000D7830/0x000D7840/0x000D7850/0x000D78A0` —
//! `IMPLEMENTED`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbattlefairycontainer.cpp:27,31,36,41,99,105`.
//!
//! Exact PDB задаёт размер `0x74` и единственный base
//! `CVolumeLimitGoodsContainer` по `+0x0`; собственных data-полей нет.
//! Constructor создаёт base с volume `0`, destructor только уничтожает base.
//! Rust хранит отдельный nominal owner, а обычные ownership/`Drop` заменяют
//! vtable, secondary listener, STL и EH cleanup без копирования x86 layout.
//!
//! Собственные codec-функции являются точными тонкими вызовами volume-codec-а:
//! wire, `include_child`, bool-result, ранний `Clear`, cursor, безопасное
//! уничтожение rejected goods и typed `BLOCKED_MISSING_FACT`-границы не меняются.
//! `Clear` всегда передаёт base literal `nullptr`, а `Release` прямо делегирует
//! base. Короткий source поэтому сохраняет очищенный container, прежний volume,
//! cursor и уже добавленные records по контракту volume-owner-а.
//!
//! Volume `0x11` задаёт будущий `CPlayer::DecordFromByteArray` отдельно между
//! virtual `Release` и decoder-ом; constructor не получает его заранее.
//! Совпадающие с `CFairyContainer` folded `Clear/Release` и игровые
//! `Add/Find/Remove` не означают объединения двух nominal классов. Все они
//! остаются thin volume-tail-calls без дополнительной type-policy.
//! `AddFromDB` выполняет derived collision lookup, затем base повторяет его и
//! делает direct insert; различие battle-owner-а ограничено string-table
//! `ZHGS0044` для технического `debug-DB` log, не меняющего state/return.

use super::super::goods::cgoodsfactory::GoodsBasePropertiesRegistry;
use super::cvolumelimitgoodscontainer::{CVolumeLimitGoodsContainer, VolumeContainerCodecError};
use crate::dbaccess::worlddb::goodslistener::TraversedGoods;
use crate::public::guid::CGuid;
use crate::worldserver::appworld::goods::cgoods::CGoods;

/// Достигнутое состояние исходного `CBattleFairyContainer` без собственного payload.
pub(crate) struct CBattleFairyContainer {
    volume_state: CVolumeLimitGoodsContainer,
}

impl CBattleFairyContainer {
    /// Создаёт точный base-state с нулевым volume.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            volume_state: CVolumeLimitGoodsContainer::with_constructor_defaults(),
        }
    }

    /// Сбрасывает owner и задаёт точное unsigned число cells.
    pub(crate) fn set_container_volume(&mut self, size: u32) {
        self.volume_state.set_container_volume(size);
    }

    /// Очищает товары и заново создаёт cells, сохраняя volume.
    pub(crate) fn clear(&mut self) {
        self.volume_state.clear();
    }

    /// Сбрасывает inherited owner, товары, volume и cells.
    pub(crate) fn release(&mut self) {
        self.volume_state.release();
    }

    /// Делегирует folded automatic volume Add.
    pub(crate) fn add(
        &mut self,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        self.volume_state.add(goods, registry)
    }

    /// Делегирует folded positional volume Add.
    pub(crate) fn add_at(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        self.volume_state.add_at(position, goods, registry)
    }

    /// Делегирует folded locked-aware GUID lookup.
    pub(crate) fn find(&self, ex_id: &CGuid) -> Option<&CGoods> {
        self.volume_state.find(ex_id)
    }

    /// Возвращает mutable DB-view exact battle-fairy cell-а.
    pub(crate) fn get_goods_mut(&mut self, position: u32) -> Option<&mut CGoods> {
        self.volume_state.get_goods_mut(position)
    }

    /// Делегирует folded volume removal с exact post-remove checks.
    pub(crate) fn remove(
        &mut self,
        ex_id: &CGuid,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        self.volume_state.remove(ex_id, registry)
    }

    /// Выполняет derived collision check до повторной base DB-проверки.
    pub(crate) fn add_from_db(
        &mut self,
        position: u32,
        goods: Box<CGoods>,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Option<Box<CGoods>>, VolumeContainerCodecError> {
        if self.volume_state.get_goods(position).is_some() {
            return Ok(Some(goods));
        }
        self.volume_state.add_from_db(position, goods, registry)
    }

    /// Замораживает inherited volume traversal без собственного payload.
    pub(crate) fn db_save_entries(
        &self,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<Vec<TraversedGoods>, super::super::goods::cgoods::GoodsDbSnapshotBlock> {
        self.volume_state.db_save_entries(registry)
    }

    /// Делегирует точный inherited volume-wire без собственного suffix-а.
    pub(crate) fn serialize(
        &self,
        destination: &mut Vec<u8>,
        include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        self.volume_state
            .serialize(destination, include_child, registry)
    }

    /// Делегирует volume-decoder с его ранним `Clear` и partial effects.
    pub(crate) fn unserialize(
        &mut self,
        source: &[u8],
        cursor: &mut usize,
        _include_child: bool,
        registry: &GoodsBasePropertiesRegistry,
    ) -> Result<bool, VolumeContainerCodecError> {
        self.volume_state.unserialize(source, cursor, registry)
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbattlefairycontainer.cpp

// ============================================================================
// FUNCTION: CBattleFairyContainer::CBattleFairyContainer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbattlefairycontainer.cpp:27
// RVA: 0x000D77F0
// ADDRESS: 004d77f0
// PROTOTYPE: undefined __thiscall CBattleFairyContainer(void)
//
// IMPLEMENTED выше; отдельный nominal owner содержит constructor-defaults base.

// ============================================================================
// FUNCTION: CBattleFairyContainer::~CBattleFairyContainer
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbattlefairycontainer.cpp:31
// RVA: 0x000D7810
// ADDRESS: 004d7810
// PROTOTYPE: void __thiscall ~CBattleFairyContainer(void)
//
// IMPLEMENTED обычным Rust ownership/Drop; vtable/EH cleanup удалён.

// ============================================================================
// FUNCTION: CBattleFairyContainer::Add
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbattlefairycontainer.cpp:52
// RVA: 0x000D7830
// ADDRESS: 004d7830
// PROTOTYPE: int __thiscall Add(ulong param_1, CGoods * param_2, void * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Find
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbattlefairycontainer.cpp:86
// RVA: 0x000D7840
// ADDRESS: 004d7840
// PROTOTYPE: CBaseObject * __thiscall Find(CGUID * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Remove
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbattlefairycontainer.cpp:92
// RVA: 0x000D7850
// ADDRESS: 004d7850
// PROTOTYPE: CBaseObject * __thiscall Remove(CGUID * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Serialize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbattlefairycontainer.cpp:99
// RVA: 0x000D7860
// ADDRESS: 004d7860
// PROTOTYPE: int __thiscall Serialize(vector<unsigned_char,std::allocator<unsigned_char>_> * param_1, int param_2)
//
// IMPLEMENTED выше точным делегированием inherited volume-codec-у.

// ============================================================================
// FUNCTION: CBattleFairyContainer::Unserialize
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbattlefairycontainer.cpp:105
// RVA: 0x000D7870
// ADDRESS: 004d7870
// PROTOTYPE: int __thiscall Unserialize(uchar * param_1, long * param_2, int param_3)
//
// IMPLEMENTED выше точным делегированием inherited volume-decoder-у.

// ============================================================================
// FUNCTION: CBattleFairyContainer::AddFromDB
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbattlefairycontainer.cpp:59
// RVA: 0x000D78A0
// ADDRESS: 004d78a0
// PROTOTYPE: int __thiscall AddFromDB(CGoods * param_1, ulong param_2)
//
// IMPLEMENTED выше; exact collision-first и base-result сохранены, string-table
// формат используется только историческим debug-file sink.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBattleFairyContainer::Clear
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbattlefairycontainer.cpp:36
// RVA: 0x000D7AA0
// ADDRESS: 004d7aa0
// PROTOTYPE: void __thiscall Clear(void * param_1)
//
// IMPLEMENTED выше; caller-param игнорируется, base получает literal `nullptr`.

// ============================================================================
// FUNCTION: CBattleFairyContainer::Release
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbattlefairycontainer.cpp:41
// RVA: 0x000D7AB0
// ADDRESS: 004d7ab0
// PROTOTYPE: void __thiscall Release(void)
//
// IMPLEMENTED выше точным делегированием base `Release`.

// ============================================================================
// FUNCTION: CBattleFairyContainer::Add
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\container\cbattlefairycontainer.cpp:46
// RVA: 0x000D7AC0
// ADDRESS: 004d7ac0
// PROTOTYPE: int __thiscall Add(CBaseObject * param_1, void * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
