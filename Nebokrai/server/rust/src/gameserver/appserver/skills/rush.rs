//! Прямой рывок `CRush` (`0x73`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/rush.cpp`. Навык требует меч категории `1`, дважды
//! проверяет ресурсы, сохраняет необратимое списание MP перед возможным
//! отказом RP, перемещает игрока к последней свободной клетке и обрабатывает
//! фигуры первой заблокированной либо целевой клетки в региональном порядке.
//! Формула длительности, замена `RushState` и отбрасывание принадлежат
//! этому владельцу; `CGame` только координирует владельцев, пространство и сеть.
//! Общий с `CRush2` хвост `End(1)` возвращает движение, освобождает путь,
//! обновляет свойства игрока и фиксирует cooldown; он не повторяет уже
//! выполненные перемещение, состояние или отбрасывание.
//! Коэффициент сокращения времени сохраняется в `f32`, после чего unsigned
//! базовая длительность умножается в x87 и усекается к нулю.

use super::baseattack::{SKILL_USAGE_REUSE_DELAY_TIME, real_distance, time_reached};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::flash::{cell_views, master_info};
use super::fightdefense::truncate_original;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use super::rushstate::RushState;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::states::summonskill::finish_summon_skill;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const RUSH_SKILL_ID: u32 = 0x73;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const USER_RP_LOSE: u32 = 3;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_BACK_STEP: u32 = 1_001;
const TARGET_MOVE_SPEED: u32 = 2_001;

pub(super) fn skill_id(dispatch: PlayerSkillDispatch) -> u32 {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
    }
}

pub(crate) fn is_rush_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    skill_id(dispatch) == RUSH_SKILL_ID
}

pub(super) fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

pub(super) fn finish_rush_owner<Runtime, MarkUsed>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
    mark_used: MarkUsed,
) where
    Runtime: GameMainLoopRuntime,
    MarkUsed: FnOnce(&mut CPlayerAI, u32),
{
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
    finish_summon_skill(game, player_id, player_ai, runtime, mark_used);
}

fn finish_player_rush<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    finish_rush_owner(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_rush_used(now_ms);
    });
}

pub(crate) fn cancel_player_rush<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.rush().map(SkillExecutionKernel::dispatch) else {
        return false;
    };
    finish_player_rush(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(super) fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 1
    })
}

pub(super) fn failure(game: &CGame, player_id: i32, code: u8, amount: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        2 => game.send_skill_system_info(player_id, b"GS0302"),
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", amount),
        8 => game.send_skill_system_info_with_unsigned(player_id, b"GS0289", amount),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        0x0e => game.send_skill_system_info(player_id, b"GS0287"),
        _ => {}
    }
}

pub(super) fn destination(
    game: &CGame,
    region_id: i32,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
) -> Option<(i32, i32)> {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { .. } => game
            .find_player(player_id)
            .and_then(CPlayer::shape_view)
            .map(|view| (view.tile_x, view.tile_y)),
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y)),
        PlayerSkillDispatch::Object { target, .. } => game
            .base_magic_target_view(region_id, target)
            .map(|view| (view.tile_x, view.tile_y)),
    }
}

pub(super) fn target_identity(dispatch: PlayerSkillDispatch) -> Option<ShapeIdentity> {
    match dispatch {
        PlayerSkillDispatch::Object { target, .. } => Some(target),
        _ => None,
    }
}

pub(super) fn build_path(
    game: &CGame,
    region_id: i32,
    source_x: i32,
    source_y: i32,
    target_x: i32,
    target_y: i32,
    maximum: u32,
) -> Option<((i32, i32), (i32, i32))> {
    let mut path = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None);
    if path.first().is_some_and(|cell| cell.0 == source_x && cell.1 == source_y) {
        path.remove(0);
    }
    path.truncate(maximum.min(path.len() as u32) as usize);
    let mut destination = (source_x, source_y);
    let mut impact = (target_x, target_y);
    for (x, y, block) in path {
        if block != 0 {
            impact = (x, y);
            return Some((destination, impact));
        }
        destination = (x, y);
    }
    (destination != (source_x, source_y)).then_some((destination, impact))
}

