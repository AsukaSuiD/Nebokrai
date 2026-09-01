//! Владелец переходов повозки `CCarriage`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/ai/carriage.cpp`. Контроллер хранит действие, секундную проверку
//! хозяина, выход хозяина, повторную привязку, таймер исчезновения и отдельную
//! active-задержку `Move/Stand`. `FindPositionForCarriage` требует ненулевую
//! активную повозку хозяина и задаёт цель ровно в двух клетках сзади; фактический
//! `CBaseAI::MoveTo` выполняет к ней один figure-aware `Slip`-шаг. `CGame`
//! оставляет пространственное перемещение, привязку игрока, журналирование,
//! пакеты и фактическое удаление.
//!
//! Унаследованный `CPet::GetPetMaster` материализован общим pet-owner-ом;
//! технические конструкторы и RTTI исходника заменены обычным владением
//! `CMonster`.

use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates};
use crate::gameserver::appserver::skills::baseattack::real_distance;

pub(crate) const CARRIAGE_FOLLOWING: i32 = 0;
pub(crate) const CARRIAGE_STAYING: i32 = 1;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CarriageLifecycleState {
    action: i32,
    invalid_master_ms: u32,
    seek_master_ms: u32,
    master_logout: bool,
    schedule_started_ms: u32,
    schedule_delay_ms: u32,
    schedule_pending: bool,
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
    Stand,
    Wait,
    Follow,
}

pub(crate) fn plan_carriage_movement(
    action: i32,
    moveable: bool,
    carriage: &CShape,
    master: Option<&CShape>,
    master_in_same_region: bool,
    master_has_carriage: bool,
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
        if !master_in_same_region {
            return CarriageMovementPlan::Stand;
        }
        let distance = real_distance(carriage_x, carriage_y, master_x, master_y);
        if distance <= 2 || !master_has_carriage {
            return CarriageMovementPlan::Stand;
        }
        let Ok(rear) = master.get_rear_direction() else {
            return CarriageMovementPlan::None;
        };
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
            CarriageMovementPlan::Stand
        } else {
            CarriageMovementPlan::Move {
                x: destination.x,
                y: destination.y,
            }
        }
    } else if action == CARRIAGE_STAYING
        && master_in_same_region
        && real_distance(carriage_x, carriage_y, master_x, master_y) <= stop_distance
    {
        CarriageMovementPlan::Follow
    } else if action == CARRIAGE_STAYING {
        CarriageMovementPlan::Stand
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

    /// Отдельная active FIFO исходного auxiliary `CCarriage`: завершившийся
    /// `Move/Stand` освобождает расписание только на следующем AI-такте.
    pub(crate) fn advance_schedule(&mut self, now_ms: u32) -> bool {
        if !self.schedule_pending {
            return true;
        }
        if now_ms.wrapping_sub(self.schedule_started_ms) >= self.schedule_delay_ms {
            self.schedule_pending = false;
        }
        false
    }

    pub(crate) const fn block_schedule(&mut self, now_ms: u32, delay_ms: u32) {
        self.schedule_started_ms = now_ms;
        self.schedule_delay_ms = delay_ms;
        self.schedule_pending = true;
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
