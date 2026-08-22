//! Номинальный minister owner исторического `WorldServer`.
//!
//! Статус constructor/destructor `CMinister` RVA `0x000DFD80/0x000DFDA0`:
//! `IMPLEMENTED`. Constructor создаёт country identity и обнуляет те же четыре
//! officer bytes; destructor не добавляет наблюдаемого эффекта поверх
//! `COfficer`. Точная пара:
//! `WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb`, SHA-256 EXE
//! `F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1`, PDB
//! `04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4`.
//! Исходный owner PDB:
//! `e:\\svn\\fengyun_russia_dev\\server\\worldserver\\appworld\\country\\minister.cpp:5,9`.
//!
//! Rust сохраняет отдельный nominal type и composition вместо C++ vtable.

use super::officer::COfficer;

/// Номинальный minister с полным достигнутым officer-prefix.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CMinister {
    officer: COfficer,
}

impl CMinister {
    /// Повторяет exact нулевой constructor-state minister-а.
    pub(crate) const fn with_constructor_defaults() -> Self {
        Self {
            officer: COfficer::with_constructor_defaults(),
        }
    }

    pub(crate) const fn officer(&self) -> &COfficer {
        &self.officer
    }

    pub(crate) fn officer_mut(&mut self) -> &mut COfficer {
        &mut self.officer
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\minister.cpp

// ============================================================================
// FUNCTION: CMinister::CMinister
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\minister.cpp:5
// RVA: 0x000DFD80
// ADDRESS: 004dfd80
// PROTOTYPE: undefined __thiscall CMinister(void)
//
// IMPLEMENTED_OWNER: `CMinister::with_constructor_defaults` выше.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMinister::~CMinister
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\minister.cpp:9
// RVA: 0x000DFDA0
// ADDRESS: 004dfda0
// PROTOTYPE: void __thiscall ~CMinister(void)
//
// IMPLEMENTED_OWNER: обычный `Drop` композиции заменяет destructor-chain.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: WorldServer
