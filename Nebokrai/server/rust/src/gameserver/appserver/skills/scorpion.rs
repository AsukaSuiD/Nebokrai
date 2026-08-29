//! Трёхударный навык `CScorpion` (`0xD1`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/scorpion.cpp`. После необратимого расхода MP навык заново
//! проверяет арбалет, дальность и весь прямой путь. Первый, второй и третий
//! удары выполняются в разные AI-tick на накопленных сроках; только третий
//! применяет skill damage factor. Каждый удар заново читает боевые свойства и
//! выполняет ровно два RNG-вызова. `CGame` оставляет применение к независимому
//! владельцу цели, износ оружия и сетевую доставку.
//! `End(false)` после восстановления движения отправляет action `3`, тогда как
//! `End(true)` обновляет свойства и cooldown без этого завершающего пакета.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use super::poisonmoth::{MONSTER_TYPE, PLAYER_TYPE, master_info, target_level, weapon_is_crossbow};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_DAMAGE_LEVEL;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::CMonster;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::summonskill::{abort_skill, finish_summon_skill};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const SCORPION_SKILL_ID: u32 = 0xd1;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const USER_MP_LOSE: u32 = 2;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const FIRST_TIME: u32 = 15_001;
const SECOND_TIME: u32 = 15_002;
const THIRD_TIME: u32 = 15_003;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScorpionExecutionState { kernel: SkillExecutionKernel<PlayerSkillDispatch>, condition_checked: bool, attacks: u8 }
impl ScorpionExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, now: u32) -> Self { Self { kernel: SkillExecutionKernel::begin(dispatch, now), condition_checked: false, attacks: 0 } }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}
fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
pub(crate) fn is_scorpion_dispatch(dispatch: PlayerSkillDispatch) -> bool { matches!(dispatch, PlayerSkillDispatch::SelfTarget { skill_id: SCORPION_SKILL_ID, .. } | PlayerSkillDispatch::Point { skill_id: SCORPION_SKILL_ID, .. } | PlayerSkillDispatch::Object { skill_id: SCORPION_SKILL_ID, .. }) }
fn restore_player_movement(game: &mut CGame, player_id: i32) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } }
fn target(dispatch: PlayerSkillDispatch) -> Option<ShapeIdentity> { match dispatch { PlayerSkillDispatch::Object { target, .. } => Some(target), _ => None } }
pub(super) fn target_snapshot(game: &CGame, region_id: i32, identity: ShapeIdentity) -> Option<(i32, i32, bool)> { match identity.object_type { PLAYER_TYPE => game.find_player(identity.id).filter(|player| player.server_region_id() == Some(region_id)).and_then(|player| Some((player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.is_dead()))), MONSTER_TYPE => game.find_region(region_id).and_then(|owner| { let monster = owner.base().find_monster_by_id(identity.id)?; Some((monster.move_shape().shape().get_tile_x().ok()?, monster.move_shape().shape().get_tile_y().ok()?, monster.hit_points() == 0)) }), _ => None } }
pub(super) fn target_name<'a>(game: &'a CGame, region_id: i32, identity: ShapeIdentity) -> &'a [u8] { match identity.object_type { PLAYER_TYPE => game.find_player(identity.id).map(CPlayer::player_name).unwrap_or_default(), MONSTER_TYPE => game.find_region(region_id).and_then(|owner| owner.base().find_monster_by_id(identity.id)).map(CMonster::display_name).unwrap_or_default(), _ => &[] } }
fn send_failure(game: &CGame, player_id: i32, code: u8, mp: u32, text: Option<&[u8]>) { game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code); match code { 7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp), 10 => game.send_skill_system_info(player_id, text.unwrap_or(b"GS0286")), 0x0b => game.send_skill_system_info(player_id, b"GS0290"), 0x0d => game.send_skill_system_info(player_id, b"GS0278"), 0x0e => game.send_skill_system_info(player_id, b"GS0293"), 0x0f => game.send_skill_system_info_with_text(player_id, b"GS0296", text.unwrap_or_default()), _ => {} } }
fn send_start(game: &mut CGame, player_id: i32, level: i32) { let Some(player) = game.find_player(player_id) else { return }; let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(1); message.add_long(SCORPION_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(player.shape().get_direction()); let _ = game.send_player_shape_around(player_id, None, &message); }
fn send_strike(game: &mut CGame, player_id: i32, level: i32, identity: ShapeIdentity, position: (i32, i32), second: u32, third: u32) { let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(2); message.add_long(SCORPION_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(position.0); message.add_long(position.1); message.add_long(0); message.add_ulong(second); message.add_ulong(second.wrapping_add(third)); let _ = game.send_player_shape_around(player_id, None, &message); }
fn send_end(game: &mut CGame, player_id: i32, level: i32) { let Some(player) = game.find_player(player_id) else { return }; let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(3); message.add_long(SCORPION_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(player.shape().get_direction()); let _ = game.send_player_shape_around(player_id, None, &message); }
fn finish_player_scorpion<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_summon_skill(game, player_id, ai, runtime, |ai, now_ms| ai.mark_scorpion_used(now_ms));
}
fn abort_player_scorpion(game: &mut CGame, player_id: i32, level: i32) { restore_player_movement(game, player_id); send_end(game, player_id, level); abort_skill(game, player_id); }
pub(crate) fn complete_player_scorpion<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.scorpion().map(|state| state.kernel().dispatch()) else { return false };
    finish_player_scorpion(game, player_id, ai, runtime);
    ai.finish_player_skill(dispatch, SkillTermination::Completed)
}
pub(crate) fn cancel_player_scorpion<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.scorpion().map(|state| state.kernel().dispatch()) else { return false };
    let level = game.find_player(player_id).map_or(0, |player| player.learned_skill_level(SCORPION_SKILL_ID));
    abort_player_scorpion(game, player_id, level);
    ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}
