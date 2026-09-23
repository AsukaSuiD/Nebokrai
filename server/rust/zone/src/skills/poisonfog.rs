//! Данные живой области CPoisonFogPhalanx; регион и CShape остаются у адаптера.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! appserver/skills/poisonfogphalanx.cpp/.h.
//! Конструктор VA 0x005FBD90, AI VA 0x005FC040.

use crate::combat::MasterInfo;
use crate::effects::PoisonFogState;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PoisonFogPhalanxTick {
    Scan,
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoisonFogPhalanx {
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    state_template: PoisonFogState,
    cell_active: bool,
}

impl PoisonFogPhalanx {
    pub const fn new(
        master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        state_template: PoisonFogState,
    ) -> Self {
        Self { master, started_at_ms, lifetime_ms, state_template, cell_active: true }
    }

    pub const fn master(&self) -> MasterInfo { self.master }
    pub const fn skill_level(&self) -> i32 { self.state_template.skill_level() }
    pub const fn started_at_ms(&self) -> u32 { self.started_at_ms }
    pub const fn lifetime_ms(&self) -> u32 { self.lifetime_ms }
    pub const fn cell_active(&self) -> bool { self.cell_active }

    pub fn clear_cell(&mut self) { self.cell_active = false; }

    pub const fn tick(&self, now_ms: u32) -> PoisonFogPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            PoisonFogPhalanxTick::Expired
        } else {
            PoisonFogPhalanxTick::Scan
        }
    }

    pub fn state(&self) -> PoisonFogState { self.state_template.clone() }
}