pub(super) fn send_visual(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    level: i32,
    fire: bool,
    target: Option<ShapeIdentity>,
) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if fire { 2 } else { 1 });
    message.add_long(skill_id as i32);
    message.add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if fire {
        message.add_long(target.map_or(0, |identity| identity.object_type));
        message.add_long(target.map_or(0, |identity| identity.id));
        message.add_long(player.shape().get_tile_x().unwrap_or_default());
        message.add_long(player.shape().get_tile_y().unwrap_or_default());
    } else {
        message.add_long(player.shape().get_direction());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(super) fn scaled_state_time(source_level: u8, target_level: u8, base_time: u32) -> u32 {
    if source_level.saturating_add(5) >= target_level {
        return base_time;
    }
    let difference = target_level.wrapping_sub(source_level).wrapping_sub(5);
    let factor = (1.0_f32 - f32::from(difference) * 0.25).max(0.0);
    truncate_original(f64::from(base_time) * f64::from(factor)) as u32
}

pub(super) fn knockback_destination(
    game: &CGame,
    region_id: i32,
    source_x: i32,
    source_y: i32,
    target: crate::gameserver::appserver::shape::ShapeView,
    steps: u32,
) -> (i32, i32, u32) {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else {
        return (target.tile_x, target.tile_y, 0);
    };
    let direction = get_line_direction(source_x, source_y, target.tile_x, target.tile_y);
    let mut position = ShapeAreaCoordinates { x: target.tile_x, y: target.tile_y };
    let mut moved = 0_u32;
    while moved < steps {
        let Ok(next) = CShape::get_direction_position(direction, position) else { break };
        if region.block_at(next.x, next.y) != Some(0) { break }
        position = next;
        moved = moved.wrapping_add(1);
    }
    (position.x, position.y, moved)
}

#[allow(clippy::too_many_arguments, reason = "параметры сохраняют формулу состояния и отбрасывания")]
fn apply_targets<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    region_id: i32,
    impact_x: i32,
    impact_y: i32,
    source_level: u8,
    state_time: u32,
    back_steps: u32,
    move_speed: u32,
    runtime: &mut Runtime,
) {
    let Some((source_x, source_y, master)) = game.find_player(player_id).and_then(|player| {
        Some((
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
            master_info(player),
        ))
    }) else { return };
    for target in cell_views(game, region_id, impact_x, impact_y) {
        let identity = target.identity;
        if (identity.object_type == PLAYER_TYPE && identity.id == player_id)
            || !matches!(identity.object_type, PLAYER_TYPE | MONSTER_TYPE)
            || !game.owned_player_skill_target_attackable(master, identity, region_id)
        {
            continue;
        }
        if identity.object_type == PLAYER_TYPE {
            let _ = game.player_on_first_attack(player_id, identity.id, Some(region_id), runtime);
        } else if identity.object_type == MONSTER_TYPE {
            let target_master_id = game.find_region(region_id).and_then(|owner| {
                owner.base().find_monster_by_id(identity.id)
                    .map(|monster| monster.master_info().master_id)
            }).unwrap_or_default();
            if target_master_id != 0 && target_master_id != player_id {
                let _ = game.player_on_first_attack(
                    player_id, target_master_id, Some(region_id), runtime,
                );
            }
        }
        let target_level = match identity.object_type {
            PLAYER_TYPE => game.find_player(identity.id).map(CPlayer::level),
            MONSTER_TYPE => game.find_region(region_id).and_then(|owner| {
                let monster = owner.base().find_monster_by_id(identity.id)?;
                game.find_monster_property_by_origin_name(monster.base_property_key()?)
                    .map(|property| property.level as u8)
            }),
            _ => None,
        };
        let Some(keep_time_ms) = target_level
            .map(|level| scaled_state_time(source_level, level, state_time))
            .filter(|time| *time != 0)
        else { continue };
        let (destination_x, destination_y, moved) = knockback_destination(
            game, region_id, source_x, source_y, target, back_steps,
        );
        let now_ms = runtime.now_milliseconds();
        let state = RushState::new(now_ms, keep_time_ms);
        let _ = game.apply_rush_control(
            region_id,
            identity,
            state,
            destination_x,
            destination_y,
            move_speed.wrapping_mul(moved),
            now_ms,
        );
    }
}

