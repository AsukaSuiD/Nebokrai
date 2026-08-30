//! Достигнутый player→player контракт `CHearten`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/skills/hearten.cpp`. Навык `324` сохраняет две проверки MP,
//! расход перед началом каста, задержку, направление на цель, замену состояния,
//! публикацию `OnChangeStates` и отдельное время восстановления. Клиентская
//! отмена снимает движение и текущий навык, но не возвращает уже списанную MP.

pub(crate) const HEARTEN_SKILL_ID: u32 = 324;
pub(crate) const HEARTEN_EFFECT_MESSAGE: i32 = 0x000b_fe01;
pub(crate) const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
pub(crate) const SKILL_USAGE_MAX_HP_GAIN: u32 = 118;
pub(crate) const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
pub(crate) const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
pub(crate) const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
pub(crate) const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub(crate) const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

use super::baseattack::time_reached;
use super::heartenstate::{send_hearten_state_visual, HeartenState};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const PLAYER_TYPE: i32 = 400;

fn send_cast(
    game: &mut CGame,
    player_id: i32,
    target_id: i32,
    skill_level: i32,
    action: u8,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let source = player.shape().identity();
    let source_direction = player.shape().get_direction();
    let target = game.find_player(target_id).map(|target| {
        (
            target.shape().identity(),
            target.shape().get_tile_x().unwrap_or_default(),
            target.shape().get_tile_y().unwrap_or_default(),
        )
    });
    let mut message = CMessage::new(HEARTEN_EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(HEARTEN_SKILL_ID as i32);
    message.base_mut().add_short(skill_level as i16);
    message.add_long(source.object_type);
    message.add_long(source.id);
    if action == 1 {
        message.add_long(source_direction);
    } else if let Some((target, target_x, target_y)) = target {
        message.add_long(target.object_type);
        message.add_long(target.id);
        message.add_long(target_x);
        message.add_long(target_y);
    } else {
        return;
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}
fn finish_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.set_current_skill_id(None);
    }
}

fn finish_player_hearten<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    finish_movement(game, player_id);
    player_ai.mark_hearten_used(runtime.now_milliseconds());
}

pub(crate) fn cancel_player_hearten<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.hearten().map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    finish_player_hearten(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_hearten<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let terminal = |state| QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    };
    let requested_target_id = match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
            if skill_id == HEARTEN_SKILL_ID => player_id,
        PlayerSkillDispatch::Object { skill_id, target }
            if skill_id == HEARTEN_SKILL_ID && target.object_type == PLAYER_TYPE => target.id,
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some((region_id, source_x, source_y, skill_level, initial_mana)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.learned_skill_level(HEARTEN_SKILL_ID),
                player.mana(),
            ))
        })
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (target_id, target_x, target_y) = game
        .find_player(requested_target_id)
        .and_then(|target| {
            (target.server_region_id()? == region_id).then(|| {
                Some((
                    requested_target_id,
                    target.shape().get_tile_x().ok()?,
                    target.shape().get_tile_y().ok()?,
                ))
            })?
        })
        .unwrap_or((player_id, source_x, source_y));
    let Some(properties) = game.skill_base_properties(HEARTEN_SKILL_ID, skill_level) else {
        game.send_self_state_skill_failure(HEARTEN_EFFECT_MESSAGE, player_id, 2);
        finish_movement(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let max_hp_gain = properties.query_property(SKILL_USAGE_MAX_HP_GAIN) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.hearten().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        let cooldown_now_ms = runtime.now_milliseconds();
        if player_ai.hearten_last_used_ms() != 0
            && !time_reached(
                cooldown_now_ms,
                player_ai.hearten_last_used_ms(),
                reuse_delay_ms,
            )
        {
            game.send_self_state_skill_failure(HEARTEN_EFFECT_MESSAGE, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            game.send_self_state_skill_failure(HEARTEN_EFFECT_MESSAGE, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if player_id != target_id
            && maximum_distance != 0
            && game
                .base_magic_path(region_id, source_x, source_y, target_x, target_y, None)
                .len()
                > maximum_distance as usize
        {
            game.send_self_state_skill_failure(HEARTEN_EFFECT_MESSAGE, player_id, 0x0b);
            game.send_skill_system_info(player_id, b"GS0290");
            game.send_self_state_skill_failure(HEARTEN_EFFECT_MESSAGE, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 && initial_mana < mp_loss {
            game.send_self_state_skill_failure(HEARTEN_EFFECT_MESSAGE, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            game.send_self_state_skill_failure(HEARTEN_EFFECT_MESSAGE, player_id, 2);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            if mp_loss != 0 {
                player.set_skill_moveable(false);
            }
            player.set_current_skill_id(Some(HEARTEN_SKILL_ID));
        }
        player_ai.begin_hearten(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if player_ai
        .hearten()
        .is_none_or(|state| state.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if game.find_player(target_id).is_some_and(CPlayer::is_dead) {
        game.send_self_state_skill_failure(HEARTEN_EFFECT_MESSAGE, player_id, 10);
        finish_movement(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai
        .hearten()
        .is_some_and(|state| state.stage() == SkillStage::Begin)
    {
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if current_mana < mp_loss {
            game.send_self_state_skill_failure(HEARTEN_EFFECT_MESSAGE, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            finish_movement(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(current_mana.wrapping_sub(mp_loss));
            if player_id != target_id {
                player
                    .movement_shape_mut()
                    .set_direction(get_line_direction(source_x, source_y, target_x, target_y));
            }
        }
        let _ = game.publish_player_states(player_id);
        send_cast(game, player_id, target_id, skill_level, 1);
        if let Some(state) = player_ai.hearten_mut() {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .hearten()
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение воодушевления создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }

    send_cast(game, player_id, target_id, skill_level, 2);
    let removed = game.find_player_mut(target_id).and_then(|player| {
        player.replace_hearten_state(HeartenState::new(
            runtime.now_milliseconds(),
            keep_time_ms,
            max_hp_gain,
        ))
    });
    if let Some(state) = removed {
        send_hearten_state_visual(game, target_id, state, false, || 0);
    }
    let _ = game.publish_player_states(target_id);
    if let Some(state) = player_ai.hearten_mut() {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_hearten(game, player_id, player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
