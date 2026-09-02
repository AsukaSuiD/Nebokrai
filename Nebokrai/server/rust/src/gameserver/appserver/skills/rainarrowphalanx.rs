//! Трёхлучевая летящая область `CRainArrowPhalanx` (`0xCE`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/rainarrowphalanx.cpp`. Владелец хранит три пути и
//! прекращает каждый из них после первой клетки хотя бы с одной допустимой
//! целью; внутри клетки сохраняется порядок `GetShape` и атакуются все цели.
//! Формула читает живой боевой снимок стрелка и сохраняет два RNG-вызова.
//! Знаковый процент коэффициента умножается через `FIMUL` в x87 на оружейный
//! `f32`, и только итог сохраняется как `f32`.
//! Критический множитель вычисляется в расширенной точности x87 и усекается к
//! нулю при записи результата в `i32`.

use super::fightdefense::truncate_original;
use crate::gameserver::appserver::legacycodec::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

pub(crate) const RAIN_ARROW_SKILL_ID: u32 = 0xce;
pub(crate) type RainArrowCell = (i32, i32, u8);

#[derive(Clone, Copy, Debug, Eq, PartialEq)] pub(crate) enum RainArrowBeam { Center, Right, Left }
#[derive(Clone, Debug, Eq, PartialEq)] pub(crate) enum RainArrowPhalanxTick {
    Pending, Attack { cells: Vec<(RainArrowBeam, i32, i32)>, sampled_at_ms: u32 }, Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CRainArrowPhalanx {
    shape: CShape, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
    skill_level: i32, hit_modifier: i32, damage_factor_percent: i32,
    left_path: Vec<RainArrowCell>, center_path: Vec<RainArrowCell>, right_path: Vec<RainArrowCell>,
    left_cells: usize, center_cells: usize, right_cells: usize,
    current_step: u32, speed_ms: i32, maximum_distance: i32,
}

impl CRainArrowPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, hit_modifier: i32, damage_factor_percent: i32,
        left_path: Vec<RainArrowCell>, left_cells: usize, center_path: Vec<RainArrowCell>, center_cells: usize,
        right_path: Vec<RainArrowCell>, right_cells: usize, speed_ms: i32, maximum_distance: i32) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self { shape, master, started_at_ms, lifetime_ms, skill_level, hit_modifier, damage_factor_percent,
            left_path, center_path, right_path, left_cells, center_cells, right_cells,
            current_step: 1, speed_ms, maximum_distance }
    }
    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn skill_level(&self) -> i32 { self.skill_level }
    pub(crate) const fn hit_modifier(&self) -> i32 { self.hit_modifier }
    pub(crate) const fn damage_factor_percent(&self) -> i32 { self.damage_factor_percent }
    pub(crate) fn finish(&mut self) { self.shape.set_change_state(SHAPE_CHANGE_DELETE); }
    pub(crate) fn stop_beam(&mut self, beam: RainArrowBeam) { match beam {
        RainArrowBeam::Center => self.center_cells = 0, RainArrowBeam::Right => self.right_cells = 0,
        RainArrowBeam::Left => self.left_cells = 0,
    }}
    pub(crate) fn tick(&mut self, lifetime_now_ms: u32, attack_now_ms: impl FnOnce() -> u32) -> RainArrowPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < lifetime_now_ms
            || (self.maximum_distance as u32) < self.current_step { self.finish(); return RainArrowPhalanxTick::Expired }
        let now_ms = attack_now_ms();
        let due = self.started_at_ms.wrapping_add((self.speed_ms as u32).wrapping_mul(self.current_step));
        if now_ms < due { return RainArrowPhalanxTick::Pending }
        let index = self.current_step.wrapping_sub(1) as usize; let mut cells = Vec::with_capacity(3);
        if self.current_step < self.center_cells as u32 + 1 && let Some(&(x, y, _)) = self.center_path.get(index) { cells.push((RainArrowBeam::Center, x, y)); }
        if self.current_step < self.right_cells as u32 + 1 && let Some(&(x, y, _)) = self.right_path.get(index) { cells.push((RainArrowBeam::Right, x, y)); }
        if self.current_step < self.left_cells as u32 + 1 && let Some(&(x, y, _)) = self.left_path.get(index) { cells.push((RainArrowBeam::Left, x, y)); }
        self.current_step = self.current_step.wrapping_add(1);
        RainArrowPhalanxTick::Attack { cells, sampled_at_ms: now_ms }
    }
    pub(crate) fn encode_client_snapshot(&self, mut now_milliseconds: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let first_now = now_milliseconds(); let remained = if self.started_at_ms.wrapping_add(self.lifetime_ms) <= first_now { 0 }
            else { self.lifetime_ms.wrapping_sub(now_milliseconds()).wrapping_add(self.started_at_ms) };
        let mut payload = Vec::new(); { let mut writer = LegacyWriter::new(&mut payload);
            writer.write_i32(RAIN_ARROW_SKILL_ID as i32); writer.write_i32(self.skill_level);
            writer.write_i32(self.shape.get_tile_x().ok()?); writer.write_i32(self.shape.get_tile_y().ok()?); writer.write_u32(remained); }
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}

