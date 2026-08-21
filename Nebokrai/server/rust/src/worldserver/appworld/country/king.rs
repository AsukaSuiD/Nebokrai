//! Три setter-а king points RVA `0x000A46C0/0x000A46F0/0x000A4720` и
//! `ChangeControlPoint` RVA `0x000DFD30` — `IMPLEMENTED`;
//! constructor/destructor ниже остаются техническими RAW-фрагментами.
//! Exact EXE подтверждает только upper clamp: отрицательные значения не
//! исправляются. Rust применяет тот же контракт к достигнутому king-state без
//! воспроизведения C++ inheritance/layout.
//! `ChangeControlPoint` exact `0x004DFD30..0x004DFD5A` сначала делает wrapping
//! signed add, затем сравнивает результат с `_max_king_control_point` и только
//! превышение заменяет максимумом. Достигнутые country call-sites передают
//! отрицание стоимости через `wrapping_neg`, сохраняя x86 `neg/add` даже для
//! `INT_MIN`; parameter singleton заменён явной ссылкой на `CCountryParam`.

use crate::dbaccess::worlddb::dbcountry::CountryKingSaveSnapshot;
use crate::worldserver::appworld::country::countryparam::{
    CCountryParam, CountryParameterUnavailable,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum KingPointKind {
    Control,
    Material,
    War,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KingPointUpdate {
    pub(crate) kind: KingPointKind,
    pub(crate) requested: i32,
    pub(crate) previous: i32,
    pub(crate) applied: i32,
}

pub(crate) fn set_control_point(
    king: &mut CountryKingSaveSnapshot,
    requested: i32,
    parameters: &CCountryParam,
) -> Result<KingPointUpdate, CountryParameterUnavailable> {
    let maximum = parameters
        .max_king_control_point()
        .ok_or(CountryParameterUnavailable {
            field: "_max_king_control_point",
        })?;
    Ok(apply_point(
        &mut king.control_point,
        requested,
        maximum,
        KingPointKind::Control,
    ))
}

pub(crate) fn set_material_point(
    king: &mut CountryKingSaveSnapshot,
    requested: i32,
    parameters: &CCountryParam,
) -> Result<KingPointUpdate, CountryParameterUnavailable> {
    let maximum = parameters
        .max_king_material_point()
        .ok_or(CountryParameterUnavailable {
            field: "_max_king_material_point",
        })?;
    Ok(apply_point(
        &mut king.material_point,
        requested,
        maximum,
        KingPointKind::Material,
    ))
}

pub(crate) fn change_control_point(
    king: &mut CountryKingSaveSnapshot,
    delta: i32,
    parameters: &CCountryParam,
) -> Result<KingPointUpdate, CountryParameterUnavailable> {
    let requested = king.control_point.wrapping_add(delta);
    set_control_point(king, requested, parameters)
}

pub(crate) fn set_war_point(
    king: &mut CountryKingSaveSnapshot,
    requested: i32,
    parameters: &CCountryParam,
) -> Result<KingPointUpdate, CountryParameterUnavailable> {
    let maximum = parameters
        .max_king_war_point()
        .ok_or(CountryParameterUnavailable {
            field: "_max_king_war_point",
        })?;
    Ok(apply_point(
        &mut king.war_point,
        requested,
        maximum,
        KingPointKind::War,
    ))
}

fn apply_point(
    point: &mut i32,
    requested: i32,
    maximum: i32,
    kind: KingPointKind,
) -> KingPointUpdate {
    let previous = *point;
    let applied = requested.min(maximum);
    *point = applied;
    KingPointUpdate {
        kind,
        requested,
        previous,
        applied,
    }
}

// COMPONENT_VARIANT_BEGIN: WorldServer
// Точная пара: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SHA-256 EXE: F3AC454DAF83E7E9C8F844C725BE2C5A24EFA946C27D75319CFCB68A2F466EF1
// SHA-256 PDB: 04E2CC4CE1187A3AAB455566DDC39E72ED7568CAB0EDBD731B4F84629F6EF1E4
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\king.h
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\king.cpp

// ============================================================================
// FUNCTION: CKing::set_control_point
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\king.h:55
// RVA: 0x000A46C0
// ADDRESS: 004a46c0
// PROTOTYPE: void __thiscall set_control_point(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CKing::set_material_point
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\king.h:65
// RVA: 0x000A46F0
// ADDRESS: 004a46f0
// PROTOTYPE: void __thiscall set_material_point(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CKing::set_war_point
// STATUS: IMPLEMENTED_SOURCE_REFERENCE
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\king.h:75
// RVA: 0x000A4720
// ADDRESS: 004a4720
// PROTOTYPE: void __thiscall set_war_point(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CKing::CKing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\king.cpp:9
// RVA: 0x000DFCF0
// ADDRESS: 004dfcf0
// PROTOTYPE: undefined __thiscall CKing(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CKing::~CKing
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\king.cpp:19
// RVA: 0x000DFD20
// ADDRESS: 004dfd20
// PROTOTYPE: void __thiscall ~CKing(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CKing::ChangeControlPoint
// STATUS: IMPLEMENTED
// COMPONENT: WorldServer
// ARTIFACT: WorldServer/Nworldserver.exe + WorldServer/WorldServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\worldserver\appworld\country\king.cpp:84
// RVA: 0x000DFD30
// ADDRESS: 004dfd30
// PROTOTYPE: void __thiscall ChangeControlPoint(long param_1)
//
// Реализовано выше через `change_control_point`; exact disassembly
// `0x004DFD30..0x004DFD5A` подтверждает wrapping add и только upper clamp.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: WorldServer
