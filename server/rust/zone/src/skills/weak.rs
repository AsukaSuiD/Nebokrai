//! Правила области ослабления CWeakPhalanx и срока призыва CWeak.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! appserver/skills/weakphalanx.cpp/.h и appserver/skills/weak.cpp.
//! Конструктор области: VA 0x600730; обход: 0x600840; AI: 0x600a70;
//! расчёт срока призыва: 0x5AF41B–0x5AF48C.

use crate::combat::{MasterInfo, truncate_original};
use crate::effects::WeakState;

pub const WEAK_SKILL_ID: u32 = 0x12e;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WeakPhalanxTick {
    Scan,
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeakPhalanx {
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    length: i32,
    height: i32,
    attack_loss: u32,
}

impl WeakPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля соответствуют конструктору EXE")]
    pub const fn new(
        master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, length: i32, height: i32, attack_loss: u32,
    ) -> Self {
        Self { master, started_at_ms, lifetime_ms, skill_level, length, height, attack_loss }
    }

    pub const fn master(&self) -> MasterInfo { self.master }
    pub const fn skill_level(&self) -> i32 { self.skill_level }
    pub const fn attack_loss(&self) -> u32 { self.attack_loss }
    pub const fn length(&self) -> i32 { self.length }
    pub const fn height(&self) -> i32 { self.height }
    pub const fn started_at_ms(&self) -> u32 { self.started_at_ms }
    pub const fn lifetime_ms(&self) -> u32 { self.lifetime_ms }

    pub const fn tick(&self, now_ms: u32) -> WeakPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            WeakPhalanxTick::Expired
        } else {
            WeakPhalanxTick::Scan
        }
    }

    pub fn active_cells(&self, center_x: i32, center_y: i32) -> Vec<(i32, i32)> {
        let start_x = center_x.wrapping_sub(self.length / 2);
        let start_y = center_y.wrapping_sub(self.height / 2);
        let mut cells = Vec::new();
        for x in start_x..start_x.wrapping_add(self.length) {
            // В оригинале внутренний предел тоже использует length, не height.
            for y in start_y..start_y.wrapping_add(self.length) {
                cells.push((x, y));
            }
        }
        cells
    }

    pub const fn state(&self, center_x: i32, center_y: i32) -> WeakState {
        WeakState::new(self.attack_loss, center_x, center_y, self.length, self.height)
    }
}

pub fn weak_lifetime(lifetime_factor: u32, element_modify: u32, base_lifetime: u32) -> u32 {
    let factor = lifetime_factor.wrapping_mul(element_modify).wrapping_add(100);
    let factor = (f64::from(factor) * f64::from(0.01_f32)) as f32;
    truncate_original(f64::from(base_lifetime) * f64::from(factor)) as u32
}
