//! Узкий process-global owner суточного номера копии ShengSiShiSu.
//!
//! `GetCopyNum` RVA `0x000A0DC0` и `AddCopyNum` RVA `0x000A0DD0` имеют статус
//! `IMPLEMENTED`; timer-owned reset/registration ниже остаются
//! `UNKNOWN` (исследовательский декомпилят хранится локально). Exact `GetCopyNum` читает signed DWORD по VA `0x0056B5E8`,
//! а PE `.data` содержит initial bytes `01 00 00 00`. `AddCopyNum` выполняет
//! обычное 32-битное сложение с единицей без overflow gate; `AtomicI32`
//! заменяет только возможную межпоточную data race и сохраняет wrapping bits.
//! Декомпилятор: Ghidra 12.1.2; точная пара указана у raw provenance ниже.
//! Полный декомпилят хранится локально и не входит в распространяемый код.
//!
//! Поздний Rust-донор верно определил начальное значение и назначение owner-а,
//! но его fail-closed overflow был новым поведением и здесь не перенесён.

use std::sync::atomic::{AtomicI32, Ordering};

static COPY_NUMBER: AtomicI32 = AtomicI32::new(1);

/// Возвращает текущий signed номер без изменения состояния.
pub(crate) fn get_copy_num() -> i32 {
    COPY_NUMBER.load(Ordering::Relaxed)
}

/// Увеличивает номер с exact 32-битным wrapping оригинала.
pub(crate) fn add_copy_num() -> i32 {
    COPY_NUMBER.fetch_add(1, Ordering::Relaxed).wrapping_add(1)
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\misc.cpp

// ============================================================================
// FUNCTION: GetCopyNum
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\misc.cpp:9
// RVA: 0x000A0DC0
// ADDRESS: 004a0dc0
// PROTOTYPE: int __cdecl GetCopyNum(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: AddCopyNum
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\misc.cpp:14
// RVA: 0x000A0DD0
// ADDRESS: 004a0dd0
// PROTOTYPE: void __cdecl AddCopyNum(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: ClearCopyNum
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\misc.cpp:19
// RVA: 0x000A0DE0
// ADDRESS: 004a0de0
// PROTOTYPE: void __stdcall ClearCopyNum(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: RegisterClearShengSiShiSuCopyNumTime
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\misc.cpp:33
// RVA: 0x000A0E50
// ADDRESS: 004a0e50
// PROTOTYPE: void __cdecl RegisterClearShengSiShiSuCopyNumTime(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
