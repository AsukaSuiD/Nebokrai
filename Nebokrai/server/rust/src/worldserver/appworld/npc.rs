//! Владелец NPC исторического `WorldServer`.
//!
//! Статус base-подобъекта и type-default внутри `CNpc::CNpc` RVA
//! `0x000E04F0` — `VERIFIED_DISASSEMBLY`; собственный список и остальной
//! корпус ниже остаются `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
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
//! Rust-композиция материализует только единственный достигнутый
//! `CMoveShape` base-подобъект и type-default. Helper не называется `new` и не
//! выдаётся за полный constructor: `m_listScript`, его элементы, AI, region и
//! container-семантика остаются в raw-блоках. Rust layout не объявляется
//! копией старого ABI, а destructor списка не подменяется пустым `Drop`.

use super::moveshape::CMoveShape;

/// Достигнутая base-часть исходного `CNpc`.
pub(crate) struct CNpc {
    move_shape_base: CMoveShape,
}

impl CNpc {
    /// Создаёт только доказанный base-подобъект с object type `500`.
    pub(crate) const fn with_constructor_base_and_type() -> Self {
        let mut move_shape_base = CMoveShape::with_constructor_shape_base();
        move_shape_base.set_type(500);
        Self { move_shape_base }
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
// единственный достигнутый `CMoveShape` base-подобъект и затем назначает
// signed object type `500`. Пустой `m_listScript` не материализован отдельно:
// его элементы `tagEnemyFaction*` и их жизненный цикл ещё не восстановлены;
// Rust не выдаёт отсутствие такого storage за полный constructor.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CNpc::~CNpc
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\npc.cpp:16
// RVA: 0x000E0560
// ADDRESS: 004e0560
// PROTOTYPE: void __thiscall ~CNpc(void)
//
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
