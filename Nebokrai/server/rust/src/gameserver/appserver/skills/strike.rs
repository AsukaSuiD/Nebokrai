//! Оглушающий снаряд `CStrike` (`0x39`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/strike.cpp`. Навык дважды проверяет MP, сохраняет
//! необратимый расход до поздней проверки пути, вычисляет время полёта по
//! текущей длине пути, выполняет ровно два собственных RNG-вызова и лишь после
//! удара устанавливает выжившей цели канонический `Rush2State`. Длительность
//! состояния уменьшается по исходной разнице уровней; само состояние не
//! перемещает цель. `CGame` только разрешает владельцев, применяет готовый
//! результат и выполняет доставку. `Attack` и `AI` не изнашивают оружие в
//! точке удара: унаследованный `AfterUseSkill` делает это один раз из
//! `End(true)`, после атаки и установки состояния вместе с обновлением свойств
//! и cooldown. `End(false)` прекращает полёт без износа, не откатывая уже
//! выполненные эффекты.
//! Критический множитель переводится в `int` с подтверждённым x87 усечением
//! к нулю отдельно для каждого боевого компонента.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use super::poisonmoth::{MONSTER_TYPE, PLAYER_TYPE, master_info, target_level};
use super::rush::scaled_state_time;
use super::rushstate2::Rush2State;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::summonskill::{abort_skill, finish_summon_skill};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const STRIKE_SKILL_ID: u32 = 0x39;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const USER_MP_LOSE: u32 = 2;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const STATE_PERSIST_TIME: u32 = 10_002;
const MISSILE_FLYING_TIME: u32 = 10_008;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const BLOCK_UNFLY: u8 = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct StrikeExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    target: ShapeIdentity,
    condition_checked: bool,
    missile_flying_time_ms: u32,
    attacking_started: bool,
}

impl StrikeExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, target: ShapeIdentity, now_ms: u32) -> Self { Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), target, condition_checked: false, missile_flying_time_ms: 0, attacking_started: false } }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
pub(crate) const fn is_strike_dispatch(dispatch: PlayerSkillDispatch) -> bool { matches!(dispatch, PlayerSkillDispatch::Object { skill_id: STRIKE_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } }) }
fn restore_player_movement(game: &mut CGame, player_id: i32) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } }
fn finish_player_strike<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    game.damage_player_weapon(player_id, runtime);
    finish_summon_skill(game, player_id, ai, runtime, |ai, now_ms| ai.mark_strike_used(now_ms));
}
fn abort_player_strike(game: &mut CGame, player_id: i32) { restore_player_movement(game, player_id); abort_skill(game, player_id); }
pub(crate) fn complete_player_strike<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.strike().map(|state| state.kernel().dispatch()) else { return false };
    finish_player_strike(game, player_id, ai, runtime);
    ai.finish_player_skill(dispatch, SkillTermination::Completed)
}
pub(crate) fn cancel_player_strike<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.strike().map(|state| state.kernel().dispatch()) else { return false };
    abort_player_strike(game, player_id);
    ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}
