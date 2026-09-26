//! Контроллер повозки `CCarriage` (AI12): действие, секундная проверка
//! хозяина, выход хозяина, повторная привязка, таймер исчезновения и план
//! follow/stay-движения. Единственный класс без собственных `OnIdle`/
//! `Tracing` (факт реестра `ai/monsterai.rs`).
//!
//! Точная пара `GameServer/gameserver.exe + GameServer/GameServer.pdb`
//! (EXE SHA-256 `4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E`,
//! PDB RSDS `5BEE6DD1-BF90-49B8-8BE9-EB25C4038D53` age 2, match; RVA истинные,
//! VA − 0x400000). Исходный владелец PDB:
//! `e:\svn\fengyun_russia_dev\server\gameserver\appserver\ai\carriage.cpp`.
//! Тела дочитаны машинно волной Z-AI (расписания и lifecycle 4 из 4; ctor
//! `0x005066D0`/dtor `0x00506700` построчно не читались — их известный
//! эффект, нулевые lifecycle-поля, зафиксирован владельцем состояния):
//!
//! | правило | якорь | здесь | статус |
//! |---|---|---|---|
//! | `SetCurrentAction` — прямая запись `m_caAction` (`[+0x7C]`) | VA `0x00506710` | [`CarriageLifecycleState::set_action`], константы [`CARRIAGE_FOLLOWING`]/[`CARRIAGE_STAYING`] | `MATCH` |
//! | follow-расписание `OnSchedule` (VA `0x00506A20`): пустой hook → умерший владелец при `GetState != 1` (vt `+0x74`) — журнал и `Evanish` (vt `+0x188`); затем ветвь действия, секундный (`0x3E8` мс) исходный timeGetTime-гейт `m_dwSeekMasterTimeStamp` и хвост master-проверки | VA `0x00506A20` | [`CarriageLifecycleState::begin_master_check`], [`CarriageLifecycleState::master_timed_out`]; журнал/Evanish/пакеты — hub | `MATCH` |
//! | `OnFallowingSchedule`: очереди `[+0x14]`/`[+0x28]` пусты и `IsMoveable` (vt `+0x148`); хозяин `CPet::GetPetMaster`, тот же регион (vt `+0x44` `GetRegionID`), знаковая дистанция > 2, `FindPositionForCarriage` (`0x004CCFD0`) даёт клетку ровно в двух шагах сзади; при новой клетке: повторная проверка региона и **беззнаковая** дистанция `≤ dwCarriageStopDistance` (`0x00EF45D0`) → общий `MoveTo(run=0)` (AI vtable `+0x58`); иначе GS0008 владельцу-игроку и `m_caAction = STAYING`; при провале фильтров — `ASA_STAND(1000)` | VA `0x00506720` | [`plan_carriage_movement`] | `MATCH` |
//! | `OnStayingSchedule`: хозяин в том же регионе и беззнаковая дистанция `≤ dwCarriageStopDistance` → GS0009 и `m_caAction = FALLOWING`; иначе `ASA_STAND(1000)` | VA `0x005068F0` | [`plan_carriage_movement`] | `MATCH` |
//! | master-проверка: отсутствующий хозяин (> другого региона — тоже отсутствие) при первом пропадании переводит в `STAYING` и запускает таймер; дубликат `m_nCarriageID` хозяина вызывает Evanish до distance-хвоста (`0x00506CFE` → `0x00506D04`); возвращение хозяина — прямая запись `m_nCarriageID` и `FALLOWING`, тоже до таймера расстояния; `disappear` — GS0007 и Evanish | VA `0x00506A20`, RVA-хвост `0x00506CFE`/`0x00506D04` | [`CarriageLifecycleState::begin_master_check`], [`CarriageLifecycleState::master_timed_out`] | `MATCH` |
//!
//! Абсолютные wrapping DWORD deadlines хранятся буквально; каждое исходное
//! чтение `timeGetTime` приходит отдельным вызовом `now` от делегата.
//! Пространственное перемещение (`CBaseAI::MoveTo` со slip-шагом), журнал
//! `0x6020E`, пакеты `GS0007`/`GS0008`/`GS0009`, фактический `Evanish`
//! (`CMonster::Evanish` `0x004E7A60` → `0x004CD700`) и привязка `m_nCarriageID`
//! игрока выполняются hub-владеющим lifecycle-входом поверх планов ниже.
//! Граница порции Z-AI (не расхождения): общий tick `Run` (`0x0060E250` →
//! `CBaseAI::Run` `0x004C7D10`), очереди `CBaseAI` и выбранный hub-вход
//! lifecycle повозки (`CGame`) остаются у своих порций.
//!
//! Унаследованный `CPet::GetPetMaster` материализован общим pet-owner-ом;
//! мост к pet-семье сведён к совместному владельцу `ai/pet.rs` (lifecycle FSM
//! питомца) и не требует данных повозки.

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
