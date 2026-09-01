//! Проникающий поклеточный навык `CBloodRose` (`0xD0`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/bloodrose.cpp`. После поздней проверки арбалета и
//! необратимого расхода MP снаряд проходит не более одной клетки за AI-tick.
//! На первой занятой клетке он обходит квадрат 3×3 в исходном порядке,
//! поражает каждую фигуру не более одного раза за полёт, отправляет end-пакет
//! и после срока пути отбрасывает самого стрелка назад. Формула каждого нового
//! попадания выполняет ровно два RNG-вызова; защитные RNG остаются у `CGame`.
//! `Attack` не изнашивает оружие на отдельных целях: унаследованный
//! `AfterUseSkill` делает это один раз из `End(true)`, после чего обновляются
//! свойства и cooldown. `End(false)` только освобождает runtime-состояние и не
//! откатывает попадания.
//! Критический множитель переводится в `int` с подтверждённым x87 усечением
//! к нулю отдельно для каждого боевого компонента.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use super::poisonmoth::{MONSTER_TYPE, PLAYER_TYPE, cell_targets, master_info, target_level, target_position, weapon_is_crossbow};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::summonskill::{abort_skill, finish_summon_skill};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const BLOOD_ROSE_SKILL_ID: u32 = 0xd0;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const USER_MP_LOSE: u32 = 2;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const MISSILE_FLYING_TIME: u32 = 10_008;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const ADDITION_ELEMENT_ATTACK: u32 = 20_013;
const TARGET_BACK_STEP: u32 = 30_002;
const TARGET_MOVE_SPEED: u32 = 30_003;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BloodRoseExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    condition_checked: bool,
    attacking_started: bool,
    path: Vec<(i32, i32, u8)>,
    current_position: usize,
    destination: (i32, i32),
    end_tile: (i32, i32),
    visual_target: Option<ShapeIdentity>,
    attacked: Vec<ShapeIdentity>,
    end_sent: bool,
}

impl BloodRoseExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32, destination: (i32, i32)) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started_at_ms), condition_checked: false, attacking_started: false, path: Vec::new(), current_position: 0, destination, end_tile: (0, 0), visual_target: None, attacked: Vec::new(), end_sent: false }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
pub(crate) fn is_blood_rose_dispatch(dispatch: PlayerSkillDispatch) -> bool { matches!(dispatch, PlayerSkillDispatch::SelfTarget { skill_id: BLOOD_ROSE_SKILL_ID, .. } | PlayerSkillDispatch::Point { skill_id: BLOOD_ROSE_SKILL_ID, .. } | PlayerSkillDispatch::Object { skill_id: BLOOD_ROSE_SKILL_ID, .. }) }
fn restore_player_movement(game: &mut CGame, player_id: i32) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } }
fn finish_player_blood_rose<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    game.damage_player_weapon(player_id, runtime);
    finish_summon_skill(game, player_id, ai, runtime, |ai, now_ms| ai.mark_blood_rose_used(now_ms));
}
fn abort_player_blood_rose(game: &mut CGame, player_id: i32) { restore_player_movement(game, player_id); abort_skill(game, player_id); }
pub(crate) fn complete_player_blood_rose<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.blood_rose().map(|state| state.kernel().dispatch()) else { return false };
    finish_player_blood_rose(game, player_id, ai, runtime);
    ai.finish_player_skill(dispatch, SkillTermination::Completed)
}
pub(crate) fn cancel_player_blood_rose<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.blood_rose().map(|state| state.kernel().dispatch()) else { return false };
    abort_player_blood_rose(game, player_id);
    ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}
