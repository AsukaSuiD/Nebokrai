//! Движущаяся область CChaosSpherePhalanx.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! EXE SHA-256 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E,
//! PDB SHA-256 B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016.
//! Ctor VA 0x005FEE80, AddToByteArray VA 0x005FECB0, AI VA 0x005FF270
//! (appserver/skills/chaosspherephalanx.cpp/.h).

use nebokrai_shared::protocol::LegacyWriter;
use crate::effects::timed_client_state_time;
use super::ElementPhalanxAttack;

pub const CHAOS_SPHERE_SKILL_ID: u32 = 0x137;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ChaosSpherePhalanx {
    attack: ElementPhalanxAttack,
    started_at_ms: u32,
    lifetime_ms: u32,
    frequency_ms: u32,
    path: Vec<(i32, i32)>,
    speed_ms: u32,
    last_attack_ms: u32,
    last_move_ms: u32,
    current_cell: u32,
    force_moved: bool,
}

impl ChaosSpherePhalanx {
    pub fn new(
        attack: ElementPhalanxAttack, started_at_ms: u32, lifetime_ms: u32,
        frequency_ms: u32, path: Vec<(i32, i32)>, speed_ms: u32,
    ) -> Self {
        Self { attack, started_at_ms, lifetime_ms, frequency_ms, path, speed_ms,
            last_attack_ms: 0, last_move_ms: 0, current_cell: 0, force_moved: false }
    }

    pub const fn master(&self) -> crate::combat::MasterInfo { self.attack.master }
    pub const fn attack_snapshot(&self) -> ElementPhalanxAttack { self.attack }
    pub const fn expired_at(&self, now: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now
    }
    pub fn has_path(&self) -> bool { !self.path.is_empty() }
    pub fn initial_force_move(&self) -> Option<(i32, i32, u32)> {
        if self.force_moved { return None; }
        let &(x, y) = self.path.last()?;
        Some((x, y, (self.path.len() as u32).wrapping_mul(self.speed_ms)))
    }
    pub fn mark_force_moved_at(&mut self, now: u32) {
        self.last_move_ms = now;
        self.force_moved = true;
    }
    pub const fn movement_due_at(&self, now: u32) -> bool {
        self.speed_ms.wrapping_add(self.last_move_ms) < now
    }
    pub fn advance_at(&mut self, now: u32) {
        self.last_move_ms = now;
        if self.current_cell < (self.path.len() as u32).wrapping_sub(1) {
            self.current_cell = self.current_cell.wrapping_add(1);
        }
    }
    pub const fn attack_due_at(&self, now: u32) -> bool {
        self.frequency_ms.wrapping_add(self.last_attack_ms) < now
    }
    pub fn mark_attack_at(&mut self, now: u32) { self.last_attack_ms = now; }
    pub fn attack_origin(&self) -> Option<(i32, i32)> {
        self.path.get(self.current_cell as usize)
            .map(|&(x, y)| (x.wrapping_sub(1), y.wrapping_sub(1)))
    }

    /// Пять полей перед базовым CShape; путь и часы периодов не сериализуются.
    pub fn write_client_snapshot_fields(&self, payload: &mut Vec<u8>, now: impl FnMut() -> u32) {
        let mut writer = LegacyWriter::new(payload);
        writer.write_u32(self.attack.skill_id);
        writer.write_i32(self.attack.skill_level);
        writer.write_i32(self.attack.master.master_type);
        writer.write_i32(self.attack.master.master_id);
        writer.write_u32(timed_client_state_time(self.started_at_ms, self.lifetime_ms, now));
    }
}
