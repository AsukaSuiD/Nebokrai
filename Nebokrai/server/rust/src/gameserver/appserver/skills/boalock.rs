//! Управляющий навык `CBoaLock` и его полёт (`0xD2`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/boalock.cpp`. До расхода MP проверяются цель, дальность и
//! исходный путь; после задержки путь к живой цели строится снова. При
//! попадании длительность уменьшается на 25% за каждый уровень цели сверх
//! `уровень владельца + 5`, затем в исходном порядке заменяются связывание и
//! оглушение, после чего выполняется пустая контактная атака с каноническим
//! default skill-id `0x7fffffff` из конструктора `tagAttackInformation`.
//! Состояния принадлежат `CanonicalStateStorage`; `CGame` только координирует
//! владельцев.
//! Унаследованный `CSkill::End(true)` фиксирует cooldown после применения;
//! `End(false)` не откатывает уже установленные состояния и контакт.
//! Коэффициент сокращения времени сохраняется в `f32`, после чего unsigned
//! базовая длительность умножается в x87 и усекается к нулю. Восстановление
//! использует абсолютный срок `CSkill::IsRestored`; задержка остаётся elapsed.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::boalockstate::BoaLockState;
use super::fightdefense::truncate_original;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::knockoutstate::KnockOutState;
use super::poisonmoth::{PLAYER_TYPE, master_info, target_level};
use super::scorpion::{target_name, target_snapshot};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::stateskill::finish_state_skill;
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const BOA_LOCK_SKILL_ID: u32 = 0xd2;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const USER_MP_LOSE: u32 = 2;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const MISSILE_FLYING_TIME: u32 = 10_008;
const STATE_PERSIST_TIME: u32 = 10_002;
const DEFAULT_CONTACT_SKILL_ID: u32 = 0x7fff_ffff;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BoaLockExecutionState { kernel: SkillExecutionKernel<PlayerSkillDispatch>, condition_checked: bool, attacking_started: bool, missile_time: u32 }
impl BoaLockExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, now: u32) -> Self { Self { kernel: SkillExecutionKernel::begin(dispatch, now), condition_checked: false, attacking_started: false, missile_time: 0 } }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}
fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }
pub(crate) fn is_boa_lock_dispatch(dispatch: PlayerSkillDispatch) -> bool { matches!(dispatch, PlayerSkillDispatch::SelfTarget { skill_id: BOA_LOCK_SKILL_ID, .. } | PlayerSkillDispatch::Point { skill_id: BOA_LOCK_SKILL_ID, .. } | PlayerSkillDispatch::Object { skill_id: BOA_LOCK_SKILL_ID, .. }) }
fn target(dispatch: PlayerSkillDispatch) -> Option<ShapeIdentity> { match dispatch { PlayerSkillDispatch::Object { target, .. } => Some(target), _ => None } }
fn restore_player_movement(game: &mut CGame, player_id: i32) { if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } }
fn finish_player_boa_lock<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_state_skill(game, player_id, ai, runtime, |ai, now_ms| ai.mark_skill_used(BOA_LOCK_SKILL_ID, now_ms));
}
fn abort_player_boa_lock(game: &mut CGame, player_id: i32) { restore_player_movement(game, player_id); }
pub(crate) fn complete_player_boa_lock<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.player_skill_state::<BoaLockExecutionState>(BOA_LOCK_SKILL_ID).map(|state| state.kernel().dispatch()) else { return false };
    finish_player_boa_lock(game, player_id, ai, runtime);
    ai.finish_player_skill(dispatch, SkillTermination::Completed)
}
pub(crate) fn cancel_player_boa_lock<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = ai.player_skill_state::<BoaLockExecutionState>(BOA_LOCK_SKILL_ID).map(|state| state.kernel().dispatch()) else { return false };
    abort_player_boa_lock(game, player_id);
    ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}