fn send_failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code { 7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss), 10 => game.send_skill_system_info(player_id, b"GS0286"), 0x0b => game.send_skill_system_info(player_id, b"GS0290"), 0x0d => game.send_skill_system_info(player_id, b"GS0278"), 0x0e => game.send_skill_system_info(player_id, b"GS0293"), _ => {} }
}
fn send_start(game: &mut CGame, player_id: i32, level: i32) {
    let Some(player) = game.find_player(player_id) else { return }; let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(1); message.add_long(BLOOD_ROSE_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(player.shape().get_direction()); let _ = game.send_player_shape_around(player_id, None, &message);
}
fn send_fire(game: &mut CGame, region_id: i32, player_id: i32, level: i32, dispatch: PlayerSkillDispatch, destination: (i32, i32), flying_time: u32) {
    let live_target = match dispatch { PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|view| (target, view.tile_x, view.tile_y)), _ => None };
    let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(2); message.add_long(BLOOD_ROSE_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(live_target.map_or(0, |value| value.0.object_type)); message.add_long(live_target.map_or(0, |value| value.0.id)); message.add_long(live_target.map_or(destination.0, |value| value.1)); message.add_long(live_target.map_or(destination.1, |value| value.2)); message.add_ulong(flying_time); let _ = game.send_player_shape_around(player_id, None, &message);
}
fn send_end(game: &mut CGame, player_id: i32, level: i32, end_tile: (i32, i32), target: Option<ShapeIdentity>) {
    let Some(player) = game.find_player(player_id) else { return }; let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(3); message.add_long(BLOOD_ROSE_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(player.shape().get_direction()); message.add_long(end_tile.0); message.add_long(end_tile.1); message.add_long(target.map_or(0, |value| value.object_type)); message.add_long(target.map_or(0, |value| value.id)); let _ = game.send_player_shape_around(player_id, None, &message);
}

fn calculate_attack(game: &mut CGame, player_id: i32, target_level: u8, level: i32, damage_factor: u32, hit_modifier: i32, element_addition: u32) -> Option<(MasterInfo, AttackInformation)> {
    let player = game.find_player(player_id)?; let combat = player.combat_properties(); let master = master_info(player);
    let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors(); let weapon_factor = player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, minimum_factor);
    let width = (combat.maximum_attack as i32).wrapping_sub(combat.minimum_attack as i32).wrapping_add(1); let physical = (combat.minimum_attack as i32).wrapping_add(game.skill_random_below(width)).max(0);
    let mut attack = AttackInformation { skill_id: BLOOD_ROSE_SKILL_ID, skill_level: level as u8, attacker_type: PLAYER_TYPE, attacker_id: player_id, attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id, hit_modifier, damage_factor: damage_factor as f32 * weapon_factor * 0.01, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 }, AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).wrapping_add(element_addition as i32).max(0), mp_damage: 0 }, AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 }] };
    if game.skill_random_below(100) < i32::from(combat.blast_attack) { attack.critical = true; let rate = game.globe_setup().critical_rate(); for power in &mut attack.damages { power.hp_damage = (power.hp_damage as f32 * rate) as i32; } } Some((master, attack))
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет порядок области, уникальность целей и применение попаданий")]
fn attack_scope<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, region_id: i32, level: i32, damage_factor: u32, hit_modifier: i32, element_addition: u32, center_x: i32, center_y: i32, attacked: &mut Vec<ShapeIdentity>, runtime: &mut Runtime) -> (bool, Option<ShapeIdentity>) {
    let Some(master) = game.find_player(player_id).map(master_info) else { return (false, None) }; let mut any = false; let mut visual_target = None;
    for offset_x in -1..=1 { for offset_y in -1..=1 { let cell_x = center_x.wrapping_add(offset_x); let cell_y = center_y.wrapping_add(offset_y); for target in cell_targets(game, region_id, cell_x, cell_y) { if (target.object_type == PLAYER_TYPE && target.id == player_id) || !matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) || !game.owned_player_skill_target_attackable(master, target, region_id) { continue } any = true; if cell_x == center_x && cell_y != 0 && visual_target.is_none() { visual_target = Some(target); } if attacked.contains(&target) { continue } attacked.push(target); let Some(target_level) = target_level(game, region_id, target) else { continue }; let Some((master, attack)) = calculate_attack(game, player_id, target_level, level, damage_factor, hit_modifier, element_addition) else { continue }; match target.object_type { PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime), MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime), _ => continue } } } }
    (any, visual_target)
}

fn knock_back_owner(game: &mut CGame, player_id: i32, region_id: i32, back_steps: u32, move_speed: u32) {
    let Some((direction, mut position)) = game.find_player(player_id).and_then(|player| Some(((player.shape().get_direction().wrapping_add(4)) & 7, ShapeAreaCoordinates { x: player.shape().get_tile_x().ok()?, y: player.shape().get_tile_y().ok()? }))) else { return }; let original = position; let mut moved = 0;
    while moved < back_steps { let Ok(next) = CShape::get_direction_position(direction, position) else { break }; let blocked = game.find_region(region_id).is_none_or(|owner| owner.base().region.get_block(next.x, next.y).map_or(true, |block| block != 0)); if blocked { break } position = next; moved = moved.wrapping_add(1); }
    if position != original { let _ = game.force_move_player(player_id, position.x, position.y, move_speed.wrapping_mul(moved)); }
}

