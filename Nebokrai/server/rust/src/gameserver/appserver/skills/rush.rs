//! Прямой рывок CRush и общая геометрия семейства Rush/Rush2.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/rush.cpp.
//!
//! Begin проверяет меч и ресурсы; AI отдельно списывает MP перед проверкой RP.
//! Путь имеет заданную длину, а не обрезается у S. Игрок перемещается в последнюю
//! свободную клетку, затем обрабатывается снимок фигур первой преграды либо
//! исходной целевой клетки. Координаты и свойства целей читаются уже при обходе.
//! Rush сообщает первый PK-контакт до расчёта времени; Rush2 вместо этого
//! вызывает OnBeenAttacked после состояния и отбрасывания. End принадлежит
//! зарегистрированному навыку и не повторяется в command-tail.

use super::baseattack::{SKILL_USAGE_REUSE_DELAY_TIME, real_distance};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::flash::cell_views;
use super::fightdefense::truncate_original;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::rushstate::{begin_primary_rush_state, RushState, RUSH_STATE_ID};
use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
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

pub(crate) const RUSH_SKILL_ID: u32 = 0x73;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const USER_RP_LOSE: u32 = 3;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_BACK_STEP: u32 = 1_001;
const TARGET_MOVE_SPEED: u32 = 2_001;

pub(crate) const fn is_rush_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == RUSH_SKILL_ID
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

pub(crate) fn cancel_player_rush<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime,
) -> bool {
    cancel_rush(game, player_id, RUSH_SKILL_ID, ai)
}

pub(super) fn cancel_rush(game: &mut CGame, player_id: i32, skill_id: u32, ai: &mut CPlayerAI) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, skill_id)
        .map(SkillExecutionKernel::dispatch)
    else { return false; };
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Cancelled)
}

fn visual_kind(skill_id: u32) -> SkillVisualEffectKind {
    if skill_id == RUSH_SKILL_ID { SkillVisualEffectKind::Rush } else { SkillVisualEffectKind::Rush2 }
}

pub(crate) fn publish_rush_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if !matches!(skill.owner(), SkillOwner::CRush | SkillOwner::CRush2)
        || skill.visual_effect().is_none_or(|effect| effect.kind() != visual_kind(skill.id()) || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    let identity = source.identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
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
        let target = resolve_skill_sufferer(game, skill.lifecycle());
        message.add_long(target.map_or(0, |(_, target)| target.object_type));
        message.add_long(target.map_or(0, |(_, target)| target.id));
        let (Ok(x), Ok(y)) = (source.get_tile_x(), source.get_tile_y()) else { return; };
        message.add_long(x);
        message.add_long(y);
    }
    if let Some(region) = game.find_region(source.get_region_id()) {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}

fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 1
    })
}

fn failure(game: &mut CGame, player_id: i32, skill_id: u32, code: u32, amount: u32) {
    game.update_player_skill_visual(player_id, skill_id, code);
    match code {
        2 => game.send_skill_system_info(player_id, b"GS0302"),
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount),
        8 => game.send_skill_system_info_with_unsigned(player_id, b"GS0289", amount),
        13 => game.send_skill_system_info(player_id, b"GS0278"),
        14 => game.send_skill_system_info(player_id, b"GS0287"),
        _ => {}
    }
}

pub(super) fn scaled_state_time(source_level: u8, target_level: u8, base_time: u32) -> u32 {
    if u32::from(source_level) + 5 >= u32::from(target_level) { return base_time; }
    let difference = i32::from(target_level) - i32::from(source_level) - 5;
    let factor = (1.0_f32 - difference as f32 * 0.25).max(0.0);
    truncate_original(f64::from(base_time) * f64::from(factor)) as u32
}

pub(super) fn prepare_rush_control(
    game: &CGame, address: RegisteredSkill, user: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
) -> Option<(i32, i32)> {
    resolve_state_move_shape(game, user.0, user.1)?;
    let target_shape = resolve_state_move_shape(game, target.0, target.1)?;
    if !game.live_skill_target_attackable(target_shape.shape().get_region_id(), user.1, target.1) {
        return None;
    }
    let skill = game.registered_skill(address)?;
    game.skill_base_properties(skill.id(), skill.level())?;
    let source = resolve_state_move_shape(game, user.0, user.1)?.shape();
    if !source.is_assigned_to_server_region() { return None; }
    game.find_region(source.get_region_id())?;
    Some((skill.level(), source.get_region_id()))
}

pub(super) fn remove_previous_rush_state(
    game: &mut CGame, target: (i32, ShapeIdentity), skill_id: u32,
) {
    if let Some((index, _)) = resolve_state_move_shape(game, target.0, target.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == skill_id))
    {
        let _ = end_and_destroy_state_at(game, target.0, target.1, index);
    }
}

