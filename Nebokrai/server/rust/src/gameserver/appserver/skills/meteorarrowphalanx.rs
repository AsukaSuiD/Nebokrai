//! Область падающих метеорных стрел `CMeteorArrowPhalanx` (`0xCD`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/meteorarrowphalanx.cpp`. Владелец хранит боевой снимок
//! стрелка, заранее выбирает по два значения MSVCRT RNG на каждую попытку
//! клетки и раз в заданную частоту обрабатывает одну клетку. `CGame` оставляет
//! за собой только разрешение региональных identity и применение удара.

use super::meteorarrow::METEOR_ARROW_SKILL_ID;
use crate::gameserver::appserver::legacycodec::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

const SCOPE_LENGTH: i32 = 7;
const SCOPE_HEIGHT: i32 = 7;
const SCOPE: [u8; 49] = [
    0, 0, 1, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 0,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0,
    0, 0, 1, 1, 1, 0, 0,
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MeteorArrowPhalanxTick { Pending, Attack { cell: (i32, i32), sampled_at_ms: u32 }, Expired }

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CMeteorArrowPhalanx {
    shape: CShape, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
    frequency_ms: u32, skill_level: i32, minimum_attack: i32, maximum_attack: i32,
    element_attack: i32, soul_attack: i32, critical_chance: i32, hit_modifier: i32,
    cells: Vec<(i32, i32)>, last_attack_at_ms: u32, attack_count: usize,
}

impl CMeteorArrowPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(id: i32, master: MasterInfo, started_at_ms: u32, frequency_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32, element_attack: i32,
        soul_attack: i32, critical_chance: i32, hit_modifier: i32, arrow_count: u32,
        center_x: i32, center_y: i32, mut random_below: impl FnMut(i32) -> i32) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        let start_x = center_x.wrapping_sub(SCOPE_LENGTH >> 1);
        let start_y = center_y.wrapping_sub(SCOPE_HEIGHT >> 1);
        let mut cells = Vec::with_capacity(arrow_count as usize);
        for _ in 0..arrow_count {
            let (scope_x, scope_y) = loop {
                let x = random_below(SCOPE_LENGTH); let y = random_below(SCOPE_HEIGHT);
                let index = y.wrapping_mul(SCOPE_LENGTH).wrapping_add(x);
                if usize::try_from(index).ok().is_some_and(|index| SCOPE.get(index).copied() == Some(1)) { break (x, y) }
            };
            cells.push((start_x.wrapping_add(scope_x), start_y.wrapping_add(scope_y)));
        }
        Self { shape, master, started_at_ms, lifetime_ms: frequency_ms.wrapping_mul(arrow_count).wrapping_add(10),
            frequency_ms, skill_level, minimum_attack, maximum_attack, element_attack, soul_attack,
            critical_chance, hit_modifier, cells, last_attack_at_ms: 0, attack_count: 0 }
    }
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn from_cells(id: i32, master: MasterInfo, started_at_ms: u32, frequency_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32, element_attack: i32,
        soul_attack: i32, critical_chance: i32, hit_modifier: i32, cells: Vec<(i32, i32)>) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        let arrow_count = cells.len() as u32;
        Self { shape, master, started_at_ms, lifetime_ms: frequency_ms.wrapping_mul(arrow_count).wrapping_add(10),
            frequency_ms, skill_level, minimum_attack, maximum_attack, element_attack, soul_attack,
            critical_chance, hit_modifier, cells, last_attack_at_ms: 0, attack_count: 0 }
    }
    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn skill_level(&self) -> i32 { self.skill_level }
    pub(crate) const fn minimum_attack(&self) -> i32 { self.minimum_attack }
    pub(crate) const fn maximum_attack(&self) -> i32 { self.maximum_attack }
    pub(crate) const fn element_attack(&self) -> i32 { self.element_attack }
    pub(crate) const fn soul_attack(&self) -> i32 { self.soul_attack }
    pub(crate) const fn critical_chance(&self) -> i32 { self.critical_chance }
    pub(crate) const fn hit_modifier(&self) -> i32 { self.hit_modifier }
    pub(crate) fn finish(&mut self) { self.shape.set_change_state(SHAPE_CHANGE_DELETE); }
    pub(crate) fn tick(&mut self, lifetime_now_ms: u32, attack_now_ms: impl FnOnce() -> u32) -> MeteorArrowPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < lifetime_now_ms || self.cells.is_empty() {
            self.finish(); return MeteorArrowPhalanxTick::Expired;
        }
        let now_ms = attack_now_ms();
        if now_ms <= self.last_attack_at_ms.wrapping_add(self.frequency_ms) { return MeteorArrowPhalanxTick::Pending }
        self.last_attack_at_ms = now_ms;
        let Some(&cell) = self.cells.get(self.attack_count) else { self.finish(); return MeteorArrowPhalanxTick::Expired };
        self.attack_count = self.attack_count.wrapping_add(1);
        MeteorArrowPhalanxTick::Attack { cell, sampled_at_ms: now_ms }
    }
    pub(crate) fn encode_client_snapshot(&self, mut now_milliseconds: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let first_now = now_milliseconds();
        let remained = if self.started_at_ms.wrapping_add(self.lifetime_ms) <= first_now { 0 }
            else { self.lifetime_ms.wrapping_sub(now_milliseconds()).wrapping_add(self.started_at_ms) };
        let x = self.shape.get_tile_x().ok()?; let y = self.shape.get_tile_y().ok()?;
        let mut payload = Vec::new();
        { let mut writer = LegacyWriter::new(&mut payload);
          writer.write_i32(METEOR_ARROW_SKILL_ID as i32); writer.write_i32(self.skill_level);
          writer.write_i32(x); writer.write_i32(y); writer.write_u32(remained);
          writer.write_i32(METEOR_ARROW_SKILL_ID as i32); writer.write_u32(self.frequency_ms);
          writer.write_i32(i32::try_from(self.cells.len()).ok()?);
          for &(cell_x, cell_y) in &self.cells { writer.write_i32(cell_x); writer.write_i32(cell_y); } }
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}

pub(crate) fn calculate_meteor_arrow_attack(game: &mut CGame, phalanx: &CMeteorArrowPhalanx)
    -> (AttackInformation, PlayerCombatProperties, u8, u8) {
    let delta = phalanx.maximum_attack().wrapping_sub(phalanx.minimum_attack());
    let width = if delta < 0 { delta.wrapping_neg() } else { delta }.wrapping_add(1);
    let physical = phalanx.minimum_attack().wrapping_add(game.skill_random_below(width)).max(0);
    let master = phalanx.master();
    let mut attack = AttackInformation { skill_id: METEOR_ARROW_SKILL_ID, skill_level: phalanx.skill_level() as u8,
        attacker_type: master.master_type, attacker_id: master.master_id, attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id,
        hit_modifier: phalanx.hit_modifier(), damage_factor: 1.0, damage_modifier: 0, critical: false,
        blast_attack: false, full_miss: 0, damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: phalanx.element_attack().max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: phalanx.soul_attack().max(0), mp_damage: 0 },
        ] };
    if game.skill_random_below(100) < phalanx.critical_chance() { attack.critical = true; let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages { power.hp_damage = (power.hp_damage as f32 * rate).round_ties_even() as i32; } }
    let combat = game.find_player(master.master_id).map_or_else(PlayerCombatProperties::default, |player| player.combat_properties());
    let occupation = game.find_player(master.master_id).map_or(0, |player| player.occupation());
    let level = game.find_player(master.master_id).map_or(0, |player| player.level());
    (attack, combat, occupation, level)
}
