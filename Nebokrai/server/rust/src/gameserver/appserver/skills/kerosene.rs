//! Горючая смесь: наложение на основную цель и соседние клетки.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/kerosene.cpp.
//! Визуальный ресурс принадлежит зарегистрированному навыку, а каждое
//! наложенное состояние — самостоятельному объекту общей арены цели.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::kerosenestate::{KeroseneState, begin_primary_kerosene_state};
use super::poisonmoth::{master_info, weapon_is_crossbow};
use super::skillbaseproperties::CSkillBaseProperties;
use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
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

pub(crate) const KEROSENE_SKILL_ID: u32 = 0xf1;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_CONST: u32 = 20_010;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

pub(crate) fn is_kerosene_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == KEROSENE_SKILL_ID
}

pub(crate) fn complete_player_kerosene<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, KEROSENE_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else { return false; };
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_kerosene<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = game.player_skill_execution(player_id, KEROSENE_SKILL_ID)
        .map(SkillExecutionKernel::dispatch)
    else { return false; };
    game.finish_player_skill(player_id, ai, dispatch, SkillTermination::Cancelled)
}

pub(super) fn kerosene_skill_target(
    game: &CGame, player_id: i32, skill_id: u32,
) -> Option<(i32, ShapeIdentity)> {
    let lifecycle = game.player_skill_lifecycle(player_id, skill_id)?;
    let (region, target) = resolve_skill_sufferer(game, lifecycle)?;
    let shape = resolve_state_move_shape(game, region, target)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

pub(super) fn kerosene_source_position(game: &CGame, player_id: i32) -> Option<(i32, i32, i32)> {
    let player = game.find_player(player_id)?;
    let y = player.shape().get_tile_y().ok()?;
    let x = player.shape().get_tile_x().ok()?;
    Some((player.shape().get_region_id(), x, y))
}

pub(super) fn kerosene_target_position(
    game: &CGame, target: (i32, ShapeIdentity),
) -> Option<(i32, i32)> {
    let shape = resolve_state_move_shape(game, target.0, target.1)?.shape();
    let y = shape.get_tile_y().ok()?;
    let x = shape.get_tile_x().ok()?;
    Some((x, y))
}

pub(super) fn kerosene_path_block(
    game: &CGame,
    player_id: i32,
    skill_id: u32,
    properties: &CSkillBaseProperties,
) -> Option<bool> {
    let path = game.player_skill_lifecycle(player_id, skill_id)
        .map(|lifecycle| game.skill_target_path(lifecycle)).unwrap_or_default();
    // Нулевой предел означает нулевую дальность, а не отсутствие ограничения.
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) < path.len() as u32 {
        return None;
    }
    Some(path.iter().any(|cell| cell.2 == 2))
}

pub(crate) fn publish_kerosene_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CKerosene
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::Kerosene || effect.is_ended()
        })
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    let identity = source.identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 14 | 15) {
        if identity.object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), identity.id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    let target = if action == 2 {
        let Some((region, identity)) = resolve_skill_sufferer(game, skill.lifecycle()) else { return; };
        let Some(shape) = resolve_state_move_shape(game, region, identity) else { return; };
        Some(shape.shape())
    } else { None };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    if let Some(target) = target {
        let (Ok(x), Ok(y)) = (target.get_tile_x(), target.get_tile_y()) else { return; };
        message.add_long(target.identity().object_type);
        message.add_long(target.identity().id);
        message.add_long(x);
        message.add_long(y);
    } else {
        message.add_long(source.get_direction());
    }
    if let Some(region) = game.find_region(source.get_region_id()) {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}

fn failure(game: &mut CGame, player_id: i32, code: u32, amount: u32) {
    game.update_player_skill_visual(player_id, KEROSENE_SKILL_ID, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount),
        10 => game.send_skill_system_info(player_id, b"GS0286"),
        0x0b => game.send_skill_system_info(player_id, b"GS0290"),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        0x0e => game.send_skill_system_info(player_id, b"GS0293"),
        _ => {}
    }
}

fn blocked_path_failure(
    game: &mut CGame, player_id: i32, target: (i32, ShapeIdentity), beginning: bool,
) {
    game.update_player_skill_visual(player_id, KEROSENE_SKILL_ID, 0x0f);
    let name = game.base_magic_target_name(target.0, target.1).unwrap_or_default();
    game.send_skill_system_info_with_text(
        player_id, if beginning { b"GS0291" } else { b"GS0307" }, name,
    );
}