pub(crate) fn execute_player_blood_rose<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_blood_rose_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) } let Some((region_id, source_x, source_y, level, initial_mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(BLOOD_ROSE_SKILL_ID), player.mana()))) else { return terminal(QueuedSkillExecutionState::Rejected) }; let Some(properties) = game.skill_base_properties(BLOOD_ROSE_SKILL_ID, level) else { if player_ai.blood_rose().is_some() { abort_player_blood_rose(game, player_id); } return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE); let reuse_delay = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let max_distance = properties.query_property(TARGET_MAX_DISTANCE); let cell_delay = properties.query_property(MISSILE_FLYING_TIME); let damage_factor = properties.query_property(TARGET_DAMAGE_FACTOR); let hit_modifier = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32; let element_addition = properties.query_property(ADDITION_ELEMENT_ATTACK); let back_steps = properties.query_property(TARGET_BACK_STEP); let move_speed = properties.query_property(TARGET_MOVE_SPEED); let _can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if player_ai.blood_rose().is_none() { let now = runtime.now_milliseconds(); let self_target = match dispatch { PlayerSkillDispatch::SelfTarget { .. } => true, PlayerSkillDispatch::Object { target, .. } => target.object_type == PLAYER_TYPE && target.id == player_id, _ => false }; if self_target { send_failure(game, player_id, 10, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) } if player_ai.blood_rose_last_used_ms() != 0 && !time_reached(now, player_ai.blood_rose_last_used_ms(), reuse_delay) { send_failure(game, player_id, 0x0d, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) } let Some(destination) = target_position(game, region_id, player_id, dispatch) else { return terminal(QueuedSkillExecutionState::Rejected) }; let path = game.base_magic_path(region_id, source_x, source_y, destination.0, destination.1, None); if max_distance != 0 && path.len() as u32 > max_distance { send_failure(game, player_id, 0x0b, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) } let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected) }; if !weapon_is_crossbow(game, player) { send_failure(game, player_id, 0x0e, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) } if mp_loss != 0 && (initial_mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, 7, mp_loss); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(BLOOD_ROSE_SKILL_ID)); } player_ai.begin_blood_rose(BloodRoseExecutionState::begin(dispatch, now, destination)); }
    else if player_ai.blood_rose().is_none_or(|state| state.kernel.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    if player_ai.blood_rose().is_some_and(|state| !state.condition_checked) { let mana = game.find_player(player_id).map_or(0, CPlayer::mana); if (mana.wrapping_sub(mp_loss) as i32) < 0 { send_failure(game, player_id, 7, mp_loss); abort_player_blood_rose(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp_loss)); } let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); if game.find_player(player_id).is_none_or(|player| !weapon_is_crossbow(game, player)) { send_failure(game, player_id, 0x0e, mp_loss); abort_player_blood_rose(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) } let destination = player_ai.blood_rose().map(|state| state.destination).unwrap_or_default(); if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, destination.0, destination.1)); } send_start(game, player_id, level); if let Some(state) = player_ai.blood_rose_mut() { state.condition_checked = true; let _ = state.kernel.advance(SkillStage::Begin, SkillStage::Check); } }
    let started = player_ai.blood_rose().map(|state| state.kernel.started_at_ms()).unwrap_or_default(); if !player_ai.blood_rose().is_some_and(|state| state.attacking_started) { if !time_reached(runtime.now_milliseconds(), started, delay) { return terminal(QueuedSkillExecutionState::Pending) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } let Some(destination) = target_position(game, region_id, player_id, dispatch) else { abort_player_blood_rose(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) }; let path = game.base_magic_path(region_id, source_x, source_y, destination.0, destination.1, (max_distance != 0).then_some(max_distance)); let index = path.iter().position(|cell| cell.2 == 2).unwrap_or(path.len()); let endpoint = path.get(index).or_else(|| path.last()).copied().unwrap_or((destination.0, destination.1, 2)); send_fire(game, region_id, player_id, level, dispatch, destination, cell_delay.wrapping_mul(index as u32)); if let Some(state) = player_ai.blood_rose_mut() { state.path = path; state.current_position = 1; state.end_tile = (endpoint.0, endpoint.1); state.visual_target = None; state.attacking_started = true; let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate); let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack); } }
    let Some((position, path_len, cell, end_sent)) = player_ai.blood_rose().map(|state| (state.current_position, state.path.len(), state.path.get(state.current_position).copied(), state.end_sent)) else { return terminal(QueuedSkillExecutionState::Rejected) }; if !time_reached(runtime.now_milliseconds(), started, delay.wrapping_add(cell_delay.wrapping_mul(position as u32))) { return terminal(QueuedSkillExecutionState::Pending) }
    let Some((x, y, _)) = cell else { knock_back_owner(game, player_id, region_id, back_steps, move_speed); if let Some(state) = player_ai.blood_rose_mut() { let _ = state.kernel.advance(SkillStage::Attack, SkillStage::Apply); } finish_player_blood_rose(game, player_id, player_ai, runtime); return terminal(QueuedSkillExecutionState::Completed) }; let live_block = game.find_region(region_id).map_or(2, |owner| owner.base().skill_cell_block(x, y)); if let Some(state) = player_ai.blood_rose_mut() { state.end_tile = (x, y) }
    if live_block == 3 { let mut attacked = player_ai.blood_rose_mut().map(|state| std::mem::take(&mut state.attacked)).unwrap_or_default(); let (any, visual) = attack_scope(game, player_id, region_id, level, damage_factor, hit_modifier, element_addition, x, y, &mut attacked, runtime); if let Some(state) = player_ai.blood_rose_mut() { state.attacked = attacked; if state.visual_target.is_none() { state.visual_target = visual; } } if any { let target = player_ai.blood_rose().and_then(|state| state.visual_target); send_end(game, player_id, level, (x, y), target); if let Some(state) = player_ai.blood_rose_mut() { state.end_sent = true; state.current_position = path_len.wrapping_add(1); } return terminal(QueuedSkillExecutionState::Pending) } }
    else if live_block == 2 { if !end_sent { send_end(game, player_id, level, (x, y), None); } if let Some(state) = player_ai.blood_rose_mut() { state.end_sent = true; state.current_position = path_len; } }
    if let Some(state) = player_ai.blood_rose_mut() { state.current_position = state.current_position.wrapping_add(1); } terminal(QueuedSkillExecutionState::Pending)
}
