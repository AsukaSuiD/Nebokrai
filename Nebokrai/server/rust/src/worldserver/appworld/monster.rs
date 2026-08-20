//! Владелец монстра исторического `WorldServer`.
//!
//! Статус base-подобъекта и type-default внутри `CMonster::CMonster` RVA
//! `0x000E0490`, а также непосредственной destructor-цепочки RVA `0x000E0410`
//! — `VERIFIED_DISASSEMBLY`; property и остальной корпус ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходные владельцы PDB:
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\monster.h` и
//! `e:\svn\fengyun_russia_dev\server\worldserver\appworld\monster.cpp:11-16`.
//!
//! Exact PDB задаёт размеры старых `CMoveShape/CMonster` `0x80/0xB8`.
//! Constructor по `0x004E0493` передаёт неизменённый `this` в
//! `CMoveShape::CMoveShape`, инициализирует собственную `strOrginName` в
//! диапазоне `+0x84..+0x98`, а по `0x004E04B6` выполняет
//! `mov [esi+4], 0x258`. Последняя запись является object type `600` в
//! унаследованном `CBaseObject::m_lType`, а не ошибочно подписанным
//! `_padding_`.
//!
//! Raw destructor ошибочно показывал ранний возврат после освобождения
//! heap-строки. Exact EXE `0x004E0419..0x004E044C` подтверждает, что обе формы
//! строки сходятся на сбросе её состояния и затем tail-jump вызывают
//! `CMoveShape::~CMoveShape`. Rust-композиция материализует только единственный
//! достигнутый base-подобъект и type-default. Helper не называется `new`:
//! `m_Property`, строка, GetFigure и AI/region/container-семантика остаются
//! raw, а их destructor не подменяется пустым `Drop`. Rust layout не
//! объявляется копией старого ABI.

use super::moveshape::CMoveShape;

/// Достигнутая base-часть исходного `CMonster`.
pub(crate) struct CMonster {
    move_shape_base: CMoveShape,
}

impl CMonster {
    /// Создаёт только доказанный base-подобъект с object type `600`.
    pub(crate) const fn with_constructor_base_and_type() -> Self {
        let mut move_shape_base = CMoveShape::with_constructor_shape_base();
        move_shape_base.set_type(600);
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
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\monster.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\monster.h

// ============================================================================
// FUNCTION: CMonster::~CMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\monster.cpp:16
// RVA: 0x000E0410
// ADDRESS: 004e0410
// PROTOTYPE: void __thiscall ~CMonster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::GetFigure
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\monster.h:36
// RVA: 0x000E0460
// ADDRESS: 004e0460
// PROTOTYPE: uchar __thiscall GetFigure(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonster::CMonster
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\monster.cpp:11
// RVA: 0x000E0490
// ADDRESS: 004e0490
// PROTOTYPE: undefined __thiscall CMonster(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