fn reject(game: &mut CGame, player_id: i32, level: i32, code: Option<(u8, u32, Option<&[u8]>)>) -> QueuedSkillExecutionOutcome { if let Some((code, mp, text)) = code { send_failure(game, player_id, code, mp, text); } abort_player_scorpion(game, player_id, level); terminal(QueuedSkillExecutionState::Rejected) }

fn calculate_attack(game: &mut CGame, player_id: i32, region_id: i32, identity: ShapeIdentity, level: i32, hit_modifier: i32, final_factor: Option<u32>) -> Option<(MasterInfo, AttackInformation)> {
    let target_level = target_level(game, region_id, identity)?; let player = game.find_player(player_id)?; let combat = player.combat_properties(); let master = master_info(player); let weapon_level = player.equipment().get_goods(2).map_or(0, |weapon| weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_DAMAGE_LEVEL, 1)); let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors(); let delta = weapon_level.wrapping_sub(i32::from(target_level)).max(0); let weapon_factor = if divisor == 0.0 { 1.0 } else { (delta as f32 / divisor).min(1.0).max(minimum_factor) }; let width = (combat.maximum_attack as i32).wrapping_sub(combat.minimum_attack as i32).wrapping_abs().wrapping_add(1); let physical = (combat.minimum_attack as i32).wrapping_add(game.skill_random_below(width)).max(0); let damage_factor = final_factor.map_or(weapon_factor, |factor| factor as f32 * weapon_factor * 0.01);
    let mut attack = AttackInformation { skill_id: SCORPION_SKILL_ID, skill_level: level as u8, attacker_type: PLAYER_TYPE, attacker_id: player_id, attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id, hit_modifier, damage_factor, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 }, AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 }, AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 }] }; if game.skill_random_below(100) < i32::from(combat.blast_attack) { attack.critical = true; let rate = game.globe_setup().critical_rate(); for power in &mut attack.damages { power.hp_damage = (power.hp_damage as f32 * rate).round_ties_even() as i32; } } Some((master, attack))
}
fn apply_attack<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, region_id: i32, identity: ShapeIdentity, level: i32, hit_modifier: i32, factor: Option<u32>, runtime: &mut Runtime) { let Some((master, attack)) = calculate_attack(game, player_id, region_id, identity, level, hit_modifier, factor) else { return }; match identity.object_type { PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, identity.id, region_id, attack, runtime), MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, identity.id, region_id, attack, runtime), _ => return } game.damage_player_weapon(player_id, runtime); }

