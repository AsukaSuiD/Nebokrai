//! Управление целью всех питомцев игрока (`CPetsControl`, навык `0xd7`).
//! Успешный Begin возвращает Begun до первого AI; координатор ставит Attack
//! и продолжает AI в том же Run. Проверки и побочные эффекты фаз сохранены.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/skills/petscontrol.cpp`. Навык сохраняет двойную проверку MP,
//! отдельную задержку повторного применения, длину клеточного пути, направление
//! к цели и формат действий `0xBFE01`. После задержки `CGame` только разрешает
//! регионального владельца и передаёт цель всем принадлежащим игроку питомцам.
//! Объектная перегрузка принимает любой отличный от владельца `CMoveShape`;
//! поэтому NPC проходит cast и назначение, а уже monster AI отвергает его своим
//! `IsAttackAble` и выполняет обычную потерю цели. Восстановление использует
//! абсолютный срок `CSkill::IsRestored`; стадийная задержка остаётся elapsed.

use super::baseattack::time_reached;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::summonskill::{
    finish_summon_skill_without_weapon_wear,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const PETS_CONTROL_SKILL_ID: u32 = 0xd7;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const NPC_TYPE: i32 = 500;
const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn send_failure(game: &mut CGame, player_id: i32, reason: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, reason);
}

fn send_cast(
    game: &mut CGame,
    player_id: i32,
    target: ShapeIdentity,
    skill_level: i32,
    action: u8,
) {
    let Some(player) = game.find_player(player_id) else { return; };
    let source = player.shape().identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(PETS_CONTROL_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(source.object_type);
    message.add_long(source.id);
    if action == 1 {
        message.add_long(player.shape().get_direction());
    } else if action == 2 {
        let Some((tile_x, tile_y)) = game.move_shape_target_tile(
            player.server_region_id(),
            target,
        ) else {
            return;
        };
        message.add_long(target.object_type);
        message.add_long(target.id);
        message.add_long(tile_x);
        message.add_long(tile_y);
    } else {
        return;
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn abort_player_pets_control(game: &mut CGame, player_id: i32) { finish_movement(game, player_id); }
fn finish_player_pets_control<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) { finish_summon_skill_without_weapon_wear(game, player_id, player_ai, runtime, |player_ai, now_ms| player_ai.mark_skill_used(PETS_CONTROL_SKILL_ID, now_ms)); }
pub(crate) fn complete_player_pets_control<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool { let Some(dispatch) = player_ai.player_skill_execution(PETS_CONTROL_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false }; finish_movement(game, player_id); finish_player_pets_control(game, player_id, player_ai, runtime); player_ai.finish_player_skill(dispatch, SkillTermination::Completed) }
pub(crate) fn cancel_player_pets_control<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool { let Some(dispatch) = player_ai.player_skill_execution(PETS_CONTROL_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false }; abort_player_pets_control(game, player_id); player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled) }

pub(crate) fn execute_player_pets_control<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let target = match dispatch {
        PlayerSkillDispatch::Object {
            skill_id: PETS_CONTROL_SKILL_ID,
            target,
        } if matches!(target.object_type, PLAYER_TYPE | NPC_TYPE | MONSTER_TYPE)
            && !(target.object_type == PLAYER_TYPE && target.id == player_id) => {
            target
        }
        _ => {
            send_failure(game, player_id, 10);
            game.send_skill_system_info(player_id, b"GS0286");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
    };
    let Some((region_id, source_x, source_y, skill_level, initial_mana)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                player.learned_skill_level(PETS_CONTROL_SKILL_ID),
                player.mana(),
            ))
        })
    else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some((target_x, target_y)) = game.move_shape_target_tile(Some(region_id), target) else {
        send_failure(game, player_id, 10);
        game.send_skill_system_info(player_id, b"GS0286");
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game.skill_base_properties(PETS_CONTROL_SKILL_ID, skill_level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.player_skill_execution(PETS_CONTROL_SKILL_ID).is_none() {
        let started_at_ms = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        if !skill_is_restored(
            player_ai.skill_last_used_ms(PETS_CONTROL_SKILL_ID),
            reuse_delay_ms,
            runtime.now_milliseconds(),
        ) {
            send_failure(game, player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if maximum_distance != 0
            && game
                .base_magic_path(region_id, source_x, source_y, target_x, target_y, None)
                .len()
                > maximum_distance as usize
        {
            send_failure(game, player_id, 0x0b);
            game.send_skill_system_info(player_id, b"GS0290");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 && initial_mana < mp_loss {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            if mp_loss != 0 {
                player.set_skill_moveable(false);
            }
            player.set_current_skill_id(Some(PETS_CONTROL_SKILL_ID));
        }
        player_ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if player_ai.player_skill_execution(PETS_CONTROL_SKILL_ID).is_none_or(|state| state.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if target.object_type != NPC_TYPE && game.periodic_state_target_dead(region_id, target) {
        send_failure(game, player_id, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        abort_player_pets_control(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai.player_skill_execution(PETS_CONTROL_SKILL_ID).is_some_and(|state| state.stage() == SkillStage::Begin) {
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if current_mana < mp_loss {
            send_failure(game, player_id, 7);
            game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss);
            abort_player_pets_control(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(current_mana.wrapping_sub(mp_loss));
            player
                .movement_shape_mut()
                .set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        let _ = game.update_player_current_state(
            player_id,
            GamePlayerFightStatePhase::MoveShapeAi,
        );
        send_cast(game, player_id, target, skill_level, 1);
        if let Some(state) = player_ai.player_skill_execution_mut(PETS_CONTROL_SKILL_ID) {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai
        .player_skill_execution(PETS_CONTROL_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение управления питомцами создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }

    abort_player_pets_control(game, player_id);
    send_cast(game, player_id, target, skill_level, 2);
    let _ = game.set_player_pets_target(player_id, target.object_type, target.id);
    if let Some(state) = player_ai.player_skill_execution_mut(PETS_CONTROL_SKILL_ID) {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_pets_control(game, player_id, player_ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
