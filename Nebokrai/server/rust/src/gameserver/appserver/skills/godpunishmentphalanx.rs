//! Одноклеточная область `CGodPunishmentPhalanx` (`0x13A`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/godpunishmentphalanx.cpp`. До строгого истечения срока
//! область просматривает одну клетку и после применения помечается на удаление.
//! Текущие боевые свойства владельца читаются при атаке; формула сохраняет два
//! вызова legacy RNG. Не достигнут только DB/wire decoder восстановленной формы.
//! Критический множитель применяется в расширенной точности x87 и усекается к
//! нулю при записи результата в `i32`.

use super::fightdefense::truncate_original;
use super::godpunishment::GOD_PUNISHMENT_SKILL_ID;
use crate::gameserver::appserver::legacycodec::LegacyWriter;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

#[derive(Clone, Copy, Debug, Eq, PartialEq)] pub(crate) enum GodPunishmentPhalanxTick { Scan { sampled_at_ms: u32 }, Expired }
#[derive(Clone, Debug, Eq, PartialEq)] pub(crate) struct CGodPunishmentPhalanx { shape: CShape, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32, skill_level: i32, minimum_attack: i32, maximum_attack: i32, element_modifier: i32 }


pub(crate) fn calculate_owned_god_punishment_attack(game: &mut CGame, phalanx: &CGodPunishmentPhalanx, target_level: u8) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    let player = game.find_player(phalanx.master().master_id)?;
    let combat = player.combat_properties();
    let occupation = player.occupation();
    let level = player.level();
    let (divisor, minimum) = game.globe_setup().weapon_damage_factors();
    let factor = player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, minimum);
    let critical = game.globe_setup().critical_rate();
    Some(phalanx.calculate_attack(factor, combat, occupation, level, critical, &mut |maximum| game.skill_random_below(maximum)))
}
impl CGodPunishmentPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32, skill_level: i32, minimum_attack: i32, maximum_attack: i32, element_modifier: i32) -> Self { let mut shape = CShape::with_constructor_defaults(); shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID }); Self { shape, master, started_at_ms, lifetime_ms, skill_level, minimum_attack, maximum_attack, element_modifier } }
    pub(crate) const fn shape(&self) -> &CShape { &self.shape } pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape } pub(crate) const fn master(&self) -> MasterInfo { self.master } pub(crate) const fn skill_level(&self) -> i32 { self.skill_level }
    pub(crate) fn replace_affect_region(&mut self, _level: i32, tile_x: i32, tile_y: i32) { if self.shape.get_tile_x() == Ok(tile_x) && self.shape.get_tile_y() == Ok(tile_y) { self.finish(); } }
    pub(crate) fn finish(&mut self) { self.shape.set_change_state(SHAPE_CHANGE_DELETE); }
    pub(crate) fn tick(&mut self, now: u32) -> GodPunishmentPhalanxTick { if self.started_at_ms.wrapping_add(self.lifetime_ms) < now { self.finish(); GodPunishmentPhalanxTick::Expired } else { GodPunishmentPhalanxTick::Scan { sampled_at_ms: now } } }
    pub(crate) fn encode_client_snapshot(&self, mut now: impl FnMut() -> u32) -> Option<Vec<u8>> { let first = now(); let remained = if self.started_at_ms.wrapping_add(self.lifetime_ms) <= first { 0 } else { self.lifetime_ms.wrapping_sub(now()).wrapping_add(self.started_at_ms) }; let mut payload = Vec::new(); { let mut writer = LegacyWriter::new(&mut payload); writer.write_i32(GOD_PUNISHMENT_SKILL_ID as i32); writer.write_i32(self.skill_level); writer.write_i32(self.shape.get_tile_x().ok()?); writer.write_i32(self.shape.get_tile_y().ok()?); writer.write_u32(remained); } self.shape.add_to_byte_array(&mut payload, true).then_some(payload) }
    pub(crate) fn calculate_attack(&self, target_factor: f32, combat: PlayerCombatProperties, occupation: u8, level: u8, critical_rate: f32, random: &mut dyn FnMut(i32) -> i32) -> (AttackInformation, PlayerCombatProperties, u8, u8) { let width = self.maximum_attack.wrapping_sub(self.minimum_attack).wrapping_abs().wrapping_add(1); let scaled = self.element_modifier.wrapping_mul(combat.element_modify).wrapping_div(100); let damage = self.minimum_attack.wrapping_add(random(width)).wrapping_add(combat.add_element_attack as i32).wrapping_add(scaled).max(0); let mut attack = AttackInformation { skill_id: GOD_PUNISHMENT_SKILL_ID, skill_level: self.skill_level as u8, attacker_type: self.master.master_type, attacker_id: self.master.master_id, attacker_team_id: self.master.master_team_id, attacker_faction_id: self.master.master_guild_id, attacker_union_id: self.master.master_union_id, hit_modifier: 100, damage_factor: target_factor, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Element, hp_damage: damage, mp_damage: 0 }] }; if random(100) < i32::from(combat.cch) { attack.critical = true; for power in &mut attack.damages { power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(critical_rate)); } } (attack, combat, occupation, level) }
}

// ============================================================================
// FUNCTION: CGodPunishmentPhalanx::DecordFromByteArray
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\godpunishmentphalanx.cpp:189
// RVA: 0x002007C0
// ADDRESS: 006007c0
// PROTOTYPE: bool __thiscall DecordFromByteArray(uchar * param_1, long * param_2, bool param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