pub(super) fn rush_knockback(
    game: &mut CGame,
    skill_id: u32,
    property_level: i32,
    source_region: i32,
    user: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity),
) {
    let Some(properties) = game.skill_base_properties(skill_id, property_level) else { return; };
    if properties.query_property(TARGET_BACK_STEP) == 0 { return; }
    let Some(target_shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let (Ok(target_y), Ok(target_x)) = (target_shape.shape().get_tile_y(), target_shape.shape().get_tile_x())
    else { return; };
    let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return; };
    let (Ok(source_y), Ok(source_x)) = (source.shape().get_tile_y(), source.shape().get_tile_x())
    else { return; };
    let direction = get_line_direction(source_x, source_y, target_x, target_y);
    let mut position = ShapeAreaCoordinates { x: target_x, y: target_y };
    let Some(region) = game.find_region(source_region).map(|region| region.base()) else { return; };
    let steps = properties.query_property(TARGET_BACK_STEP);
    let mut moved = 0u32;
    while moved < steps {
        let Ok(next) = CShape::get_direction_position(direction, position) else { break; };
        if region.block_at(next.x, next.y) != Some(0) { break; }
        position = next;
        moved = moved.wrapping_add(1);
    }
    let duration = properties.query_property(TARGET_MOVE_SPEED).wrapping_mul(moved);
    // При ненулевом BACK_STEP ForceMove вызывается и после нуля свободных шагов.
    let _ = game.force_move_skill_target(target.0, target.1, position.x, position.y, duration);
}

fn add_rush_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, address: RegisteredSkill, user: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), keep: u32, runtime: &mut Runtime,
) {
    let Some((level, source_region)) = prepare_rush_control(game, address, user, target) else { return; };
    let state = RushState::new(keep);
    remove_previous_rush_state(game, target, RUSH_STATE_ID);
    let _ = begin_primary_rush_state(
        game, target.0, target.1, Some(user), Some(target), state, &mut || runtime.now_milliseconds(),
    );
    rush_knockback(game, RUSH_SKILL_ID, level, source_region, user, target);
}

