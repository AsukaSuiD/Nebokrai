//! Ослепление CBlind (0x76).
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/blind.cpp.
//! Подготовка и visual принадлежат зарегистрированному навыку; AddBlindState
//! создаёт CRushState2 (0x7C), а не CBlindState. Его запреты принадлежат цели.

use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination, skill_is_restored};
use super::rush::scaled_state_time;
use super::rushstate2::{RUSH_2_STATE_ID, Rush2State, begin_primary_rush_2_state};
use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_skill_sufferer, resolve_state_move_shape,
};
use crate::gameserver::appserver::states::visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const BLIND_SKILL_ID: u32 = 0x76;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MP_LOSS: u32 = 2;
const MAX_DISTANCE: u32 = 5_003;
const DELAY: u32 = 10_001;
const STATE_TIME: u32 = 10_002;
const REUSE: u32 = 10_005;
const CAN_BREAK: u32 = 10_006;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

pub(crate) const fn is_blind_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == BLIND_SKILL_ID
}

pub(crate) fn complete_player_blind<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, BLIND_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else { return false; };
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_blind<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, BLIND_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else { return false; };
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Cancelled)
}

fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 2
    })
}

fn target(game: &CGame, player_id: i32) -> Option<(i32, ShapeIdentity)> {
    let lifecycle = game.player_skill_lifecycle(player_id, BLIND_SKILL_ID)?;
    let (region, identity) = resolve_skill_sufferer(game, lifecycle)?;
    let shape = resolve_state_move_shape(game, region, identity)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

pub(crate) fn publish_blind_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CBlind
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::Blind || effect.is_ended()
        })
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    let identity = source.identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    // Вызовы с mode 10/11/15 существуют, но CBlindEffect их не публикует:
    // общий visual tail всё равно выполняется, а текст ошибки отправляет caller.
    if matches!(mode, 2 | 7 | 8 | 13 | 14) {
        if identity.object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), identity.id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    if action == 1 {
        message.add_long(source.get_direction());
    } else {
        message.add_long(0);
        message.add_long(0);
    }
    if let Some(region) = game.find_region(source.get_region_id()) {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}

fn failure(game: &mut CGame, player_id: i32, mode: u32, text: &[u8], mp: Option<u32>) {
    game.update_player_skill_visual(player_id, BLIND_SKILL_ID, mode);
    if let Some(mp) = mp {
        game.send_skill_system_info_with_unsigned(player_id, text, mp);
    } else {
        game.send_skill_system_info(player_id, text);
    }
}

fn begin_blind<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    game.replace_player_skill_visual_effect(
        player_id, BLIND_SKILL_ID, SkillVisualEffect::new(SkillVisualEffectKind::Blind, 1),
    );
    let target = target(game, player_id);
    if game.find_player(player_id).is_none() { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some(skill) = game.registered_player_skill(player_id, BLIND_SKILL_ID)
        .and_then(|address| game.registered_skill(address))
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    let started = skill.lifecycle().started_at_ms();
    let Some(properties) = game.skill_base_properties(BLIND_SKILL_ID, skill.level()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let reuse = properties.query_property(REUSE);
    if !skill_is_restored(
        game.player_skill_last_used_ms(player_id, BLIND_SKILL_ID), reuse, runtime.now_milliseconds(),
    ) {
        failure(game, player_id, 13, b"GS0278", None);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(target) = target else {
        failure(game, player_id, 10, b"GS0286", None);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let path = game.skill_target_path(skill.lifecycle());
    if properties.query_property(MAX_DISTANCE) != 0
        && properties.query_property(MAX_DISTANCE) < path.len() as u32
    {
        failure(game, player_id, 11, b"GS0290", None);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if path.iter().any(|cell| cell.2 == 2) {
        game.update_player_skill_visual(player_id, BLIND_SKILL_ID, 15);
        let name = game.base_magic_target_name(target.0, target.1).unwrap_or_default();
        game.send_skill_system_info_with_text(player_id, b"GS0291", name);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !weapon_is_valid(game, player) {
        failure(game, player_id, 14, b"GS0292", None);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if properties.query_property(MP_LOSS) != 0
        && (player.mana().wrapping_sub(properties.query_property(MP_LOSS)) as i32) < 0
    {
        let amount = properties.query_property(MP_LOSS);
        failure(game, player_id, 7, b"GS0288", Some(amount));
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); }
    game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, started));
    terminal(QueuedSkillExecutionState::Begun)
}

fn add_blind_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, target: (i32, ShapeIdentity), keep: u32, runtime: &mut Runtime,
) {
    if keep == 0 { return; }
    let Some(user) = game.find_player(player_id).map(|player| player.shape().identity()) else { return; };
    if !game.live_skill_target_attackable(target.0, user, target.1) { return; }
    let Some(level) = game.registered_player_skill(player_id, BLIND_SKILL_ID)
        .and_then(|address| game.registered_skill(address)).map(MoveShapeSkill::level)
    else { return; };
    if game.skill_base_properties(BLIND_SKILL_ID, level).is_none() { return; }
    let Some(region) = game.find_player(player_id).and_then(|player| {
        player.shape().is_assigned_to_server_region().then_some(player.shape().get_region_id())
    }) else { return; };
    if game.find_region(region).is_none() { return; }
    let state = Rush2State::new(keep);
    if let Some((position, _)) = resolve_state_move_shape(game, target.0, target.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == RUSH_2_STATE_ID))
    {
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    }
    let user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let sufferer = resolve_state_move_shape(game, target.0, target.1)
        .map(|shape| (shape.shape().get_region_id(), shape.shape().identity()));
    let _ = begin_primary_rush_2_state(
        game, target.0, target.1, user, sufferer, state, &mut || runtime.now_milliseconds(),
    );
}

fn apply_blind<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, target: (i32, ShapeIdentity), source_region: i32,
    level: i32, runtime: &mut Runtime,
) -> bool {
    let Some(user) = game.find_player(player_id).map(|player| player.shape().identity()) else { return false; };
    if !game.live_skill_target_attackable(target.0, user, target.1) { return false; }
    if let Some(controller) = game.skill_target_controller(target.0, target.1)
        .filter(|controller| *controller != player_id)
    {
        let position = game.find_player(player_id).and_then(|player| {
            let y = player.shape().get_tile_y().ok()?;
            let x = player.shape().get_tile_x().ok()?;
            Some((x, y))
        });
        if let Some(position) = position {
            let _ = game.player_on_first_skill_at_position(
                player_id, controller, source_region, position.0, position.1, runtime,
            );
        }
    }
    let Some(source_level) = game.find_player(player_id).map(CPlayer::level) else { return true; };
    let Some(target_level) = game.move_shape_level(target.0, target.1) else { return true; };
    let Some(properties) = game.skill_base_properties(BLIND_SKILL_ID, level) else { return true; };
    let keep = scaled_state_time(source_level, target_level, properties.query_property(STATE_TIME));
    if keep != 0 { add_blind_state(game, player_id, target, keep, runtime); }
    true
}

