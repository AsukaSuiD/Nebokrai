//! Владелец переходов повозки `CCarriage`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/ai/carriage.cpp`. Контроллер хранит действие, секундную проверку
//! хозяина, выход хозяина, повторную привязку и таймер исчезновения. `CGame`
//! оставляет пространственное перемещение, привязку игрока, журналирование,
//! пакеты и фактическое удаление.
//!
//! `CPet::GetPetMaster`, унаследованный повозкой, разрешает игрока через
//! глобальный реестр, а любой другой тип — только через текущий регион. В Rust
//! эта развилка выражена типизированной ссылкой; технические конструкторы и
//! RTTI исходника заменены обычным владением `CMonster`.

use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::skills::baseattack::real_distance;
use crate::public::guid::CGuid;

const PLAYER_TYPE: i32 = 400;

pub(crate) const CARRIAGE_FOLLOWING: i32 = 0;
pub(crate) const CARRIAGE_STAYING: i32 = 1;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CarriageLifecycleState {
    action: i32,
    invalid_master_ms: u32,
    seek_master_ms: u32,
    master_logout: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CarriageMasterFacts {
    pub(crate) now_ms: u32,
    pub(crate) disappear_time_ms: u32,
    pub(crate) master_present: bool,
    pub(crate) master_owns_other: bool,
    pub(crate) master_close: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CarriageMasterOutcome {
    pub(crate) checked: bool,
    pub(crate) rebound: bool,
    pub(crate) vanish_reason: Option<i32>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CarriageMovementPlan {
    None,
    Move { x: i32, y: i32 },
    Wait,
    Follow,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CarriageMasterRef {
    Player(i32),
    Region(ShapeIdentity),
}

/// Безопасный эквивалент унаследованного `CPet::GetPetMaster`: нулевой master
/// не разрешается, игрок ищется глобально, остальные типы остаются привязаны к
/// реестру текущего региона.
pub(crate) const fn carriage_master_ref(master: MasterInfo) -> Option<CarriageMasterRef> {
    if master.master_type == 0 || master.master_id == 0 {
        None
    } else if master.master_type == PLAYER_TYPE {
        Some(CarriageMasterRef::Player(master.master_id))
    } else {
        Some(CarriageMasterRef::Region(ShapeIdentity {
            object_type: master.master_type,
            id: master.master_id,
            ex_id: CGuid::GUID_INVALID,
        }))
    }
}

pub(crate) fn plan_carriage_movement(
    action: i32,
    moveable: bool,
    carriage: &CShape,
    master: Option<&CShape>,
    stop_distance: i32,
) -> CarriageMovementPlan {
    if !moveable {
        return CarriageMovementPlan::None;
    }
    let Some(master) = master else {
        return CarriageMovementPlan::None;
    };
    let (Ok(carriage_x), Ok(carriage_y), Ok(master_x), Ok(master_y)) = (
        carriage.get_tile_x(),
        carriage.get_tile_y(),
        master.get_tile_x(),
        master.get_tile_y(),
    ) else {
        return CarriageMovementPlan::None;
    };
    if action == CARRIAGE_FOLLOWING {
        let Ok(rear) = master.get_rear_direction() else {
            return CarriageMovementPlan::None;
        };
        let distance = real_distance(carriage_x, carriage_y, master_x, master_y);
        if distance <= 2 {
            return CarriageMovementPlan::None;
        }
        if distance > stop_distance {
            return CarriageMovementPlan::Wait;
        }
        let start = ShapeAreaCoordinates {
            x: master_x,
            y: master_y,
        };
        let Ok(first) = CShape::get_direction_position(rear, start) else {
            return CarriageMovementPlan::None;
        };
        let Ok(destination) = CShape::get_direction_position(rear, first) else {
            return CarriageMovementPlan::None;
        };
        if (carriage_x, carriage_y) == (destination.x, destination.y) {
            CarriageMovementPlan::None
        } else {
            CarriageMovementPlan::Move {
                x: destination.x,
                y: destination.y,
            }
        }
    } else if action == CARRIAGE_STAYING
        && real_distance(carriage_x, carriage_y, master_x, master_y) <= stop_distance
    {
        CarriageMovementPlan::Follow
    } else {
        CarriageMovementPlan::None
    }
}

impl CarriageLifecycleState {
    pub(crate) const fn action(self) -> i32 {
        self.action
    }

    pub(crate) const fn set_action(&mut self, action: i32) {
        self.action = action;
    }

    pub(crate) fn tick_master(&mut self, facts: CarriageMasterFacts) -> CarriageMasterOutcome {
        if facts.now_ms.wrapping_sub(self.seek_master_ms) < 1_000 {
            return CarriageMasterOutcome::default();
        }
        self.seek_master_ms = facts.now_ms;
        let mut outcome = CarriageMasterOutcome {
            checked: true,
            ..Default::default()
        };
        if !facts.master_present {
            if !self.master_logout {
                self.master_logout = true;
                if self.invalid_master_ms == 0 {
                    self.action = CARRIAGE_STAYING;
                    self.invalid_master_ms = facts.now_ms;
                }
            }
        } else {
            if !self.master_logout && facts.master_owns_other {
                outcome.vanish_reason = Some(5);
            } else if self.master_logout {
                self.action = CARRIAGE_FOLLOWING;
                self.master_logout = false;
                outcome.rebound = true;
            }
            if facts.master_close {
                self.invalid_master_ms = 0;
            } else if self.invalid_master_ms == 0 {
                self.invalid_master_ms = facts.now_ms;
            }
        }
        if self.invalid_master_ms != 0
            && facts.now_ms.wrapping_sub(self.invalid_master_ms) >= facts.disappear_time_ms
        {
            outcome.vanish_reason.get_or_insert(4);
        }
        outcome
    }
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\carriage.cpp

// COMPONENT_VARIANT_END: GameServer
