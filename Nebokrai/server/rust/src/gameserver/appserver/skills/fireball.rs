//! Огненный шар `CFireBall` (`0x13D`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/fireball.cpp`. Владелец сохраняет обе проверки MP,
//! время восстановления, предел пути, задержку, направление и визуальные пакеты. После
//! задержки он целиком снимает `CSoulCollectState`, публикует его завершение и
//! только затем создаёт региональный `CFireBallPhalanx`. `CGame` выполняет
//! лишь доступ к независимым владельцам, регистрацию в пространстве и доставку.

use super::baseattack::time_reached;
use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_ELEMENT_MODIFIER,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_SUMMONED_LIFETIME, SKILL_USAGE_SUMMONED_SPEED,
    SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::fireballphalanx::CFireBallPhalanx;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::soulcollectstate::send_soul_collect_state_visual;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const FIRE_BALL_SKILL_ID: u32 = 0x13d;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

const fn has_mana(mana: u32, loss: u32) -> bool {
    mana.wrapping_sub(loss) as i32 >= 0
}

fn master_info(player: &CPlayer) -> MasterInfo {
    let permissions = player.pk_permissions();
    MasterInfo {
        master_type: PLAYER_TYPE,
        master_id: player.player_id(),
        master_guild_id: player.faction_id(),
        master_team_id: player.team_id(),
        master_union_id: player.union_id(),
        master_country_id: 0,
        permitted_to_kill_player: i32::from(permissions.player),
        permitted_to_kill_teammate: i32::from(permissions.teammate),
        permitted_to_kill_guild_member: i32::from(permissions.guild_member),
        permitted_to_kill_criminal: i32::from(permissions.criminal),
    }
}

fn send_failure(game: &CGame, player_id: i32, code: u8, mp_loss: u32) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
    match code {
        7 => game.send_skill_system_info_with_unsigned(player_id, b"GS0288", mp_loss),
        10 => game.send_skill_system_info(player_id, b"GS0285"),
        0x0b => game.send_skill_system_info(player_id, b"GS0290"),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        _ => {}
    }
}

fn finish(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.set_current_skill_id(None);
    }
}

fn destination(
    game: &CGame,
    region_id: i32,
    dispatch: PlayerSkillDispatch,
) -> Option<(i32, i32, Option<ShapeIdentity>)> {
    match dispatch {
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y, None)),
        PlayerSkillDispatch::Object { target, .. }
            if matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE) => {
                let view = game.base_magic_target_view(region_id, target)?;
                Some((view.tile_x, view.tile_y, Some(target)))
            }
        _ => None,
    }
}

fn target_dead(game: &CGame, region_id: i32, target: ShapeIdentity) -> bool {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).is_none_or(CPlayer::is_dead),
        MONSTER_TYPE => game.find_region(region_id)
            .and_then(|owner| owner.base().find_monster_by_id(target.id))
            .is_none_or(|monster| monster.hit_points() == 0),
        _ => true,
    }
}

fn send_visual(
    game: &mut CGame,
    player_id: i32,
    level: i32,
    destination: Option<(ShapeIdentity, i32, i32)>,
) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(if destination.is_some() { 2 } else { 1 });
    message.add_long(FIRE_BALL_SKILL_ID as i32);
    message.base_mut().add_short(level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if let Some((target, x, y)) = destination {
        message.add_long(target.object_type);
        message.add_long(target.id);
        message.add_long(x);
        message.add_long(y);
    } else {
        message.add_long(player.shape().get_direction());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) const fn is_fire_ball_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::Point { skill_id: FIRE_BALL_SKILL_ID, .. }
        | PlayerSkillDispatch::Object {
            skill_id: FIRE_BALL_SKILL_ID,
            target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. },
        }
    )
}