fn send_failure(game: &CGame, player_id: i32, code: u8, mp: u32, text: Option<&[u8]>) { game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code); match code { 7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp), 10 => game.send_skill_system_info(player_id, text.unwrap_or(b"GS0294")), 0x0b => game.send_skill_system_info(player_id, b"GS0290"), 0x0d => game.send_skill_system_info(player_id, b"GS0278"), _ => {} } }
fn send_path_failure(game: &CGame, player_id: i32, string_id: &[u8], name: &[u8]) { game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, 0x0f); game.send_skill_system_info_with_text(player_id, string_id, name); }
fn send_start(game: &mut CGame, player_id: i32, level: i32) { let Some(player) = game.find_player(player_id) else { return }; let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(1); message.add_long(BOA_LOCK_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(player.shape().get_direction()); let _ = game.send_player_shape_around(player_id, None, &message); }
fn send_fire(game: &mut CGame, player_id: i32, level: i32, identity: ShapeIdentity, position: (i32, i32), missile_time: u32) { let mut message = CMessage::new(EFFECT_MESSAGE); message.add_byte(2); message.add_long(BOA_LOCK_SKILL_ID as i32); message.add_short(level as i16); message.add_long(PLAYER_TYPE); message.add_long(player_id); message.add_long(identity.object_type); message.add_long(identity.id); message.add_long(position.0); message.add_long(position.1); message.add_ulong(missile_time); let _ = game.send_player_shape_around(player_id, None, &message); }
fn reject_begin(game: &mut CGame, player_id: i32, code: Option<(u8, u32, Option<&[u8]>)>) -> QueuedSkillExecutionOutcome { if let Some((code, mp, text)) = code { send_failure(game, player_id, code, mp, text); } send_failure(game, player_id, 2, 0, None); abort_player_boa_lock(game, player_id); terminal(QueuedSkillExecutionState::Rejected) }
fn reject_runtime(game: &mut CGame, player_id: i32, code: Option<(u8, u32, Option<&[u8]>)>) -> QueuedSkillExecutionOutcome { if let Some((code, mp, text)) = code { send_failure(game, player_id, code, mp, text); } abort_player_boa_lock(game, player_id); terminal(QueuedSkillExecutionState::Rejected) }
fn contact(player: &CPlayer) -> AttackInformation { let master = master_info(player); AttackInformation { skill_id: DEFAULT_CONTACT_SKILL_ID, skill_level: 1, attacker_type: PLAYER_TYPE, attacker_id: player.player_id(), attacker_team_id: master.master_team_id, attacker_faction_id: master.master_guild_id, attacker_union_id: master.master_union_id, hit_modifier: 0, damage_factor: 1.0, damage_modifier: 0, critical: false, blast_attack: false, full_miss: 0, damages: Vec::new() } }
fn adjusted_time(source: u8, target: u8, base: u32) -> u32 { if u32::from(source).wrapping_add(5) >= u32::from(target) { return base } let delta = u32::from(target).wrapping_sub(u32::from(source)).wrapping_sub(5) as f32; let factor = (1.0 - delta * 0.25).max(0.0); truncate_original(f64::from(base) * f64::from(factor)) as u32 }

pub(crate) fn execute_player_boa_lock<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    if !is_boa_lock_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) } let Some((region_id, source_x, source_y, level, source_level, initial_mana)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?, player.learned_skill_level(BOA_LOCK_SKILL_ID), player.level(), player.mana()))) else { return terminal(QueuedSkillExecutionState::Rejected) }; let Some(properties) = game.skill_base_properties(BOA_LOCK_SKILL_ID, level) else { return if player_ai.player_skill_state::<BoaLockExecutionState>(BOA_LOCK_SKILL_ID).is_none() { reject_begin(game, player_id, None) } else { reject_runtime(game, player_id, None) } }; let mp = properties.query_property(USER_MP_LOSE); let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME); let delay = properties.query_property(SKILL_USAGE_DELAY_TIME); let max_distance = properties.query_property(TARGET_MAX_DISTANCE); let missile_per_cell = properties.query_property(MISSILE_FLYING_TIME); let persist = properties.query_property(STATE_PERSIST_TIME); let _can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if player_ai.player_skill_state::<BoaLockExecutionState>(BOA_LOCK_SKILL_ID).is_none() { let now = runtime.now_milliseconds(); let Some(identity) = target(dispatch) else { return reject_begin(game, player_id, Some((10, mp, Some(b"GS0294")))) }; if identity.object_type == PLAYER_TYPE && identity.id == player_id { return reject_begin(game, player_id, Some((10, mp, Some(b"GS0286")))) } if !skill_is_restored(player_ai.skill_last_used_ms(BOA_LOCK_SKILL_ID), reuse, now) { return reject_begin(game, player_id, Some((0x0d, mp, None))) } let Some((x, y, _)) = target_snapshot(game, region_id, identity) else { return reject_begin(game, player_id, Some((10, mp, Some(b"GS0294")))) }; let path = game.base_magic_path(region_id, source_x, source_y, x, y, None); if max_distance != 0 && path.len() as u32 > max_distance { return reject_begin(game, player_id, Some((0x0b, mp, None))) } if path.iter().any(|cell| cell.2 == 2) { let name = target_name(game, region_id, identity).to_vec(); send_path_failure(game, player_id, b"GS0295", &name); send_failure(game, player_id, 2, 0, None); abort_player_boa_lock(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) } if mp != 0 && (initial_mana.wrapping_sub(mp) as i32) < 0 { return reject_begin(game, player_id, Some((7, mp, None))) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); player.set_current_skill_id(Some(BOA_LOCK_SKILL_ID)); } player_ai.begin_player_skill_execution(BoaLockExecutionState::begin(dispatch, now)); }
    else if player_ai.player_skill_state::<BoaLockExecutionState>(BOA_LOCK_SKILL_ID).is_none_or(|state| state.kernel.dispatch() != dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let identity = target(dispatch).expect("активный BoaLock сохраняет объектную цель"); let Some((target_x, target_y, dead)) = target_snapshot(game, region_id, identity) else { return reject_runtime(game, player_id, None) }; if dead { return reject_runtime(game, player_id, Some((10, mp, Some(b"GS0285")))) }
    if player_ai.player_skill_state::<BoaLockExecutionState>(BOA_LOCK_SKILL_ID).is_some_and(|state| !state.condition_checked) { let mana = game.find_player(player_id).map_or(0, CPlayer::mana); if (mana.wrapping_sub(mp) as i32) < 0 { return reject_runtime(game, player_id, Some((7, mp, None))) } if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(mp)); player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y)); } let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi); send_start(game, player_id, level); if let Some(state) = player_ai.player_skill_state_mut::<BoaLockExecutionState>(BOA_LOCK_SKILL_ID) { state.condition_checked = true; let _ = state.kernel.advance(SkillStage::Begin, SkillStage::Check); } }
    let started = player_ai.player_skill_state::<BoaLockExecutionState>(BOA_LOCK_SKILL_ID).map(|state| state.kernel.started_at_ms()).unwrap_or_default(); if !player_ai.player_skill_state::<BoaLockExecutionState>(BOA_LOCK_SKILL_ID).is_some_and(|state| state.attacking_started) { if !time_reached(runtime.now_milliseconds(), started, delay) { return terminal(QueuedSkillExecutionState::Pending) } if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); } let Some((x, y, _)) = target_snapshot(game, region_id, identity) else { return reject_runtime(game, player_id, None) }; let path = game.base_magic_path(region_id, source_x, source_y, x, y, None); if max_distance != 0 && path.len() as u32 > max_distance { return reject_runtime(game, player_id, Some((0x0b, mp, None))) } if path.iter().any(|cell| cell.2 == 2) { let name = target_name(game, region_id, identity).to_vec(); send_path_failure(game, player_id, b"GS0296", &name); abort_player_boa_lock(game, player_id); return terminal(QueuedSkillExecutionState::Rejected) } let missile_time = missile_per_cell.wrapping_mul(path.len() as u32); send_fire(game, player_id, level, identity, (x, y), missile_time); if let Some(state) = player_ai.player_skill_state_mut::<BoaLockExecutionState>(BOA_LOCK_SKILL_ID) { state.attacking_started = true; state.missile_time = missile_time; let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate); let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack); } }
    let missile_time = player_ai.player_skill_state::<BoaLockExecutionState>(BOA_LOCK_SKILL_ID).map_or(0, |state| state.missile_time); if !time_reached(runtime.now_milliseconds(), started, delay.wrapping_add(missile_time)) { return terminal(QueuedSkillExecutionState::Pending) } let Some(target_level) = target_level(game, region_id, identity) else { finish_player_boa_lock(game, player_id, player_ai, runtime); return terminal(QueuedSkillExecutionState::Completed) }; let master = game.find_player(player_id).map(master_info); if let Some(master) = master && game.owned_player_skill_target_attackable(master, identity, region_id) { let keep = adjusted_time(source_level, target_level, persist); if keep != 0 { let now = runtime.now_milliseconds(); let lock = (target_level < source_level).then_some(BoaLockState::new(now, keep)); let _ = game.apply_boa_lock_control(region_id, identity, lock, KnockOutState::new(now, keep), now); } if let Some(contact) = game.find_player(player_id).map(contact) { game.apply_owned_skill_contact(master, identity, region_id, contact, runtime); } } if let Some(state) = player_ai.player_skill_state_mut::<BoaLockExecutionState>(BOA_LOCK_SKILL_ID) { let _ = state.kernel.advance(SkillStage::Attack, SkillStage::Apply); } finish_player_boa_lock(game, player_id, player_ai, runtime); terminal(QueuedSkillExecutionState::Completed)
}
