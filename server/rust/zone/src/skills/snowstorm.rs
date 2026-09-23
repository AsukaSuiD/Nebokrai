//! Данные области CSnowStormPhalanx и окна выбранных клеток.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! appserver/skills/snowstormphalanx.cpp/.h.
//! Конструктор VA 0x005F9040, Initialize 0x005F8D70,
//! CalculateAttackPower 0x005F92D0, Attack 0x005F93B0, AI 0x005F94B0.

use crate::combat::{AttackInformation, AttackPower, AttackPowerType, MasterInfo};

pub const SNOW_STORM_SKILL_ID: u32 = 0x193;
pub const SNOW_STORM_SCOPE_AREA: u32 = 25;
const SCOPE_SIDE: i32 = 5;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SnowStormParametersError { ZeroFrequency, TooManyTargets, CellArrayTooLarge }

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnowStormAttack {
    pub master: MasterInfo,
    pub skill_level: i32,
    pub minimum_attack: i32,
    pub maximum_attack: i32,
    pub element_modifier: i32,
}

impl SnowStormAttack {
    pub fn attack_master(self) -> MasterInfo {
        if self.master.master_type == 400 { return self.master; }
        MasterInfo {
            master_type: self.master.master_type, master_id: self.master.master_id,
            ..MasterInfo::default()
        }
    }

    fn element_damage(self, random_below: impl FnOnce(i32) -> i32) -> i32 {
        let width = self.maximum_attack.wrapping_sub(self.minimum_attack)
            .wrapping_abs().wrapping_add(1);
        random_below(width).wrapping_add(self.minimum_attack)
            .wrapping_add(self.element_modifier).max(0)
    }

    pub fn attack_information(self, random_below: impl FnOnce(i32) -> i32) -> AttackInformation {
        let mut attack = AttackInformation::for_master(self.attack_master());
        attack.skill_id = SNOW_STORM_SKILL_ID;
        attack.skill_level = self.skill_level as u8;
        attack.damage_modifier = 0;
        attack.damage_factor = 1.0;
        attack.hit_modifier = 100;
        attack.damages.push(AttackPower {
            kind: AttackPowerType::Element,
            hp_damage: self.element_damage(random_below),
            mp_damage: 0,
        });
        attack
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SnowStormPhalanx {
    attack: SnowStormAttack,
    started_at_ms: u32,
    lifetime_ms: u32,
    frequency_ms: u32,
    target_count: u32,
    last_attack_ms: u32,
    attack_count: u32,
    cells: Vec<(i32, i32)>,
}

impl SnowStormPhalanx {
    pub fn new(
        attack: SnowStormAttack, started_at_ms: u32, lifetime_ms: u32,
        frequency_ms: u32, target_count: u32,
    ) -> Result<Self, SnowStormParametersError> {
        let windows = lifetime_ms.checked_div(frequency_ms)
            .ok_or(SnowStormParametersError::ZeroFrequency)?;
        if target_count > SNOW_STORM_SCOPE_AREA {
            return Err(SnowStormParametersError::TooManyTargets);
        }
        let count = windows.checked_mul(SNOW_STORM_SCOPE_AREA)
            .filter(|count| count.checked_mul(8).is_some())
            .ok_or(SnowStormParametersError::CellArrayTooLarge)?;
        let mut cells = Vec::new();
        cells.try_reserve_exact(count as usize)
            .map_err(|_| SnowStormParametersError::CellArrayTooLarge)?;
        cells.resize(count as usize, (0, 0));
        Ok(Self { attack, started_at_ms, lifetime_ms, frequency_ms, target_count,
            last_attack_ms: 0, attack_count: 0, cells })
    }

    pub const fn master(&self) -> MasterInfo { self.attack.master }
    pub const fn attack_snapshot(&self) -> SnowStormAttack { self.attack }
    pub const fn skill_level(&self) -> i32 { self.attack.skill_level }
    pub const fn started_at_ms(&self) -> u32 { self.started_at_ms }
    pub const fn lifetime_ms(&self) -> u32 { self.lifetime_ms }
    pub const fn frequency_ms(&self) -> u32 { self.frequency_ms }
    pub fn cells(&self) -> &[(i32, i32)] { &self.cells }

    pub fn initialize(
        &mut self, center_x: i32, center_y: i32,
        random_below: &mut dyn FnMut(i32) -> i32,
    ) {
        let origin_x = center_x.wrapping_sub(SCOPE_SIDE >> 1);
        let origin_y = center_y.wrapping_sub(SCOPE_SIDE >> 1);
        self.cells.fill((0, 0));
        for window in self.cells.chunks_exact_mut(SNOW_STORM_SCOPE_AREA as usize) {
            for cell in window.iter_mut().take(self.target_count as usize) {
                let x = random_below(SCOPE_SIDE);
                let y = random_below(SCOPE_SIDE);
                *cell = (origin_x.wrapping_add(x), origin_y.wrapping_add(y));
            }
        }
    }

    pub const fn expired_at(&self, now: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now
    }

    pub const fn attack_due_at(&self, now: u32) -> bool {
        self.frequency_ms.wrapping_add(self.last_attack_ms) < now
    }

    pub fn mark_attack_at(&mut self, now: u32) { self.last_attack_ms = now; }
    pub fn advance_attack_window(&mut self) { self.attack_count = self.attack_count.wrapping_add(1); }

    pub fn current_cell(&self, index: u32) -> Option<(i32, i32)> {
        let index = self.attack_count.wrapping_mul(SNOW_STORM_SCOPE_AREA).wrapping_add(index);
        // Native читает за массивом после его исчерпания; не создаём фиктивную цель.
        self.cells.get(index as usize).copied().filter(|cell| *cell != (0, 0))
    }
}
