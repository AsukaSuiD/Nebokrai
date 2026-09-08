//! Самонакладываемая смазка оружия ядом `CDaubPoison` (`0xDF`).
//! Begin возвращает Begun после инициализации; повторные проверки и эффекты
//! первого AI исполняются после постановки Attack в том же Run.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/daubpoison.cpp`. Все перегрузки `Begin` подтверждённо
//! игнорируют запрошенную цель и выбирают игрока. Сохранены две проверки MP,
//! условная блокировка движения только при ненулевом расходе, задержка,
//! cooldown, пакеты `0xBFE01` и порядок замены состояния `end → begin`.
//! `CGame` предоставляет player owner, свойства, обновление состояния и
//! фактическую доставку; lifecycle навыка остаётся здесь.
//! Достигнутый apply-path фиксирует cooldown даже при смерти владельца или
//! неудачной установке состояния; раннее прерывание этого не делает.
//! Входной cooldown сохраняет absolute DWORD deadline `CSkill::IsRestored`.

use super::baseattack::time_reached;
use super::daubpoisonstate::{DaubPoisonState, DAUB_POISON_STATE_ID, replace_player_daub_poison_state};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::stateskill::finish_state_skill;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;

pub(crate) const DAUB_POISON_SKILL_ID: u32 = DAUB_POISON_STATE_ID;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const DELAY_TIME: u32 = 10_001;
const STATE_PERSIST_TIME: u32 = 10_002;
const REUSE_DELAY_TIME: u32 = 10_005;
const CAN_BE_BREAKED: u32 = 10_006;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

pub(crate) const fn is_daub_poison_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id == DAUB_POISON_SKILL_ID,
    }
}

fn restore_player_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player_daub_poison<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, _ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_state_skill(game, player_id, DAUB_POISON_SKILL_ID, runtime);
}

fn abort_player_daub_poison(game: &mut CGame, player_id: i32) { restore_player_movement(game, player_id); }

pub(crate) fn complete_player_daub_poison<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, DAUB_POISON_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_daub_poison(game, player_id, ai, runtime);
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_daub_poison<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, DAUB_POISON_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false };
    abort_player_daub_poison(game, player_id);
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Cancelled)
}

fn send_failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        _ => {}
    }
}

fn send_cast(game: &mut CGame, player_id: i32, level: i32, apply: bool) {
    let Some(player) = game.find_player(player_id) else { return };
    let x = player.shape().get_tile_x().unwrap_or_default();
    let y = player.shape().get_tile_y().unwrap_or_default();
    let direction = player.shape().get_direction();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if apply { 2 } else { 1 });
    message.add_long(DAUB_POISON_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if apply {
        message.add_long(PLAYER_TYPE);
        message.add_long(player_id);
        message.add_long(x);
        message.add_long(y);
    } else {
        message.add_long(direction);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) fn execute_player_daub_poison<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_daub_poison_dispatch(dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((level, initial_mana)) = game.find_player(player_id).map(|player| {
        (player.learned_skill_level(DAUB_POISON_SKILL_ID, game.skill_factory()), player.mana())
    }) else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(DAUB_POISON_SKILL_ID, level) else {
        if game.player_skill_execution(player_id, DAUB_POISON_SKILL_ID).is_some() { abort_player_daub_poison(game, player_id); }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay = properties.query_property(DELAY_TIME);
    let keep_time = properties.query_property(STATE_PERSIST_TIME);
    let reuse = properties.query_property(REUSE_DELAY_TIME);
    let _breakable = properties.query_property(CAN_BE_BREAKED);

    if game.player_skill_execution(player_id, DAUB_POISON_SKILL_ID).is_none() {
        let now_ms = runtime.now_milliseconds();
        if !skill_is_restored(game.player_skill_last_used_ms(player_id, DAUB_POISON_SKILL_ID), reuse, now_ms) {
            send_failure(game, player_id, 0x0d, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 && (initial_mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            if mp_loss != 0 { player.set_skill_moveable(false); }
            player.set_current_skill_id(Some(DAUB_POISON_SKILL_ID));
        }
        game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, now_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if game.player_skill_execution(player_id, DAUB_POISON_SKILL_ID)
        .is_none_or(|execution| execution.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if game.find_player(player_id).is_none_or(CPlayer::is_dead) {
        send_failure(game, player_id, 2, mp_loss);
        finish_player_daub_poison(game, player_id, player_ai, runtime);
        return terminal(QueuedSkillExecutionState::Completed);
    }
    if game.player_skill_execution(player_id, DAUB_POISON_SKILL_ID).is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            send_failure(game, player_id, 7, mp_loss);
            abort_player_daub_poison(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_cast(game, player_id, level, false);
        if let Some(execution) = game.player_skill_execution_mut(player_id, DAUB_POISON_SKILL_ID) {
            let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started = game.player_skill_execution(player_id, DAUB_POISON_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение смазки оружия создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started, delay) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    send_cast(game, player_id, level, true);
    let now_ms = runtime.now_milliseconds();
    let installed = replace_player_daub_poison_state(
        game,
        player_id,
        DaubPoisonState::new(now_ms, keep_time),
        || runtime.now_milliseconds(),
    );
    if let Some(execution) = game.player_skill_execution_mut(player_id, DAUB_POISON_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_daub_poison(game, player_id, player_ai, runtime);
    terminal(if installed { QueuedSkillExecutionState::Completed } else { QueuedSkillExecutionState::Rejected })
}
