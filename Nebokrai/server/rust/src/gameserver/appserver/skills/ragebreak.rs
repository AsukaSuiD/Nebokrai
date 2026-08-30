//! Подготовка яростного удара `CRageBreak` (`0x6E`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/ragebreak.cpp`. Владелец сохраняет проверку и повторное
//! списание RP, задержку, replacement `CRageBreakState`, порядок снятия
//! конфликтующих состояний и последующее наложение `CCureState`. `CGame`
//! используется только для канонического player-owner-а, доставки и общего
//! пересчёта свойств. Подтверждённый `End` возвращает движение и выполняет
//! общий хвост `CSummonSkill::End(1)` после установки состояний.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::cure::finish_curable_state;
use super::curestate::{CureState, send_cure_state_visual};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use super::ragebreakstate::{RageBreakState, send_rage_break_state_visual};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

pub(crate) const RAGE_BREAK_SKILL_ID: u32 = 0x6e;
const PLAYER_TYPE: i32 = 400;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const USER_RP_LOSE: u32 = 3;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_ATTACK_GAIN: u32 = 105;
const CONFLICTING_STATES: [u32; 9] = [0x138, 0xd2, 0xc9, 0x67, 0x192, 0x191, 0x198, 0x199, 0x1a6];

fn skill_id(dispatch: PlayerSkillDispatch) -> u32 { match dispatch { PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id } }
pub(crate) fn is_rage_break_dispatch(dispatch: PlayerSkillDispatch) -> bool { skill_id(dispatch) == RAGE_BREAK_SKILL_ID }
fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome { QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None } }

fn finish_player_rage_break<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) {
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| { player_ai.mark_rage_break_used(now_ms); });
}

pub(crate) fn cancel_player_rage_break<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.rage_break().map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_rage_break(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn fail(game: &CGame, player_id: i32, code: u8, rp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        8 => game.send_skill_system_info_with_unsigned(player_id, b"GS0289", rp_loss),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        _ => {}
    }
}

fn send_cast_visual(game: &mut CGame, player_id: i32, level: i32, fired: bool) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if fired { 2 } else { 1 });
    message.add_long(RAGE_BREAK_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if fired {
        message.add_long(PLAYER_TYPE);
        message.add_long(player_id);
        message.add_long(player.shape().get_tile_x().unwrap_or_default());
        message.add_long(player.shape().get_tile_y().unwrap_or_default());
    } else {
        message.add_long(player.shape().get_direction());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) fn execute_player_rage_break<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_rage_break_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected) }
    let Some((level, rp)) = game.find_player(player_id).map(|player| (player.learned_skill_level(RAGE_BREAK_SKILL_ID), player.rp())) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(RAGE_BREAK_SKILL_ID, level) else { if ai.rage_break().is_some() { finish_player_rage_break(game, player_id, ai, runtime); } return terminal(QueuedSkillExecutionState::Rejected) };
    let rp_loss = properties.query_property(USER_RP_LOSE);
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let keep = properties.query_property(STATE_PERSIST_TIME);
    let attack_gain = properties.query_property(TARGET_ATTACK_GAIN) as i32;
    let _breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if ai.rage_break().is_none() {
        let now = runtime.now_milliseconds();
        if ai.rage_break_last_used_ms() != 0 && now < ai.rage_break_last_used_ms().wrapping_add(reuse) {
            fail(game, player_id, 0x0d, rp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if rp_loss == 0 { return terminal(QueuedSkillExecutionState::Rejected) }
        if (u32::from(rp).wrapping_sub(rp_loss) as i32) < 0 {
            fail(game, player_id, 8, rp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(RAGE_BREAK_SKILL_ID));
        }
        ai.begin_rage_break(SkillExecutionKernel::begin(dispatch, now));
    } else if ai.rage_break().is_none_or(|state| state.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if ai.rage_break().is_some_and(|state| state.stage() == SkillStage::Begin) {
        let current = game.find_player(player_id).map_or(0, CPlayer::rp);
        if (u32::from(current).wrapping_sub(rp_loss) as i32) < 0 {
            fail(game, player_id, 8, rp_loss);
            finish_player_rage_break(game, player_id, ai, runtime);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_rp(u32::from(current).wrapping_sub(rp_loss) as u16); }
        send_cast_visual(game, player_id, level, false);
        if let Some(state) = ai.rage_break_mut() { let _ = state.advance(SkillStage::Begin, SkillStage::Check); }
    }

    let started = ai.rage_break().map(SkillExecutionKernel::started_at_ms).unwrap_or_default();
    if started.wrapping_add(delay) > runtime.now_milliseconds() { return terminal(QueuedSkillExecutionState::Pending) }
    send_cast_visual(game, player_id, level, true);
    if let Some(state) = ai.rage_break_mut() {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
    }

    let now = runtime.now_milliseconds();
    let Some((region_id, tile_x, tile_y)) = game.find_player(player_id).and_then(|player| Some((player.server_region_id()?, player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?))) else {
        finish_player_rage_break(game, player_id, ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let identity = ShapeIdentity { object_type: PLAYER_TYPE, id: player_id, ex_id: CGuid::GUID_INVALID };
    let replacement = game.find_player_mut(player_id).and_then(CPlayer::take_rage_break_state);
    if let Some(previous) = replacement { send_rage_break_state_visual(game, region_id, identity, tile_x, tile_y, previous, false, now); }
    let state = RageBreakState::new(now, keep, attack_gain);
    if let Some(player) = game.find_player_mut(player_id) { player.replace_rage_break_state(state); }
    send_rage_break_state_visual(game, region_id, identity, tile_x, tile_y, state, true, now);

    let order = game.find_player(player_id).map(CPlayer::curable_state_ids).unwrap_or_default();
    for state_id in order {
        if CONFLICTING_STATES.contains(&state_id) { let _ = finish_curable_state(game, region_id, identity, state_id, now); }
    }
    let cure = CureState::new(identity, identity).begin_now();
    let previous_cure = game.find_player_mut(player_id).and_then(|player| player.replace_cure_state(cure));
    if let Some(previous) = previous_cure { send_cure_state_visual(game, player_id, previous, false); }
    send_cure_state_visual(game, player_id, cure, true);
    let _ = game.update_player_properties(player_id, runtime);

    if let Some(state) = ai.rage_break_mut() { let _ = state.advance(SkillStage::Attack, SkillStage::Apply); }
    finish_player_rage_break(game, player_id, ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