pub(crate) fn execute_player_fire_ball<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_fire_ball_dispatch(dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, level, initial_mana, source_x, source_y)) = game
        .find_player(player_id)
        .and_then(|player| Some((
            player.server_region_id()?,
            player.learned_skill_level(FIRE_BALL_SKILL_ID),
            player.mana(),
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
        )))
    else { return terminal(QueuedSkillExecutionState::Rejected) };
    let Some(properties) = game.skill_base_properties(FIRE_BALL_SKILL_ID, level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mp_loss = properties.query_property(USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let cooldown_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let speed_ms = properties.query_property(SKILL_USAGE_SUMMONED_SPEED);
    let minimum_attack = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum_attack = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let element_modifier = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER) as i32;
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);

    if player_ai.fire_ball().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        if matches!(dispatch, PlayerSkillDispatch::Object { target, .. } if target.object_type == PLAYER_TYPE && target.id == player_id) {
            game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, 10);
            game.send_skill_system_info(player_id, b"GS0286");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if player_ai.fire_ball_last_used_ms() != 0
            && !time_reached(runtime.now_milliseconds(), player_ai.fire_ball_last_used_ms(), cooldown_ms)
        {
            send_failure(game, player_id, 0x0d, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some((target_x, target_y, target)) = destination(game, region_id, dispatch) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if target.is_some_and(|identity| target_dead(game, region_id, identity)) {
            send_failure(game, player_id, 10, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let route = game.base_magic_path(region_id, source_x, source_y, target_x, target_y, None);
        if maximum_distance != 0 && route.len() > maximum_distance as usize {
            send_failure(game, player_id, 0x0b, mp_loss);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss == 0 || !has_mana(initial_mana, mp_loss) {
            if mp_loss != 0 { send_failure(game, player_id, 7, mp_loss); }
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(FIRE_BALL_SKILL_ID));
        }
        player_ai.begin_fire_ball(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if player_ai.fire_ball().is_none_or(|execution| execution.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let Some((target_x, target_y, target)) = destination(game, region_id, dispatch) else {
        finish(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if target.is_some_and(|identity| target_dead(game, region_id, identity)) {
        send_failure(game, player_id, 10, mp_loss);
        finish(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if player_ai.fire_ball().is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if !has_mana(mana, mp_loss) {
            send_failure(game, player_id, 7, mp_loss);
            finish(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(mana.wrapping_sub(mp_loss));
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x, source_y, target_x, target_y,
            ));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_visual(game, player_id, level, None);
        if let Some(execution) = player_ai.fire_ball_mut() {
            let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        }
    }

    let started_at_ms = player_ai.fire_ball()
        .map(SkillExecutionKernel::started_at_ms)
        .expect("выполнение огненного шара создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    if let Some(player) = game.find_player_mut(player_id) { player.set_skill_moveable(true); }
    send_visual(
        game,
        player_id,
        level,
        Some((target.unwrap_or(ShapeIdentity { object_type: 0, id: 0, ex_id: Default::default() }), target_x, target_y)),
    );

    let Some((live_x, live_y, face_x, face_y, master)) = game.find_player(player_id).and_then(|player| {
        let face = player.shape().get_face_position().ok()?;
        Some((
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
            face.x,
            face.y,
            master_info(player),
        ))
    }) else {
        finish(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let mut path = game.base_magic_path(region_id, live_x, live_y, target_x, target_y, None);
    if maximum_distance != 0 && path.len() > maximum_distance as usize {
        path.truncate(maximum_distance as usize);
    }
    if let Some(index) = path.iter().position(|cell| cell.2 == 2) { path.truncate(index); }
    let path = path.into_iter().map(|(x, y, _)| (x, y)).collect::<Vec<_>>();

    let soul = game.find_player_mut(player_id).and_then(CPlayer::take_soul_collect_state);
    let (soul_count, soul_variable) = soul.map_or((0, 0), |state| {
        send_soul_collect_state_visual(
            game,
            region_id,
            ShapeIdentity { object_type: PLAYER_TYPE, id: player_id, ex_id: Default::default() },
            live_x,
            live_y,
            state,
            false,
        );
        (state.souls(), state.variable_percent())
    });
    let summon_id = game.allocate_summon_shape_id();
    let summon_started_at_ms = runtime.now_milliseconds();
    let mut phalanx = CFireBallPhalanx::new(
        summon_id, master, summon_started_at_ms, lifetime_ms, level,
        minimum_attack, maximum_attack, element_modifier, path.clone(), speed_ms,
        soul_count, soul_variable,
    );
    phalanx.shape_mut().set_region_id(region_id);
    let (spawn_x, spawn_y) = path.first().copied().unwrap_or((face_x, face_y));
    let result = game.add_fire_ball_phalanx(
        region_id, phalanx, spawn_x, spawn_y, summon_started_at_ms, runtime,
    );
    if result.as_ref().is_some_and(|result| result.is_ok()) {
        let _ = game.send_fire_ball_phalanx_entry(region_id, summon_id, runtime);
    }
    tracing::trace!(region_id, player_id, summon_id, ?result, "создан огненный шар");

    if let Some(execution) = player_ai.fire_ball_mut() {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    player_ai.mark_fire_ball_used(runtime.now_milliseconds());
    finish(game, player_id);
    terminal(QueuedSkillExecutionState::Completed)
}