fn reject_begin(game: &mut CGame, player_id: i32) -> QueuedSkillExecutionOutcome {
    game.update_player_skill_visual(player_id, KEROSENE_SKILL_ID, 2);
    terminal(QueuedSkillExecutionState::Rejected)
}

fn apply_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    target: (i32, ShapeIdentity),
    level: i32,
    master: MasterInfo,
    runtime: &mut Runtime,
) {
    if target.1.object_type == PLAYER_TYPE
        && let Some((_, x, y)) = kerosene_source_position(game, player_id)
    {
        let _ = game.player_on_first_skill_at_position(player_id, target.1.id, target.0, x, y, runtime);
    }
    if let Some((position, _)) = resolve_state_move_shape(game, target.0, target.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == KEROSENE_SKILL_ID))
    {
        let _ = end_and_destroy_state_at(game, target.0, target.1, position);
    }
    // Таблица выбрана в начале AI, но значения читаются отдельно для каждой
    // цели после завершения её старого состояния. Новый объект всегда append.
    let Some(properties) = game.skill_base_properties(KEROSENE_SKILL_ID, level) else { return; };
    let hp_loss = properties.query_property(SKILL_USAGE_CONST);
    let frequency = properties.query_property(TARGET_AFFECT_FREQUENCY);
    let keep = properties.query_property(STATE_PERSIST_TIME);
    let state = KeroseneState::new(master, keep, frequency, hp_loss);
    let user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let _ = begin_primary_kerosene_state(
        game, target.0, target.1, user, Some(target), state, None,
        &mut || runtime.now_milliseconds(),
    );
}

fn apply_states<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    target: (i32, ShapeIdentity),
    level: i32,
    runtime: &mut Runtime,
) {
    let Some(mut master) = game.find_player(player_id).map(master_info) else { return; };
    master.master_country_id = 0;
    apply_state(game, player_id, target, level, master, runtime);
    let Some((center_x, center_y)) = kerosene_target_position(game, target) else { return; };
    let Some((width, height)) = game.find_region(target.0)
        .map(|owner| (owner.base().region.width, owner.base().region.height))
    else { return; };
    let left = center_x.wrapping_sub(1).max(0);
    let top = center_y.wrapping_sub(1).max(0);
    let right = center_x.wrapping_add(1).min(width);
    let bottom = center_y.wrapping_add(1).min(height);
    // Центр читается после callbacks основной цели; область остаётся в её
    // прежнем регионе. В каждой клетке выбирается только первый GetShape,
    // поэтому наложение на него не открывает доступ к остальным фигурам клетки.
    for x in left..=right {
        for y in top..=bottom {
            let (area_width, area_height) = game.area_dimensions();
            let Some(identity) = game.find_region(target.0)
                .and_then(|owner| owner.base().get_shape(x, y, area_width, area_height, game).ok().flatten())
                .map(|shape| shape.identity)
            else { continue; };
            // Сравниваются числовые ID без типа. У соседей нет проверки смерти,
            // иммунитета или прав атаки; MasterInfo общий для всего наложения.
            if identity.id == target.1.id || identity.id == player_id
                || !matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE..=603)
            { continue; }
            apply_state(game, player_id, (target.0, identity), level, master, runtime);
        }
    }
}

