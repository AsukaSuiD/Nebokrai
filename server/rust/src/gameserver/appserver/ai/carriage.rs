//! Владелец переходов повозки `CCarriage`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/ai/carriage.cpp`. Контроллер хранит действие, секундную проверку
//! хозяина, выход хозяина, повторную привязку, таймер исчезновения и отдельную
//! базовую FIFO своего AI. `FindPositionForCarriage` требует ненулевую
//! активную повозку хозяина и задаёт цель ровно в двух клетках сзади; фактический
//! `CBaseAI::MoveTo` выполняет к ней один figure-aware `Slip`-шаг. `CGame`
//! оставляет пространственное перемещение, привязку игрока, журналирование,
//! пакеты и фактическое удаление.
//!
//! Унаследованный `CPet::GetPetMaster` материализован общим pet-owner-ом;
//! технические конструкторы и RTTI исходника заменены обычным владением
//! `CMonster`.
//! Run (0x0060E250 → 0x004C7D10) сначала вызывает OnSchedule, затем
//! обслуживает очереди. Только OnFollowing/OnStaying проверяют их пустоту:
//! смерть и секундная проверка хозяина продолжаются при незавершённом Move.
//! Stand(1000) и MoveTo используют общий CBaseAI, без отдельного таймера
//! задержки: сохраняются handling и абсолютный DWORD deadline. Его очередь
//! независима от первичного AI того же монстра; OnIdle повозки пуст.
//! Проверки расстояния используют CShape::Distance(CShape*) с figure extents:
//! порог 2 сравнивается знаково, carriage_stop_distance — как DWORD.
//! Секундный gate и исчезновение сравнивают абсолютные wrapping DWORD
//! deadlines; каждый исходный timeGetTime получает отдельное чтение часов.
//! Проверка хозяина разделена границей внешнего эффекта: удаление duplicate
//! (0x00506CFE) выполняется до distance/timeout-хвоста (0x00506D04), а не
//! заменяет его. Повторная привязка также предшествует таймеру расстояния.
//! CMonster::Evanish (0x004E7A60 → 0x004CD700) удаляет pet association
//! у найденного в регионе хозяина, помечает форму и каждый раз шлёт BF504.
//! Он не очищает active_carriage и не прерывает оставшиеся фазы текущего Run;
//! после duplicate допускается повторное Evanish по истёкшему таймеру.
//! В секундной проверке хозяин другого региона считается отсутствующим;
//! timeout-notice повторно ищет игрока глобально, уже без этого ограничения.

use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates};

pub(crate) const CARRIAGE_FOLLOWING: i32 = 0;
pub(crate) const CARRIAGE_STAYING: i32 = 1;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CarriageLifecycleState {
    action: i32,
    invalid_master_ms: u32,
    seek_master_ms: u32,
    master_logout: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CarriageMasterFacts {
    pub(crate) disappear_time_ms: u32,
    pub(crate) master_present: bool,
    pub(crate) master_owns_other: bool,
    pub(crate) master_close: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CarriageMasterOutcome {
    pub(crate) checked: bool,
    pub(crate) duplicate: bool,
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
    master_distance: i32,
    stop_distance: u32,
) -> CarriageMovementPlan {
    if !moveable {
        return CarriageMovementPlan::None;
    }
    let Some(master) = master else {
        return if action == CARRIAGE_STAYING {
            CarriageMovementPlan::Stand
        } else {
            CarriageMovementPlan::None
        };
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
        if master_distance <= 2 || !master_has_carriage {
            return CarriageMovementPlan::Stand;
        }
        let Ok(rear) = master.get_rear_direction() else {
            return CarriageMovementPlan::None;
        };
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
        } else if master_distance as u32 > stop_distance {
            CarriageMovementPlan::Wait
        } else {
            CarriageMovementPlan::Move {
                x: destination.x,
                y: destination.y,
            }
        }
    } else if action == CARRIAGE_STAYING
        && master_in_same_region
        && master_distance as u32 <= stop_distance
    {
        CarriageMovementPlan::Follow
    } else if action == CARRIAGE_STAYING {
        CarriageMovementPlan::Stand
    } else {
        CarriageMovementPlan::None
    }
}

impl CarriageLifecycleState {
    pub(crate) const fn action(&self) -> i32 {
        self.action
    }

    pub(crate) const fn set_action(&mut self, action: i32) {
        self.action = action;
    }

    pub(crate) fn begin_master_check(
        &mut self,
        facts: CarriageMasterFacts,
        mut now: impl FnMut() -> u32,
        mut rebind: impl FnMut(),
    ) -> CarriageMasterOutcome {
        if now() < self.seek_master_ms.wrapping_add(1_000) {
            return CarriageMasterOutcome::default();
        }
        self.seek_master_ms = now();
        let mut outcome = CarriageMasterOutcome {
            checked: true,
            ..Default::default()
        };
        if !facts.master_present {
            if !self.master_logout {
                self.master_logout = true;
                if self.invalid_master_ms == 0 {
                    self.action = CARRIAGE_STAYING;
                    self.invalid_master_ms = now();
                }
            }
        } else {
            if !self.master_logout && facts.master_owns_other {
                outcome.duplicate = true;
            } else if self.master_logout {
                self.action = CARRIAGE_FOLLOWING;
                rebind();
                self.master_logout = false;
            }
        }
        outcome
    }

    pub(crate) fn master_timed_out(
        &mut self,
        facts: CarriageMasterFacts,
        mut now: impl FnMut() -> u32,
    ) -> bool {
        if facts.master_present {
            if facts.master_close {
                self.invalid_master_ms = 0;
            } else if self.invalid_master_ms == 0 {
                self.invalid_master_ms = now();
            }
        }
        self.invalid_master_ms != 0
            && now() >= self.invalid_master_ms.wrapping_add(facts.disappear_time_ms)
    }
}
