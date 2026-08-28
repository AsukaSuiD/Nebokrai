//! Периодическая область второго божественного грома `CGodThunderPhalanx2` (`0x143`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/godthunderphalanx2.cpp`. Три таблицы по адресам
//! `0x006A4AF8/0x006A4B2C/0x006A4B60` совпадают с маской 7×7 грома боевого
//! духа. `Initialize` сохраняет пары MSVCRT RNG по окнам; AI для каждой
//! выбранной клетки обрабатывает сначала упорядоченный war-soul index, затем
//! обычные фигуры. Повтор клетки и четыре RNG-вызова на две атаки сохраняются.

use super::godthunder2::GOD_THUNDER_2_SKILL_ID;
use super::thunderphalanx::{THUNDER_SCOPE, THUNDER_SCOPE_SIDE};
use crate::gameserver::appserver::legacycodec::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::public::guid::CGuid;

const SCOPE_AREA: u32 = 49;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum GodThunder2PhalanxTick { Pending, Attack { sampled_at_ms: u32 }, Expired }

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CGodThunderPhalanx2 {
    shape: CShape, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
    skill_level: i32, frequency_ms: u32, minimum_attack: i32,
    maximum_attack: i32, element_modifier: i32, target_count: u32, cch: i32,
    last_attack_ms: u32, attack_count: u32, cells: Vec<(i32, i32)>,
}

impl CGodThunderPhalanx2 {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, frequency_ms: u32, minimum_attack: i32,
        maximum_attack: i32, element_modifier: i32, target_count: u32, cch: i32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self { shape, master, started_at_ms, lifetime_ms, skill_level,
            frequency_ms: frequency_ms.max(1), minimum_attack, maximum_attack,
            element_modifier, target_count, cch, last_attack_ms: 0,
            attack_count: 0, cells: Vec::new() }
    }
    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) fn finish(&mut self) { self.shape.set_change_state(SHAPE_CHANGE_DELETE); }
    pub(crate) fn initialize(&mut self, tile_x: i32, tile_y: i32, random: &mut dyn FnMut(i32) -> i32) {
        let windows = self.lifetime_ms / self.frequency_ms;
        self.cells = vec![(0, 0); windows.wrapping_mul(SCOPE_AREA) as usize];
        let origin_x = tile_x.wrapping_sub(3); let origin_y = tile_y.wrapping_sub(3);
        for window in 0..windows { for target in 0..self.target_count {
            let (x, y) = loop { let x = random(THUNDER_SCOPE_SIDE); let y = random(THUNDER_SCOPE_SIDE);
                let index = x.wrapping_add(THUNDER_SCOPE_SIDE.wrapping_mul(y)) as usize;
                if THUNDER_SCOPE.get(index).copied().unwrap_or_default() != 0 { break (x, y); } };
            let index = window.wrapping_mul(SCOPE_AREA).wrapping_add(target) as usize;
            if let Some(cell) = self.cells.get_mut(index) { *cell = (origin_x.wrapping_add(x), origin_y.wrapping_add(y)); }
        } }
    }
    pub(crate) fn tick(&mut self, now: u32) -> GodThunder2PhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now { self.finish(); return GodThunder2PhalanxTick::Expired; }
        if self.frequency_ms.wrapping_add(self.last_attack_ms) < now { self.last_attack_ms = now; self.attack_count = self.attack_count.wrapping_add(1); return GodThunder2PhalanxTick::Attack { sampled_at_ms: now }; }
        GodThunder2PhalanxTick::Pending
    }
    pub(crate) fn attack_cells(&self) -> impl Iterator<Item = (i32, i32)> + '_ {
        let start = self.attack_count.wrapping_sub(1).wrapping_mul(SCOPE_AREA) as usize;
        self.cells.get(start..start.saturating_add(SCOPE_AREA as usize)).unwrap_or_default()
            .iter().copied().take_while(|cell| *cell != (0, 0))
    }
    pub(crate) fn encode_client_snapshot(&self, mut now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let first = now(); let remained = if self.started_at_ms.wrapping_add(self.lifetime_ms) <= first { 0 } else { self.lifetime_ms.wrapping_sub(now()).wrapping_add(self.started_at_ms) };
        let mut payload = Vec::new(); { let mut writer = LegacyWriter::new(&mut payload);
            writer.write_i32(GOD_THUNDER_2_SKILL_ID as i32); writer.write_i32(self.skill_level);
            writer.write_i32(self.shape.identity().object_type); writer.write_i32(self.shape.identity().id);
            writer.write_u32(remained); writer.write_u32(self.lifetime_ms); writer.write_u32(self.frequency_ms);
            let count = (self.lifetime_ms / self.frequency_ms).wrapping_mul(self.target_count); writer.write_u32(count);
            for &(x, y) in self.cells.iter().take(count as usize) { writer.write_i32(x); writer.write_i32(y); }
        } self.shape.encode_to_byte_array(&mut payload, true).then_some(payload)
    }
    pub(crate) fn calculate_attack(&self, combat: PlayerCombatProperties, occupation: u8, level: u8, factor: f32, critical_rate: f32, random: &mut dyn FnMut(i32) -> i32) -> (AttackInformation, PlayerCombatProperties, u8, u8) {
        let width = self.maximum_attack.wrapping_sub(self.minimum_attack).wrapping_abs().wrapping_add(1);
        let damage = self.minimum_attack.wrapping_add(random(width)).wrapping_add(self.element_modifier).max(0);
        let mut attack = AttackInformation { skill_id: GOD_THUNDER_2_SKILL_ID, skill_level: self.skill_level as u8,
            attacker_type: self.master.master_type, attacker_id: self.master.master_id,
            attacker_team_id: self.master.master_team_id, attacker_faction_id: self.master.master_guild_id,
            attacker_union_id: self.master.master_union_id, hit_modifier: 100, damage_factor: factor,
            damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0,
            damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 }] };
        if random(100) < self.cch { attack.critical = true; for power in &mut attack.damages { power.hp_damage = (power.hp_damage as f32 * critical_rate).round_ties_even() as i32; } }
        (attack, combat, occupation, level)
    }
}