pub(crate) fn execute_player_kerosene<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_kerosene_dispatch(dispatch) { return terminal(QueuedSkillExecutionState::Rejected); }
    let beginning = game.player_skill_execution(player_id, KEROSENE_SKILL_ID).is_none();
    if beginning {
        game.replace_player_skill_visual_effect(
            player_id, KEROSENE_SKILL_ID, SkillVisualEffect::new(SkillVisualEffectKind::Kerosene, 1),
        );
    }
    let Some(level) = game.registered_player_skill(player_id, KEROSENE_SKILL_ID)
        .and_then(|address| game.registered_skill(address)).map(MoveShapeSkill::level)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !beginning && game.skill_base_properties(KEROSENE_SKILL_ID, level).is_none() {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(target) = kerosene_skill_target(game, player_id, KEROSENE_SKILL_ID) else {
        if beginning {
            failure(game, player_id, 10, 0);
            return reject_begin(game, player_id);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if beginning && target.1.object_type == PLAYER_TYPE && target.1.id == player_id {
        failure(game, player_id, 10, 0);
        return reject_begin(game, player_id);
    }
    let Some(properties) = game.skill_base_properties(KEROSENE_SKILL_ID, level) else {
        return reject_begin(game, player_id);
    };

    if beginning {
        let started = game.player_skill_lifecycle(player_id, KEROSENE_SKILL_ID)
            .expect("общий Begin расписания сохранил базу горючей смеси").started_at_ms();
        let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        if !skill_is_restored(
            game.player_skill_last_used_ms(player_id, KEROSENE_SKILL_ID),
            reuse, runtime.now_milliseconds(),
        ) {
            failure(game, player_id, 0x0d, 0);
            return reject_begin(game, player_id);
        }
        let Some(blocked) = kerosene_path_block(game, player_id, KEROSENE_SKILL_ID, properties) else {
            failure(game, player_id, 0x0b, 0);
            return reject_begin(game, player_id);
        };
        if blocked {
            blocked_path_failure(game, player_id, target, true);
            return reject_begin(game, player_id);
        }
        let Some(player) = game.find_player(player_id) else { return reject_begin(game, player_id); };
        if !weapon_is_crossbow(game, player) {
            failure(game, player_id, 0x0e, 0);
            return reject_begin(game, player_id);
        }
        let mp_loss = properties.query_property(USER_MP_LOSE);
        if mp_loss != 0 && (player.mana().wrapping_sub(properties.query_property(USER_MP_LOSE)) as i32) < 0 {
            let amount = properties.query_property(USER_MP_LOSE);
            failure(game, player_id, 7, amount);
            return reject_begin(game, player_id);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
        }
        game.begin_player_skill_execution(player_id, SkillExecutionKernel::begin(dispatch, started));
        return terminal(QueuedSkillExecutionState::Begun);
    }

    if game.base_magic_target_dead(target.0, target.1) {
        game.update_player_skill_visual(player_id, KEROSENE_SKILL_ID, 10);
        game.send_skill_system_info(player_id, b"GS0285");
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if game.player_skill_execution(player_id, KEROSENE_SKILL_ID)
        .is_some_and(|kernel| kernel.stage() == SkillStage::Begin)
    {
        let Some(mana) = game.find_player(player_id).map(CPlayer::mana) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let mp_loss = properties.query_property(USER_MP_LOSE);
        if (mana.wrapping_sub(mp_loss) as i32) < 0 {
            let amount = properties.query_property(USER_MP_LOSE);
            failure(game, player_id, 7, amount);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
        }
        // Поздняя проверка арбалета не откатывает уже записанный MP.
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        if game.find_player(player_id).is_none_or(|player| !weapon_is_crossbow(game, player)) {
            failure(game, player_id, 0x0e, 0);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(properties) = game.skill_base_properties(KEROSENE_SKILL_ID, level) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let breakable = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, KEROSENE_SKILL_ID) {
            kernel.lifecycle_mut().set_available(breakable != 0);
        }
        let Some((target_x, target_y)) = kerosene_target_position(game, target) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let Some((_, source_x, source_y)) = kerosene_source_position(game, player_id) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(source_x, source_y, target_x, target_y));
        }
        game.update_player_skill_visual(player_id, KEROSENE_SKILL_ID, 0);
        if let Some(kernel) = game.player_skill_execution_mut(player_id, KEROSENE_SKILL_ID) {
            let _ = kernel.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(properties) = game.skill_base_properties(KEROSENE_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.player_skill_execution(player_id, KEROSENE_SKILL_ID)
        .map(SkillExecutionKernel::started_at_ms)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    let Some(properties) = game.skill_base_properties(KEROSENE_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(blocked) = kerosene_path_block(game, player_id, KEROSENE_SKILL_ID, properties) else {
        failure(game, player_id, 0x0b, 0);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if blocked {
        blocked_path_failure(game, player_id, target, false);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    game.update_player_skill_visual(player_id, KEROSENE_SKILL_ID, 1);
    if let Some(kernel) = game.player_skill_execution_mut(player_id, KEROSENE_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = kernel.advance(SkillStage::Calculate, SkillStage::Attack);
    }
    let _ = game.with_published_player_ai(player_id, ai, |game| {
        apply_states(game, player_id, target, level, runtime);
    });
    if let Some(kernel) = game.player_skill_execution_mut(player_id, KEROSENE_SKILL_ID) {
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    terminal(QueuedSkillExecutionState::Completed)
}