pub(crate) fn execute_player_scorpion<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_scorpion_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) } let Some((region_id, source_x, source_y, level, initial_mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(SCORPION_SKILL_ID), player.mana()))) else { return terminal(QueuedSkillExecutionState::Rejected) }; let Some(properties) = game.skill_base_properties(SCORPION_SKILL_ID, level) else { return reject(game, player_id, level, None) }; let mp = properties.query_property(USER_MP_LOSE); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let first = properties.query_property(FIRST_TIME); let second = properties.query_property(SECOND_TIME); let third = properties.query_property(THIRD_TIME); let max_distance = properties.query_property(TARGET_MAX_DISTANCE); let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32; let final_factor = properties.query_property(TARGET_DAMAGE_FACTOR); let _can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if player_ai.scorpion().is_none() { let now = runtime.now_milliseconds(); let Some(identity) = target(dispatch) else { return reject(game, player_id, level, Some((10, mp, Some(b"GS0286")))) }; if player_ai.scorpion_last_used_ms() != 0 && !time_reached(now, player_ai.scorpion_last_used_ms(), reuse) { return reject(game, player_id, level, Some((0x0d, mp, None))) } let Some((target_x, target_y, _)) = target_snapshot(game, region_id, identity) else { return reject(game, player_id, level, Some((10, mp, Some(b"GS0286")))) }; let path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None); if max_distance != 0 && path.len() as u32 > max_distance { return reject(game, player_id, level, Some((0x0b, mp, None))) } let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) }; if !weapon_is_crossbow(game, player) { return reject(game, player_id, level, Some((0x0e, mp, None))) } if mp != 0 && (initial_mana.wrapping_sub(mp) as i32) < 0 { return reject(game, player_id, level, Some((7, mp, None))) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(SCORPION_SKILL_ID)); } player_ai.begin_scorpion(ScorpionExecutionState::begin(dispatch, now)); }
    else if player_ai.scorpion().is_none_or(|state| state.kernel.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let identity = target(dispatch).expect("активный Scorpion сохраняет объектную цель"); let Some((target_x, target_y, dead)) = target_snapshot(game, region_id, identity) else { return reject(game, player_id, level, Some((10, mp, Some(b"GS0285")))) }; if dead { return reject(game, player_id, level, Some((10, mp, Some(b"GS0285")))) } if identity.object_type == PLAYER_TYPE && identity.id == player_id { return reject(game, player_id, level, Some((10, mp, Some(b"GS0286")))) }
    if player_ai.scorpion().is_some_and(|state| !state.condition_checked) { let mana = game.find_player(player_id).map_or(0, CPlayer::mana); if (mana.wrapping_sub(mp) as i32) < 0 { return reject(game, player_id, level, Some((7, mp, None))) } if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp)); } let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); if game.find_player(player_id).is_none_or(|player| !weapon_is_crossbow(game, player)) { return reject(game, player_id, level, Some((0x0e, mp, None))) } if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y)); } let path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None); if max_distance != 0 && path.len() as u32 > max_distance { return reject(game, player_id, level, Some((0x0b, mp, None))) } if path.iter().any(|cell| cell.2 == 2) { let name = target_name(game, region_id, identity).to_vec(); return reject(game, player_id, level, Some((0x0f, mp, Some(&name)))) } send_start(game, player_id, level); if let Some(state) = player_ai.scorpion_mut() { state.condition_checked = true; let _ = state.kernel.advance(SkillStage::Begin, SkillStage::Check); } }
    let started = player_ai.scorpion().map(|state| state.kernel.started_at_ms()).unwrap_or_default(); let attacks = player_ai.scorpion().map_or(0, |state| state.attacks); let deadline = match attacks { 0 => delay.wrapping_add(first), 1 => delay.wrapping_add(first).wrapping_add(second), _ => delay.wrapping_add(first).wrapping_add(second).wrapping_add(third) }; if !time_reached(runtime.now_milliseconds(), started, deadline) { return terminal(QueuedSkillExecutionState::Pending) }
    if attacks == 0 { send_strike(game, player_id, level, identity, (target_x, target_y), second, third); apply_attack(game, player_id, region_id, identity, level, hit_modifier, None, runtime); if let Some(state) = player_ai.scorpion_mut() { state.attacks = 1; let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate); let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack); } return terminal(QueuedSkillExecutionState::Pending) }
    if attacks == 1 { apply_attack(game, player_id, region_id, identity, level, hit_modifier, None, runtime); if let Some(state) = player_ai.scorpion_mut() { state.attacks = 2; } return terminal(QueuedSkillExecutionState::Pending) }
    apply_attack(game, player_id, region_id, identity, level, hit_modifier, Some(final_factor), runtime); if let Some(state) = player_ai.scorpion_mut() { let _ = state.kernel.advance(SkillStage::Attack, SkillStage::Apply); } finish_player_scorpion(game, player_id, player_ai, runtime); terminal(QueuedSkillExecutionState::Completed)
}
