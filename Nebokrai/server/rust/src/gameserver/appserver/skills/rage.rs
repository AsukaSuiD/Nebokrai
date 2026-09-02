//! Канальный навык ярости `CRage` (`0x6D`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/rage.cpp`. После задержки навык публикует начало канала,
//! а затем со строгим интервалом списывает MP и пополняет RP. Достижение
//! предельного RP проверяется после необратимого списания MP. Собственный
//! `RageExecutionState` хранит только жизненный цикл канала; `CGame`
//! разрешает player-owner-а, обновляет общий боевой режим и доставляет пакеты.
//! Замена команды и потеря цели вызывают тот же owner-`End` до очистки AI.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;

pub(crate) const RAGE_SKILL_ID: u32 = 0x6d;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const USER_RP_GAIN: u32 = 23;
const TARGET_AFFECT_FREQUENCY: u32 = 6001;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RageExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    last_using_time_ms: u32,
}

impl RageExecutionState {
    pub(crate) const fn begin(dispatch: PlayerSkillDispatch, started_at_ms: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started_at_ms),
            last_using_time_ms: 0,
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> {
        &self.kernel
    }

    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> {
        &mut self.kernel
    }

    pub(crate) const fn last_using_time_ms(self) -> u32 {
        self.last_using_time_ms
    }

    pub(crate) const fn mark_used(&mut self, now_ms: u32) {
        self.last_using_time_ms = now_ms;
    }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

pub(crate) const fn is_rage_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id == RAGE_SKILL_ID,
    }
}

fn send_failure(game: &CGame, player_id: i32, action: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, action);
    match action {
        7 if mp_loss != 0 => {
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
        }
        7 => game.send_skill_system_info(player_id, b"GS0316"),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        _ => {}
    }
}

fn send_cast_visual(
    game: &mut CGame,
    player_id: i32,
    level: i32,
    action: u8,
    frequency_ms: u32,
) {
    let Some(direction) = game
        .find_player(player_id)
        .map(|player| player.shape().get_direction())
    else {
        return;
    };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(RAGE_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if action == 2 {
        let Some((tile_x, tile_y)) = game.find_player(player_id).and_then(|player| {
            Some((
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
            ))
        }) else {
            return;
        };
        message.add_long(PLAYER_TYPE);
        message.add_long(player_id);
        message.add_long(tile_x);
        message.add_long(tile_y);
        message.add_ulong(frequency_ms);
    } else {
        message.add_long(direction);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

/// Выполняет общий хвост `CRage::End`: свойства и состояние игрока обновляются
/// до визуального завершения. Оружейный `AfterUseSkill` и отметка cooldown
/// остаются у успешного AI-owner-а, поскольку только он знает значение
/// `useRestoreTime` исходного вызова.
pub(crate) fn end_player_rage(game: &mut CGame, player_id: i32, level: i32) {
    let _ = game.update_player_properties(player_id);
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.set_current_skill_id(None);
    }
    let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
    send_cast_visual(game, player_id, level, 3, 0);
}

fn finish_player_rage<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    level: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    game.damage_player_weapon(player_id, runtime);
    end_player_rage(game, player_id, level);
    player_ai.mark_rage_used(runtime.now_milliseconds());
}

pub(crate) fn cancel_player_rage<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.rage().map(|state| state.kernel().dispatch()) else {
        return false;
    };
    let level = game
        .find_player(player_id)
        .map_or(0, |player| player.learned_skill_level(RAGE_SKILL_ID));
    finish_player_rage(game, player_id, level, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_rage<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_rage_dispatch(dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((level, mana)) = game
        .find_player(player_id)
        .map(|player| (player.learned_skill_level(RAGE_SKILL_ID), player.mana()))
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };

    if ai.rage().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        ai.begin_rage(RageExecutionState::begin(dispatch, started_at_ms));
    } else if ai
        .rage()
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some(properties) = game.skill_base_properties(RAGE_SKILL_ID, level) else {
        end_player_rage(game, player_id, level);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let rp_gain = properties.query_property(USER_RP_GAIN);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let frequency_ms = properties.query_property(TARGET_AFFECT_FREQUENCY);
    let reuse_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if ai
        .rage()
        .is_some_and(|state| state.kernel().stage() == SkillStage::Begin)
    {
        let now_ms = runtime.now_milliseconds();
        let last_used_ms = ai.rage_last_used_ms();
        if last_used_ms != 0 && now_ms.wrapping_sub(last_used_ms) < reuse_ms {
            send_failure(game, player_id, 0x0d, 0);
            end_player_rage(game, player_id, level);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 && (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7, mp_loss);
            end_player_rage(game, player_id, level);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            if mp_loss != 0 {
                player.set_skill_moveable(false);
            }
            player.set_current_skill_id(Some(RAGE_SKILL_ID));
        }
    }

    if game.find_player(player_id).is_none_or(|player| player.health() == 0) {
        game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, 2);
        finish_player_rage(game, player_id, level, ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if ai
        .rage()
        .is_some_and(|state| state.kernel().stage() == SkillStage::Begin)
    {
        send_cast_visual(game, player_id, level, 1, frequency_ms);
        if let Some(state) = ai.rage_mut() {
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = ai
        .rage()
        .map(|state| state.kernel().started_at_ms())
        .unwrap_or_default();
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }

    let last_using_time_ms = ai
        .rage()
        .map_or(0, |state| state.last_using_time_ms());
    if last_using_time_ms == 0 {
        send_cast_visual(game, player_id, level, 2, frequency_ms);
        if let Some(state) = ai.rage_mut() {
            let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
            let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        }
    }
    if runtime
        .now_milliseconds()
        .wrapping_sub(last_using_time_ms)
        <= frequency_ms
    {
        return terminal(QueuedSkillExecutionState::Pending);
    }

    game.enter_player_combat_state(player_id);
    let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
    if (current_mana.wrapping_sub(mp_loss) as i32) < 0 {
        send_failure(game, player_id, 7, 0);
        finish_player_rage(game, player_id, level, ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_mana(current_mana.wrapping_sub(mp_loss));
    }

    let reached_maximum = game
        .find_player(player_id)
        .is_none_or(|player| player.maximum_rp() <= player.rp());
    if reached_maximum {
        if let Some(state) = ai.rage_mut() {
            let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
        }
        finish_player_rage(game, player_id, level, ai, runtime);
        return terminal(QueuedSkillExecutionState::Completed);
    }
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_rp(player.rp().wrapping_add(rp_gain as u16));
    }
    let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
    if let Some(state) = ai.rage_mut() {
        state.mark_used(runtime.now_milliseconds());
    }
    terminal(QueuedSkillExecutionState::Pending)
}