pub(crate) fn execute_player_blind<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_blind_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    if game.player_skill_execution(player_id, BLIND_SKILL_ID).is_none() {
        return begin_blind(game, player_id, dispatch, runtime);
    }
    let Some(level) = game.registered_player_skill(player_id, BLIND_SKILL_ID)
        .and_then(|address| game.registered_skill(address)).map(MoveShapeSkill::level)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(properties) = game.skill_base_properties(BLIND_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let source_region = game.find_player(player_id).and_then(|player| {
        player.shape().is_assigned_to_server_region().then_some(player.shape().get_region_id())
    });
    let Some(target) = target(game, player_id) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(source_region) = source_region else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.base_magic_target_dead(target.0, target.1) {
        failure(game, player_id, 10, b"GS0285", None);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game.player_skill_execution(player_id, BLIND_SKILL_ID)
        .is_some_and(|kernel| kernel.stage() == SkillStage::Begin)
    {
        let Some(mana) = game.find_player(player_id).map(CPlayer::mana) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let loss = properties.query_property(MP_LOSS);
        if (mana.wrapping_sub(loss) as i32) < 0 {
            let amount = properties.query_property(MP_LOSS);
            failure(game, player_id, 7, b"GS0288", Some(amount));
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) { player.set_mana(mana.wrapping_sub(loss)); }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        if game.find_player(player_id).is_none_or(|player| !weapon_is_valid(game, player)) {
            failure(game, player_id, 14, b"GS0287", None);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some((target_x, target_y)) = resolve_state_move_shape(game, target.0, target.1).and_then(|target| {
            let y = target.shape().get_tile_y().ok()?;
            let x = target.shape().get_tile_x().ok()?;
            Some((x, y))
        }) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let Some((source_x, source_y)) = game.find_player(player_id).and_then(|player| {
            let y = player.shape().get_tile_y().ok()?;
            let x = player.shape().get_tile_x().ok()?;
            Some((x, y))
        }) else { return terminal(QueuedSkillExecutionState::Rejected); };
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        let Some(properties) = game.skill_base_properties(BLIND_SKILL_ID, level) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let can_break = properties.query_property(CAN_BREAK);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, BLIND_SKILL_ID) {
            kernel.lifecycle_mut().set_available(can_break != 0);
        }
        game.update_player_skill_visual(player_id, BLIND_SKILL_ID, 0);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, BLIND_SKILL_ID) {
            let _ = kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(properties) = game.skill_base_properties(BLIND_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let delay = properties.query_property(DELAY);
    let Some(started) = game.player_skill_execution(player_id, BLIND_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some(address) = game.registered_player_skill(player_id, BLIND_SKILL_ID) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(skill) = game.registered_skill(address) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let (_, saved_target) = skill.lifecycle().sufferer();
    if saved_target.object_type != 0 && saved_target.id != 0 {
        let live_target = self::target(game, player_id);
        if live_target.is_none_or(|target| game.base_magic_target_dead(target.0, target.1)) {
            game.update_player_skill_visual(player_id, BLIND_SKILL_ID, 10);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some((region, identity)) = live_target else { return terminal(QueuedSkillExecutionState::Rejected); };
        let Some(shape) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let (Ok(x), Ok(y)) = (shape.shape().get_tile_x(), shape.shape().get_tile_y()) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if let Some(skill) = game.registered_skill_mut(address) { skill.lifecycle_mut().set_point_target((x, y)); }
    }
    let Some(skill) = game.registered_skill(address) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let path = game.skill_target_path(skill.lifecycle());
    let Some(properties) = game.skill_base_properties(BLIND_SKILL_ID, level) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if properties.query_property(MAX_DISTANCE) != 0
        && properties.query_property(MAX_DISTANCE) < path.len() as u32
    {
        failure(game, player_id, 11, b"GS0290", None);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    // Повторный GetS выше нужен только для conversion/path. Наложение и PK
    // сохраняют S, регион U и таблицу свойств, выбранные в начале этого AI.
    game.update_player_skill_visual(player_id, BLIND_SKILL_ID, 1);
    let applied = game.with_published_player_ai(player_id, ai, |game| {
        apply_blind(game, player_id, target, source_region, level, runtime)
    });
    if !applied { return terminal(QueuedSkillExecutionState::Rejected); }
    if let Some(kernel) = game.player_skill_execution_mut(player_id, BLIND_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    terminal(QueuedSkillExecutionState::Completed)
}
