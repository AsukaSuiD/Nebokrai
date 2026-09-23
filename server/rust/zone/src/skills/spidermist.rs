//! Данные области CSpiderMistPhalanx и её маска.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! appserver/skills/spidermistphalanx.cpp/.h.
//! Конструктор VA 0x005EAEB0, AI 0x005EB110, ReplaceAffectRegion 0x005EAC30.

use crate::combat::MasterInfo;

pub const SPIDER_MIST_SKILL_ID: u32 = 0x198;
const SCOPE_SIDE: usize = 5;
const SCOPE: [[u8; SCOPE_SIDE]; SCOPE_SIDE] = [[1; SCOPE_SIDE]; SCOPE_SIDE];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpiderMistPhalanxTick { Scan, Expired }

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SpiderMistPhalanx {
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    state_lifetime_ms: u32,
    frequency_ms: u32,
    hp_loss: u32,
    scope: [[u8; SCOPE_SIDE]; SCOPE_SIDE],
}

impl SpiderMistPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля соответствуют конструктору EXE")]
    pub const fn new(
        master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, state_lifetime_ms: u32, frequency_ms: u32, hp_loss: u32,
    ) -> Self {
        Self { master, started_at_ms, lifetime_ms, skill_level, state_lifetime_ms,
            frequency_ms, hp_loss, scope: SCOPE }
    }

    pub const fn master(&self) -> MasterInfo { self.master }
    pub const fn skill_level(&self) -> i32 { self.skill_level }
    pub const fn state_lifetime_ms(&self) -> u32 { self.state_lifetime_ms }
    pub const fn frequency_ms(&self) -> u32 { self.frequency_ms }
    pub const fn hp_loss(&self) -> u32 { self.hp_loss }

    pub const fn tick(&self, now_ms: u32) -> SpiderMistPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            SpiderMistPhalanxTick::Expired
        } else {
            SpiderMistPhalanxTick::Scan
        }
    }

    pub fn active_cells(&self, center_x: i32, center_y: i32) -> Vec<(i32, i32)> {
        let start_x = center_x.wrapping_sub((SCOPE_SIDE / 2) as i32);
        let start_y = center_y.wrapping_sub((SCOPE_SIDE / 2) as i32);
        let mut cells = Vec::new();
        for x in 0..SCOPE_SIDE {
            for y in 0..SCOPE_SIDE {
                if self.scope[y][x] != 0 {
                    cells.push((start_x.wrapping_add(x as i32), start_y.wrapping_add(y as i32)));
                }
            }
        }
        cells
    }

    pub fn replace_affect_region(
        &mut self, center_x: i32, center_y: i32,
        incoming_tile_x: i32, incoming_tile_y: i32,
    ) {
        let half = (SCOPE_SIDE / 2) as i32;
        let existing_left = center_x.wrapping_sub(half);
        let existing_top = center_y.wrapping_sub(half);
        let incoming_left = incoming_tile_x.wrapping_sub(half);
        let incoming_top = incoming_tile_y.wrapping_sub(half);
        let existing_right = existing_left.wrapping_add(SCOPE_SIDE as i32);
        let existing_bottom = existing_top.wrapping_add(SCOPE_SIDE as i32);
        let incoming_right = incoming_left.wrapping_add(SCOPE_SIDE as i32);
        let incoming_bottom = incoming_top.wrapping_add(SCOPE_SIDE as i32);

        let overlap_left = existing_left.max(incoming_left);
        let overlap_top = existing_top.max(incoming_top);
        let overlap_right = existing_right.min(incoming_right);
        let overlap_bottom = existing_bottom.min(incoming_bottom);
        if overlap_left >= overlap_right || overlap_top >= overlap_bottom { return; }

        for world_y in overlap_top..overlap_bottom {
            for world_x in overlap_left..overlap_right {
                let incoming_x = world_x.wrapping_sub(incoming_left) as usize;
                let incoming_y = world_y.wrapping_sub(incoming_top) as usize;
                if SCOPE[incoming_y][incoming_x] != 0 {
                    let existing_x = world_x.wrapping_sub(existing_left) as usize;
                    let existing_y = world_y.wrapping_sub(existing_top) as usize;
                    self.scope[existing_y][existing_x] = 0;
                }
            }
        }
    }
}