fn fail(game: &CGame, player_id: i32, code: u8, mp_loss: u32, text: &[u8]) { game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code); if code == 7 { game.send_skill_system_info_with_unsigned(player_id, text, mp_loss); } else { game.send_skill_system_info(player_id, text); } }
fn target_view(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<(i32, i32)> { game.base_magic_target_view(region_id, target).map(|view| (view.tile_x, view.tile_y)) }
fn send_start(game: &mut CGame, player_id: i32, level: i32) { let Some(player) = game.find_player(player_id) else { return }; let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(1); message.add_long(STRIKE_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(player.shape().get_direction()); let _ = game.send_player_shape_around(player_id, None, &message); }
fn send_fire(game: &mut CGame, player_id: i32, level: i32, target: ShapeIdentity, position: (i32, i32), flying_time: u32) { let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(2); message.add_long(STRIKE_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(target.object_type); message.add_long(target.id); message.add_long(position.0); message.add_long(position.1); message.add_ulong(flying_time); let _ = game.send_player_shape_around(player_id, None, &message); }

fn calculate_attack(game: &mut CGame, player_id: i32, region_id: i32, target: ShapeIdentity, level: i32, factor: u32, hit: i32) -> Option<(MasterInfo, AttackInformation)> {
    let target_level = target_level(game, region_id, target)?; let player = game.find_player(player_id)?; let combat = player.combat_properties(); let master = master_info(player); let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors(); let weapon_factor = player.weapon_modifier(game.goods_factory(), i32::from(target_level), divisor, minimum_factor); let width = (combat.maximum_attack as i32).wrapping_sub(combat.minimum_attack as i32).max(0).wrapping_add(1); let physical = (combat.minimum_attack as i32).wrapping_add(game.skill_random_below(width)).max(0);
    let mut attack = AttackInformation { skill_id: STRIKE_SKILL_ID, skill_level: level as u8, attacker_type: PLAYER_TYPE, attacker_id: player_id, attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id, hit_modifier: hit.wrapping_neg(), damage_factor: factor as f32 * weapon_factor * 0.01, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: vec![AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 }, AttackPower { kind: AttackPowerType::Element, hp_damage: (combat.add_element_attack as i32).max(0), mp_damage: 0 }, AttackPower { kind: AttackPowerType::Soul, hp_damage: i32::from(combat.add_soul_attack), mp_damage: 0 }] };
    if game.skill_random_below(100) < i32::from(combat.cch) { attack.critical = true; let rate = game.globe_setup().critical_rate(); for power in &mut attack.damages { power.hp_damage = (power.hp_damage as f32 * rate) as i32; } } Some((master, attack))
}

pub(crate) fn execute_player_strike<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_strike_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) } let PlayerSkillDispatch::Object { target, .. } = dispatch else { unreachable!() }; let Some((region_id, source_x, source_y, source_level, level, mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.level(), player.learned_skill_level(STRIKE_SKILL_ID), player.mana()))) else { return terminal(QueuedSkillExecutionState::Rejected) }; let Some(properties) = game.skill_base_properties(STRIKE_SKILL_ID, level) else { if ai.strike().is_some() { abort_player_strike(game, player_id); } return terminal(QueuedSkillExecutionState::Rejected) };
    let mp_loss = properties.query_property(USER_MP_LOSE); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let maximum = properties.query_property(TARGET_MAX_DISTANCE); let missile_step = properties.query_property(MISSILE_FLYING_TIME); let state_time = properties.query_property(STATE_PERSIST_TIME); let factor = properties.query_property(TARGET_DAMAGE_FACTOR); let hit = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32; let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if ai.strike().is_none() { if target.object_type == PLAYER_TYPE && target.id == player_id { fail(game, player_id, 10, mp_loss, b"GS0286"); return terminal(QueuedSkillExecutionState::Rejected) } let now = runtime.now_milliseconds(); if ai.strike_last_used_ms() != 0 && !time_reached(now, ai.strike_last_used_ms(), reuse) { fail(game, player_id, 0x0d, mp_loss, b"GS0278"); return terminal(QueuedSkillExecutionState::Rejected) } let Some(position) = target_view(game, region_id, target) else { fail(game, player_id, 10, mp_loss, b"GS0286"); return terminal(QueuedSkillExecutionState::Rejected) }; let path = game.base_magic_path(region_id, source_x, source_y, position.0, position.1, None); if maximum != 0 && path.len() as u32 > maximum { fail(game, player_id, 0x0b, mp_loss, b"GS0290"); return terminal(QueuedSkillExecutionState::Rejected) } if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) { fail(game, player_id, 0x0f, mp_loss, b"GS0282"); return terminal(QueuedSkillExecutionState::Rejected) } if mp_loss != 0 && (mana.wrapping_sub(mp_loss) as i32) < 0 { fail(game, player_id, 7, mp_loss, b"GS0288"); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(STRIKE_SKILL_ID)); } ai.begin_strike(StrikeExecutionState::begin(dispatch, target, now)); }
    else if ai.strike().is_none_or(|state| state.kernel.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    if game.periodic_state_target_dead(region_id, target) { fail(game, player_id, 10, mp_loss, b"GS0285"); abort_player_strike(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) }
    if ai.strike().is_some_and(|state| !state.condition_checked) { let current = game.find_player(player_id).map_or(0, CPlayer::mana); if (current.wrapping_sub(mp_loss) as i32) < 0 { fail(game, player_id, 7, mp_loss, b"GS0288"); abort_player_strike(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) } if let Some(player) = game.find_player_mut(player_id) { player.set_mana(current.wrapping_sub(mp_loss)); } let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); let Some(position) = target_view(game, region_id, target) else { abort_player_strike(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) }; if let Some(player) = game.find_player_mut(player_id) { player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, position.0, position.1)); } send_start(game, player_id, level); if let Some(state) = ai.strike_mut() { state.condition_checked = true; let _ = state.kernel.advance(SkillStage::Begin, SkillStage::Check); } }
    let started = ai.strike().map(|state| state.kernel.started_at_ms()).unwrap_or_default(); if !ai.strike().is_some_and(|state| state.attacking_started) { if !time_reached(runtime.now_milliseconds(), started, delay) { return terminal(QueuedSkillExecutionState::Pending) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } let Some(position) = target_view(game, region_id, target) else { abort_player_strike(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) }; let path = game.base_magic_path(region_id, source_x, source_y, position.0, position.1, None); if maximum != 0 && path.len() as u32 > maximum { fail(game, player_id, 0x0b, mp_loss, b"GS0290"); abort_player_strike(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) } if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) { game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, 0x0f); let name = game.periodic_state_target_name(region_id, target).to_vec(); game.send_skill_system_info_with_text(player_id, b"GS0296", &name); abort_player_strike(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) } let flying_time = missile_step.wrapping_mul(path.len() as u32); send_fire(game, player_id, level, target, position, flying_time); if let Some(state) = ai.strike_mut() { state.missile_flying_time_ms = flying_time; state.attacking_started = true; let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate); } }
    let flying_time = ai.strike().map_or(0, |state| state.missile_flying_time_ms); if !time_reached(runtime.now_milliseconds(), started, delay.wrapping_add(flying_time)) { return terminal(QueuedSkillExecutionState::Pending) }
    if let Some((master, attack)) = calculate_attack(game, player_id, region_id, target, level, factor, hit) { match target.object_type { PLAYER_TYPE => game.apply_owned_skill_attack_to_player(master, target.id, region_id, attack, runtime), MONSTER_TYPE => game.apply_owned_skill_attack_to_monster(master, target.id, region_id, attack, runtime), _ => {} } }
    if !game.periodic_state_target_dead(region_id, target) && game.find_player(player_id).map(master_info).is_some_and(|master| game.owned_player_skill_target_attackable(master, target, region_id)) { let target_level = target_level(game, region_id, target).unwrap_or(1); let keep = scaled_state_time(source_level, target_level, state_time); if keep != 0 { let now = runtime.now_milliseconds(); let _ = game.install_rush_2_state(region_id, target, Rush2State::new(now, keep), now); } }
    if let Some(state) = ai.strike_mut() { let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack); let _ = state.kernel.advance(SkillStage::Attack, SkillStage::Apply); } finish_player_strike(game, player_id, ai, runtime); terminal(QueuedSkillExecutionState::Completed)
}