fn begin_rush<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let skill_id = dispatch.skill_id();
    game.replace_player_skill_visual_effect(player_id, skill_id, SkillVisualEffect::new(visual_kind(skill_id), 1));
    let Some(skill) = game.registered_player_skill(player_id, skill_id).and_then(|address| game.registered_skill(address))
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    let started = skill.lifecycle().started_at_ms();
    let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(game.player_skill_last_used_ms(player_id, skill_id), reuse, runtime.now_milliseconds()) {
        failure(game, player_id, skill_id, 13, 0);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if !weapon_is_valid(game, player) {
        failure(game, player_id, skill_id, 14, 0);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if properties.query_property(USER_MP_LOSE) != 0 {
        let loss = properties.query_property(USER_MP_LOSE);
        if (player.mana().wrapping_sub(loss) as i32) < 0 {
            let amount = properties.query_property(USER_MP_LOSE);
            failure(game, player_id, skill_id, 7, amount);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
    }
    if properties.query_property(USER_RP_LOSE) != 0 {
        let loss = properties.query_property(USER_RP_LOSE);
        if (u32::from(player.rp()).wrapping_sub(loss) as i32) < 0 {
            let amount = properties.query_property(USER_RP_LOSE);
            failure(game, player_id, skill_id, 8, amount);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
    }
    if player.has_state_by_skill_id(0x74) {
        failure(game, player_id, skill_id, 2, 0);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(false); }
    game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, started));
    terminal(QueuedSkillExecutionState::Begun)
}

fn rush_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, skill_id: u32, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(address) = game.registered_player_skill(player_id, skill_id) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(skill) = game.registered_skill(address) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let level = skill.level();
    let Some(properties) = game.skill_base_properties(skill_id, level) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let user = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !source.shape().is_assigned_to_server_region() { return terminal(QueuedSkillExecutionState::Rejected); }
    let mut impact = match resolve_skill_sufferer(game, skill.lifecycle()) {
        Some((region, identity)) => {
            let Some(target) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let (Ok(x), Ok(y)) = (target.shape().get_tile_x(), target.shape().get_tile_y()) else { return terminal(QueuedSkillExecutionState::Rejected); };
            (x, y)
        }
        None => skill.lifecycle().destination(),
    };
    let Some(player) = game.find_player(user.1.id).filter(|_| user.1.object_type == PLAYER_TYPE) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let mana = player.mana();
    let loss = properties.query_property(USER_MP_LOSE);
    if (mana.wrapping_sub(loss) as i32) < 0 {
        let amount = properties.query_property(USER_MP_LOSE);
        failure(game, player_id, skill_id, 7, amount);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if let Some(player) = game.find_player_mut(user.1.id) { player.set_mana(mana.wrapping_sub(loss)); }
    let Some(player) = game.find_player(user.1.id) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let rp = u32::from(player.rp());
    let Some(properties) = game.skill_base_properties(skill_id, level) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let loss = properties.query_property(USER_RP_LOSE);
    if (rp.wrapping_sub(loss) as i32) < 0 {
        let amount = properties.query_property(USER_RP_LOSE);
        failure(game, player_id, skill_id, 8, amount);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if let Some(player) = game.find_player_mut(user.1.id) { player.set_rp(rp.wrapping_sub(loss) as u16); }
    let _ = game.update_player_current_state(user.1.id, GamePlayerFightStatePhase::MoveShapeAi);
    if game.find_player(user.1.id).is_none_or(|player| !weapon_is_valid(game, player)) {
        failure(game, player_id, skill_id, 14, 0);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(properties) = game.skill_base_properties(skill_id, level) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if let Some(skill) = game.registered_skill_mut(address) { skill.lifecycle_mut().set_available(can_break != 0); }
    let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let (Ok(source_y), Ok(source_x)) = (source.shape().get_tile_y(), source.shape().get_tile_x()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if let Some(player) = game.find_player_mut(user.1.id) {
        player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, impact.0, impact.1));
    }
    let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let (Ok(x), Ok(y)) = (source.shape().get_tile_x(), source.shape().get_tile_y()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let mut destination = (x, y);
    let Some(properties) = game.skill_base_properties(skill_id, level) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let maximum = properties.query_property(TARGET_MAX_DISTANCE);
    let Some(skill) = game.registered_skill(address) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let path = game.skill_target_path_with_length(skill.lifecycle(), maximum);
    if path.is_empty() { return terminal(QueuedSkillExecutionState::Rejected); }
    for &(x, y, block) in &path {
        if block != 0 {
            impact = (x, y);
            break;
        }
        destination = (x, y);
    }
    let _ = game.set_player_tile_position(user.1.id, destination.0, destination.1);
    game.update_player_skill_visual(player_id, skill_id, 0);
    if let Some(kernel) = game.player_skill_execution_mut(player_id, skill_id) { let _ = kernel.advance(SkillStage::Begin, SkillStage::Check); }
    game.update_player_skill_visual(player_id, skill_id, 1);
    let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let region_id = source.shape().get_region_id();
    if game.find_region(region_id).is_none() {
        game.update_player_skill_visual(player_id, skill_id, 2);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let (Ok(x), Ok(y)) = (source.shape().get_tile_x(), source.shape().get_tile_y()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let distance = real_distance(x, y, impact.0, impact.1) as u32;
    let Some(properties) = game.skill_base_properties(skill_id, level) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let maximum = properties.query_property(TARGET_MAX_DISTANCE);
    if if skill_id == RUSH_SKILL_ID { distance > maximum } else { distance >= maximum } {
        game.update_player_skill_visual(player_id, skill_id, 2);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    for view in cell_views(game, region_id, impact.0, impact.1) {
        let target = view.identity;
        if !matches!(target.object_type, 400 | 600 | 601 | 602) || target == user.1 { continue; }
        let Some(shape) = resolve_state_move_shape(game, region_id, target) else { continue; };
        let target = (shape.shape().get_region_id(), target);
        if game.base_magic_target_dead(target.0, target.1)
            || !game.live_skill_target_attackable(target.0, user.1, target.1)
        { continue; }
        if skill_id == RUSH_SKILL_ID
            && let Some(controller) = game.skill_target_controller(target.0, target.1)
            && controller != user.1.id
        {
            let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { continue; };
            let (Ok(y), Ok(x)) = (source.shape().get_tile_y(), source.shape().get_tile_x()) else { continue; };
            let _ = game.player_on_first_attack_at_position(user.1.id, controller, Some(region_id), (x, y), runtime);
        }
        let (Some(source_level), Some(target_level)) = (
            game.move_shape_level(user.0, user.1), game.move_shape_level(target.0, target.1),
        ) else { continue; };
        let Some(properties) = game.skill_base_properties(skill_id, level) else { continue; };
        let keep = scaled_state_time(source_level, target_level, properties.query_property(STATE_PERSIST_TIME));
        if keep == 0 { continue; }
        if skill_id == RUSH_SKILL_ID {
            add_rush_state(game, address, user, target, keep, runtime);
        } else {
            super::rush2::add_rush_2_state(game, address, user, target, keep, runtime);
        }
    }
    if let Some(kernel) = game.player_skill_execution_mut(player_id, skill_id) {
        let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    terminal(QueuedSkillExecutionState::Completed)
}

pub(super) fn execute_rush<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let skill_id = dispatch.skill_id();
    if game.player_skill_execution(player_id, skill_id).is_none() {
        return begin_rush(game, player_id, dispatch, runtime);
    }
    game.with_published_player_ai(player_id, ai, |game| rush_ai(game, player_id, skill_id, runtime))
}

pub(crate) fn execute_player_rush<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_rush_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    execute_rush(game, player_id, dispatch, ai, runtime)
}
