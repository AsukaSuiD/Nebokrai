//! Владелец king state исторического `WorldServer`, перенесённый в Realm
//! `organizations/`.
//!
//! Три setter-а king points,
//! `ChangeControlPoint` и constructor/destructor
//! — часть контракта owner-а.
//! `ChangeControlPoint` применяет только upper clamp: отрицательные значения не
//! исправляются. Rust применяет тот же контракт к действующему king-state без
//! воспроизведения C++ inheritance/layout.
//! `ChangeControlPoint` сначала делает wrapping
//! signed add, затем сравнивает результат с `_max_king_control_point` и только
//! превышение заменяет максимумом. Действующие country call-sites передают
//! отрицание стоимости через `wrapping_neg`, сохраняя x86 `neg/add` даже для
//! `INT_MIN`; parameter singleton заменён явной ссылкой на `CCountryParam`.
//!
//! Constructor обнуляет только officer bytes и три king-точки: `m_bRegister`
//! остаётся неинициализированным, а подтверждённых project-caller-ов у него
//! нет. Это внутренний UB-дефект, не контракт: safe Rust назначает ему `false`
//! и не переносит случайное значение allocator-а. Исходный layout —
//! docs/reconstruction/realm-services.md.
//!
//! Data-типы point-отчётов (`KingPointKind`, `KingPointUpdate`) лежат рядом в
//! `organizations/country`.

use crate::content::countryparam::{CCountryParam, CountryParameterUnavailable};
use crate::organizations::country::{KingPointKind, KingPointUpdate};
use crate::organizations::dbcountry::CountryKingSaveSnapshot;

use super::officer::COfficer;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CKing {
    officer: COfficer,
    control_point: i32,
    material_point: i32,
    war_point: i32,
    registered: bool,
}

impl Default for CKing {
    fn default() -> Self {
        Self::with_constructor_defaults()
    }
}

impl CKing {
    pub const fn with_constructor_defaults() -> Self {
        Self {
            officer: COfficer::with_constructor_defaults(),
            control_point: 0,
            material_point: 0,
            war_point: 0,
            registered: false,
        }
    }

    pub const fn officer(&self) -> &COfficer {
        &self.officer
    }

    pub fn officer_mut(&mut self) -> &mut COfficer {
        &mut self.officer
    }

    pub const fn control_point(&self) -> i32 {
        self.control_point
    }

    pub const fn material_point(&self) -> i32 {
        self.material_point
    }

    pub const fn war_point(&self) -> i32 {
        self.war_point
    }

    pub const fn registered(&self) -> bool {
        self.registered
    }

    pub const fn set_registered(&mut self, registered: bool) {
        self.registered = registered;
    }

    pub fn set_control_point(
        &mut self,
        requested: i32,
        parameters: &CCountryParam,
    ) -> Result<KingPointUpdate, CountryParameterUnavailable> {
        let maximum = parameters
            .max_king_control_point()
            .ok_or(CountryParameterUnavailable {
                field: "_max_king_control_point",
            })?;
        Ok(apply_point(
            &mut self.control_point,
            requested,
            maximum,
            KingPointKind::Control,
        ))
    }

    pub fn set_material_point(
        &mut self,
        requested: i32,
        parameters: &CCountryParam,
    ) -> Result<KingPointUpdate, CountryParameterUnavailable> {
        let maximum = parameters
            .max_king_material_point()
            .ok_or(CountryParameterUnavailable {
                field: "_max_king_material_point",
            })?;
        Ok(apply_point(
            &mut self.material_point,
            requested,
            maximum,
            KingPointKind::Material,
        ))
    }

    pub fn set_war_point(
        &mut self,
        requested: i32,
        parameters: &CCountryParam,
    ) -> Result<KingPointUpdate, CountryParameterUnavailable> {
        let maximum = parameters
            .max_king_war_point()
            .ok_or(CountryParameterUnavailable {
                field: "_max_king_war_point",
            })?;
        Ok(apply_point(
            &mut self.war_point,
            requested,
            maximum,
            KingPointKind::War,
        ))
    }

    pub fn change_control_point(
        &mut self,
        delta: i32,
        parameters: &CCountryParam,
    ) -> Result<KingPointUpdate, CountryParameterUnavailable> {
        self.set_control_point(self.control_point.wrapping_add(delta), parameters)
    }
}

pub fn set_control_point(
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

pub fn set_material_point(
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

pub fn change_control_point(
    king: &mut CountryKingSaveSnapshot,
    delta: i32,
    parameters: &CCountryParam,
) -> Result<KingPointUpdate, CountryParameterUnavailable> {
    let requested = king.control_point.wrapping_add(delta);
    set_control_point(king, requested, parameters)
}

pub fn set_war_point(
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
