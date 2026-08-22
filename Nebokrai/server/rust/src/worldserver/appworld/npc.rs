//! Владелец NPC исторического `WorldServer`.
//!
//! Статус base-подобъекта, type-default и lifecycle собственного списка внутри
//! `CNpc::CNpc/~CNpc` RVA `0x000E04F0/0x000E0560` — `IMPLEMENTED`; остальной
//! корпус ниже остаётся `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\npc.h` и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\npc.cpp:11`.
//!
//! Exact PDB задаёт размеры старых `CMoveShape/CNpc` `0x80/0x8C`. Конструктор
//! по `0x004E050E` первым передаёт неизменённый `this` в
//! `CMoveShape::CMoveShape`, создаёт собственный `m_listScript` с offset
//! `+0x80`, а по `0x004E053C` выполняет `mov [esi+4], 0x1F4`. Последняя запись
//! является object type `500` в унаследованном `CBaseObject::m_lType`, а не
//! ошибочно подписанным `_padding_`. Destructor RVA `0x000E0560` сначала
//! очищает тот же список, затем вызывает `CMoveShape::~CMoveShape`.
//!
//! Rust-композиция материализует единственный достигнутый `CMoveShape`
//! base-подобъект, type-default и пустой non-owning `m_listScript`. Exact
//! корпус не записывает в этот список и не читает его элементов; их
//! decompiler-имя `tagEnemyFaction*` не выдаётся за подтверждённый Rust-тип.
//! `Drop` сначала очищает storage списка, затем автоматически освобождает
//! base-owner, как исходный destructor. AI, region и container-семантика
//! остаются в raw-блоках; Rust layout не является копией старого ABI.

use super::moveshape::CMoveShape;

/// Opaque non-owning entry точного `CNpc::m_listScript`.
///
/// В исследованном World-корпусе нет writer/reader этого списка, поэтому
/// хранится только host-address без выдуманного типа или ownership pointee.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct NpcScriptLink {
    address: usize,
}

/// Достигнутый owner `CNpc`.
pub(crate) struct CNpc {
    move_shape_base: CMoveShape,
    script_links: Vec<NpcScriptLink>,
}

impl CNpc {
    /// Создаёт `CMoveShape`, пустой `m_listScript` и object type `500`.
    pub(crate) const fn with_constructor_base_and_type() -> Self {
        let mut move_shape_base = CMoveShape::with_constructor_shape_base();
        move_shape_base.set_type(500);
        Self {
            move_shape_base,
            script_links: Vec::new(),
        }
    }

    /// Возвращает унаследованный object type без дополнительных эффектов.
    pub(crate) const fn get_type(&self) -> i32 {
        self.move_shape_base.get_type()
    }

    /// Возвращает унаследованный signed object ID.
    pub(crate) const fn get_id(&self) -> i32 {
        self.move_shape_base.get_id()
    }

    /// Присваивает унаследованный signed object ID.
    pub(crate) const fn set_id(&mut self, id: i32) {
        self.move_shape_base.set_id(id);
    }

    /// Присваивает унаследованное byte-exact имя до первого NUL.
    pub(crate) fn set_name(&mut self, name: &[u8]) {
        self.move_shape_base.set_name(name);
    }

    /// Заимствует унаследованное byte-exact имя без завершающего NUL.
    pub(crate) fn get_name(&self) -> &[u8] {
        self.move_shape_base.get_name()
    }

    /// Присваивает унаследованный signed graphics ID.
    pub(crate) const fn set_graphics_id(&mut self, graphics_id: i32) {
        self.move_shape_base.set_graphics_id(graphics_id);
    }
}

impl Drop for CNpc {
    fn drop(&mut self) {
        // Exact `~CNpc` сначала вызывает `_Tidy(m_listScript)`, и лишь затем
        // `CMoveShape::~CMoveShape`. Pointee не owned старым list, поэтому
        // очистка Rust storage не освобождает неизвестные external objects.
        self.script_links.clear();
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\npc.cpp

// ============================================================================
// FUNCTION: tagBaseProperty::~tagBaseProperty
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\npc.cpp
// RVA: 0x0005B740
// ADDRESS: 0045b740
// PROTOTYPE: void __thiscall ~tagBaseProperty(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CPlayer::tagCarriageInfo::~tagCarriageInfo
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\npc.cpp
// RVA: 0x0005B7B0
// ADDRESS: 0045b7b0
// PROTOTYPE: void __thiscall ~tagCarriageInfo(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0045d168
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\npc.cpp
// RVA: 0x0005D168
// ADDRESS: 0045d168
// PROTOTYPE: undefined Catch@0045d168()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0045d674
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\npc.cpp
// RVA: 0x0005D674
// ADDRESS: 0045d674
// PROTOTYPE: undefined Catch@0045d674()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0045d989
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\npc.cpp
// RVA: 0x0005D989
// ADDRESS: 0045d989
// PROTOTYPE: undefined Catch@0045d989()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0045e1dd
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\npc.cpp
// RVA: 0x0005E1DD
// ADDRESS: 0045e1dd
// PROTOTYPE: undefined Catch@0045e1dd()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Catch@0045e290
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\npc.cpp
// RVA: 0x0005E290
// ADDRESS: 0045e290
// PROTOTYPE: undefined Catch@0045e290()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNpc::CNpc
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\npc.cpp:11
// RVA: 0x000E04F0
// ADDRESS: 004e04f0
// PROTOTYPE: undefined __thiscall CNpc(void)
//
// IMPLEMENTED_OWNER: `CNpc::with_constructor_base_and_type` выше создаёт
// единственный достигнутый `CMoveShape`, пустой non-owning `m_listScript` и
// затем назначает signed object type `500`. Тип pointee из raw не подтверждён
// отдельным caller-ом, поэтому Rust хранит opaque address без ownership.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNpc::~CNpc
// STATUS: IMPLEMENTED / API_SHAPE_REPLACED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\npc.cpp:16
// RVA: 0x000E0560
// ADDRESS: 004e0560
// PROTOTYPE: void __thiscall ~CNpc(void)
//
// Реализовано `Drop for CNpc`: сначала очищается non-owning list storage,
// затем Rust освобождает `CMoveShape` field. MSVC list allocation, vtable и
// освобождение list-nodes не являются наблюдаемой игровой семантикой.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//






// ============================================================================
// FUNCTION: Unwind@0052f3b0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\npc.cpp
// RVA: 0x0012F3B0
// ADDRESS: 0052f3b0
// PROTOTYPE: undefined Unwind@0052f3b0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052f3f0
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\npc.cpp
// RVA: 0x0012F3F0
// ADDRESS: 0052f3f0
// PROTOTYPE: undefined Unwind@0052f3f0()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: Unwind@0052f410
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\npc.cpp
// RVA: 0x0012F410
// ADDRESS: 0052f410
// PROTOTYPE: undefined Unwind@0052f410()
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//































































// COMPONENT_VARIANT_END: WorldServer
