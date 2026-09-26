//! Контроллер повозки `CCarriage` (AI12): действие, секундная проверка
//! хозяина, выход хозяина, повторная привязка, таймер исчезновения и план
//! follow/stay-движения. Единственный класс без собственных `OnIdle`/`Tracing`
//! (факт реестра `ai/monsterai.rs`). Исходный владелец PDB:
//! `appserver/ai/carriage.cpp`; сверка по точной паре `gameserver.exe` +
//! `GameServer.pdb` (разобраны 4 из 4 расписаний; ctor/dtor построчно не
//! читались — их нулевые lifecycle-поля зафиксированы владельцем состояния).
//!
//! Абсолютные wrapping DWORD deadlines хранятся буквально; каждое исходное
//! чтение `timeGetTime` приходит отдельным вызовом `now` от делегата.
//! Пространственное перемещение, журнал `0x6020E`, пакеты `GS0007..GS0009`,
//! фактический `Evanish` и привязка `m_nCarriageID` игрока выполняются
//! hub-владеющим lifecycle-входом поверх планов ниже; унаследованный
//! `CPet::GetPetMaster` материализован общим pet-owner-ом, и мост к pet-семье
//! сведён к совместному владельцу `ai/pet.rs`.
//! Доказательства: docs/reconstruction/gameserver-npc-and-regions.md#ai-расписаний-и-поведение

use crate::regions::shape::{CShape, ShapeAreaCoordinates};

pub const CARRIAGE_FOLLOWING: i32 = 0;
pub const CARRIAGE_STAYING: i32 = 1;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CarriageLifecycleState {
    action: i32,
    invalid_master_ms: u32,
    seek_master_ms: u32,
    master_logout: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CarriageMasterFacts {
    pub disappear_time_ms: u32,
    pub master_present: bool,
    pub master_owns_other: bool,
    pub master_close: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CarriageMasterOutcome {
    pub checked: bool,
    pub duplicate: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CarriageMovementPlan {
    None,
    Move { x: i32, y: i32 },
    Stand,
    Wait,
    Follow,
}

#[allow(clippy::too_many_arguments, reason = "факты сохраняют отдельные исходные предикаты")]
pub fn plan_carriage_movement(
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
    pub const fn action(&self) -> i32 {
        self.action
    }

    pub const fn set_action(&mut self, action: i32) {
        self.action = action;
    }

    pub fn begin_master_check(
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

    pub fn master_timed_out(
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