pub(crate) fn execute_player_rush<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_rush_dispatch(dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, level, source_level, source_x, source_y, mana, rp)) = game
        .find_player(player_id)
        .and_then(|player| Some((
            player.server_region_id()?,
            player.learned_skill_level(RUSH_SKILL_ID),
            player.level(),
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
            player.mana(),
            player.rp(),
        )))
    else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(RUSH_SKILL_ID, level) else {
        if ai.rush().is_some() { finish_player_rush(game, player_id, ai, runtime) }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let rp_loss = properties.query_property(USER_RP_LOSE);
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum = properties.query_property(TARGET_MAX_DISTANCE);
    let state_time = properties.query_property(STATE_PERSIST_TIME);
    let back_steps = properties.query_property(TARGET_BACK_STEP);
    let move_speed = properties.query_property(TARGET_MOVE_SPEED);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if ai.rush().is_none() {
        let now_ms = runtime.now_milliseconds();
        if ai.rush_last_used_ms() != 0 && !time_reached(now_ms, ai.rush_last_used_ms(), reuse) {
            failure(game, player_id, 0x0d, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(player) = game.find_player(player_id) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if !weapon_is_valid(game, player) {
            failure(game, player_id, 0x0e, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss != 0 && (mana.wrapping_sub(mp_loss) as i32) < 0 {
            failure(game, player_id, 7, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if rp_loss != 0 && (u32::from(rp).wrapping_sub(rp_loss) as i32) < 0 {
            failure(game, player_id, 8, rp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if player.has_state_by_skill_id(0x74) {
            failure(game, player_id, 2, 0);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(RUSH_SKILL_ID));
        }
        ai.begin_rush(SkillExecutionKernel::begin(dispatch, now_ms));
    } else if ai.rush().is_none_or(|execution| execution.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
    if (current_mana.wrapping_sub(mp_loss) as i32) < 0 {
        failure(game, player_id, 7, mp_loss);
        finish_player_rush(game, player_id, ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_mana(current_mana.wrapping_sub(mp_loss));
    }
    let current_rp = game.find_player(player_id).map_or(0, CPlayer::rp);
    if (u32::from(current_rp).wrapping_sub(rp_loss) as i32) < 0 {
        failure(game, player_id, 8, rp_loss);
        finish_player_rush(game, player_id, ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_rp(u32::from(current_rp).wrapping_sub(rp_loss) as u16);
    }
    let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
    if game.find_player(player_id).is_none_or(|player| !weapon_is_valid(game, player)) {
        failure(game, player_id, 0x0e, 0);
        finish_player_rush(game, player_id, ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((target_x, target_y)) = destination(game, region_id, player_id, dispatch) else {
        finish_player_rush(game, player_id, ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let direction = get_line_direction(source_x, source_y, target_x, target_y);
    if let Some(player) = game.find_player_mut(player_id) {
        player.movement_shape_mut().set_direction(direction);
    }
    let Some((destination, impact)) = build_path(
        game, region_id, source_x, source_y, target_x, target_y, maximum,
    ) else {
        finish_player_rush(game, player_id, ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let _ = game.relocate_player_shape(player_id, region_id, destination.0, destination.1);
    send_visual(game, player_id, RUSH_SKILL_ID, level, false, target_identity(dispatch));
    if let Some(execution) = ai.rush_mut() {
        let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
    }

    send_visual(game, player_id, RUSH_SKILL_ID, level, true, target_identity(dispatch));
    if game.find_region(region_id).is_none()
        || real_distance(destination.0, destination.1, impact.0, impact.1) > maximum as i32
    {
        game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, 2);
        finish_player_rush(game, player_id, ai, runtime);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    apply_targets(
        game, player_id, region_id, impact.0, impact.1, source_level,
        state_time, back_steps, move_speed, runtime,
    );
    if let Some(execution) = ai.rush_mut() {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_rush(game, player_id, ai, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}