pub(crate) fn calculate_rain_arrow_attack(game: &mut CGame, phalanx: &CRainArrowPhalanx, target_level: u8)
    -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let player = game.find_player(phalanx.master().master_id)?; let mut combat = player.combat_properties();
    let occupation = player.occupation(); let attacker_level = player.level();
    let (divisor, floor) = game.globe_setup().weapon_damage_factors();
    let weapon_factor = player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, floor);
    let minimum = combat.minimum_attack as i32; let maximum = combat.maximum_attack as i32;
    let delta = maximum.wrapping_sub(minimum); let width = if delta < 0 { delta.wrapping_neg() } else { delta }.wrapping_add(1);
    let physical = minimum.wrapping_add(game.skill_random_below(width)); let master = phalanx.master();
    let mut attack = AttackInformation { skill_id: RAIN_ARROW_SKILL_ID, skill_level: phalanx.skill_level() as u8,
        attacker_type: master.master_type, attacker_id: master.master_id, attacker_team_id: master.master_team_id,
        attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id,
        hit_modifier: phalanx.hit_modifier(), damage_factor: (f64::from(weapon_factor)
            * f64::from(phalanx.damage_factor_percent())
            * f64::from(0.01_f32)) as f32,
        damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical.max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 }] };
    if game.skill_random_below(100) < i32::from(combat.cch) { attack.critical = true; let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages { power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate)); } }
    let [ba, bd, eba, ebd, fm] = game.globe_setup().base_combat_scales();
    if combat.blast_attack_scale() < 1.0 { combat.blast_attack_scale_bits = ba.max(1.0).to_bits(); }
    if combat.blast_defense_scale() < 0.01 { combat.blast_defense_scale_bits = bd.max(0.01).to_bits(); }
    if combat.element_blast_attack_scale() < 1.0 { combat.element_blast_attack_scale_bits = eba.max(1.0).to_bits(); }
    if combat.element_blast_defense_scale() < 0.01 { combat.element_blast_defense_scale_bits = ebd.max(0.01).to_bits(); }
    if combat.full_miss_scale() < 0.01 { combat.full_miss_scale_bits = fm.max(0.01).to_bits(); }
    if combat.critical_rate() < 1.0 { combat.critical_rate_bits = game.globe_setup().critical_rate().max(1.0).to_bits(); }
    Some((attack, combat, occupation, attacker_level))
}

// Декодирование серверного снимка пока не имеет достигнутого caller-а. Оно
// восстанавливает только базовую форму, но не три runtime-пути, поэтому его
// нельзя подменять объектом с выдуманными пустыми лучами.
//
// FUNCTION: CRainArrowPhalanx::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\rainarrowphalanx.cpp:269
// RVA: 0x001F9FA0
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
