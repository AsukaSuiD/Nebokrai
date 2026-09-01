//! Ярость синего босса `CBossBlueFury` (`0x1f7`) для игрока и монстра.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/bossbluefury.cpp`. Владелец сохраняет обе проверки RP
//! игрока, необратимый расход, действия `0/1`, задержку, перезарядку и замену
//! собственного `BossBlueFuryState`. Состояние остаётся каноническим в
//! `CMoveShape`; `CGame` только координирует владельца и доставку.

use super::baseattack::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
    time_reached,
};
use super::bossbluefurystate::{
    BossBlueFuryState, send_boss_blue_fury_state_visual,
};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::callosity::SKILL_USAGE_USER_RP_LOSE;
use super::monsterattack::resolve_owned_monster_attack_target;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::{
    SkillExecutionKernel, SkillStage, SkillTermination,
};
use crate::gameserver::appserver::skills::stateskill::finish_state_skill;
use crate::gameserver::appserver::states::state::send_owned_state_visual;
use crate::gameserver::appserver::states::summonskill::abort_skill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome,
    QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_STATE_PERSIST_TIME_MODIFIER: u32 = 10_003;
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;
pub(crate) const BOSS_BLUE_FURY_SKILL_ID: u32 = 0x1f7;

fn self_identity(monster_id: i32) -> ShapeIdentity {
    ShapeIdentity {
        object_type: MONSTER_TYPE,
        id: monster_id,
        ex_id: CGuid::GUID_INVALID,
    }
}

fn send_cast_start(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(BOSS_BLUE_FURY_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_cast_fire(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
) {
    let Ok(tile_x) = source.get_tile_x() else {
        return;
    };
    let Ok(tile_y) = source.get_tile_y() else {
        return;
    };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(BOSS_BLUE_FURY_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(tile_x);
    message.add_long(tile_y);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn player_terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

pub(crate) fn is_player_boss_blue_fury_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id == BOSS_BLUE_FURY_SKILL_ID,
    }
}

fn send_player_failure(game: &CGame, player_id: i32, action: u8, rp_loss: u32) {
    game.send_self_state_skill_failure(0x000b_fe01, player_id, action);
    match action {
        8 => game.send_skill_system_info_with_unsigned(player_id, b"GS0289", rp_loss),
        0x0d => game.send_skill_system_info(player_id, b"GS0278"),
        _ => {}
    }
}

fn send_player_visual(game: &mut CGame, player_id: i32, skill_level: i32, action: u8) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let identity = player.shape().identity();
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(action);
    message.add_long(BOSS_BLUE_FURY_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    if action == 1 {
        message.add_long(player.shape().get_direction());
    } else {
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.add_long(player.shape().get_tile_x().unwrap_or_default());
        message.add_long(player.shape().get_tile_y().unwrap_or_default());
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn restore_player_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player_boss_blue_fury<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    finish_state_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_boss_blue_fury_used(now_ms);
    });
}

fn abort_player_boss_blue_fury(game: &mut CGame, player_id: i32) {
    restore_player_movement(game, player_id);
    abort_skill(game, player_id);
}

pub(crate) fn cancel_player_boss_blue_fury<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.boss_blue_fury().map(|state| state.dispatch()) else {
        return false;
    };
    abort_player_boss_blue_fury(game, player_id);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_boss_blue_fury<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_player_boss_blue_fury_dispatch(dispatch) {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, skill_level, initial_rp, dead)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.server_region_id()?,
                player.learned_skill_level(BOSS_BLUE_FURY_SKILL_ID),
                player.rp(),
                player.is_dead(),
            ))
        })
    else {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(properties) = game
        .skill_base_properties(BOSS_BLUE_FURY_SKILL_ID, skill_level)
        .cloned()
    else {
        if player_ai.boss_blue_fury().is_some() {
            abort_player_boss_blue_fury(game, player_id);
        }
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let rp_loss = properties.query_property(SKILL_USAGE_USER_RP_LOSE);
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let damage_factor = properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR) as i32;
    let weak_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME_MODIFIER);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let now_ms = runtime.now_milliseconds();
    if player_ai.boss_blue_fury().is_none() {
        if player_ai.boss_blue_fury_last_used_ms() != 0
            && !time_reached(
                now_ms,
                player_ai.boss_blue_fury_last_used_ms(),
                reuse_delay_ms,
            )
        {
            send_player_failure(game, player_id, 0x0d, rp_loss);
            send_player_failure(game, player_id, 2, rp_loss);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if rp_loss == 0 || (u32::from(initial_rp).wrapping_sub(rp_loss) as i32) < 0 {
            if rp_loss != 0 {
                send_player_failure(game, player_id, 8, rp_loss);
            }
            send_player_failure(game, player_id, 2, rp_loss);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(BOSS_BLUE_FURY_SKILL_ID));
        }
        player_ai.begin_boss_blue_fury(SkillExecutionKernel::begin(dispatch, now_ms));
    }
    if dead {
        send_player_failure(game, player_id, 2, rp_loss);
        restore_player_movement(game, player_id);
        finish_player_boss_blue_fury(game, player_id, player_ai, runtime);
        return player_terminal(QueuedSkillExecutionState::Completed);
    }
    if player_ai
        .boss_blue_fury()
        .is_some_and(|state| state.stage() == SkillStage::Begin)
    {
        let rp = game.find_player(player_id).map_or(0, CPlayer::rp);
        if (u32::from(rp).wrapping_sub(rp_loss) as i32) < 0 {
            send_player_failure(game, player_id, 8, rp_loss);
            abort_player_boss_blue_fury(game, player_id);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_rp(rp.wrapping_sub(rp_loss as u16));
        }
        let _ = game.update_player_current_state(
            player_id,
            GamePlayerFightStatePhase::MoveShapeAi,
        );
        send_player_visual(game, player_id, skill_level, 1);
        if let Some(state) = player_ai.boss_blue_fury_mut() {
            let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started_at_ms = player_ai
        .boss_blue_fury()
        .map(|state| state.started_at_ms())
        .expect("выполнение ярости хранит время начала");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
        return player_terminal(QueuedSkillExecutionState::Pending);
    }
    send_player_visual(game, player_id, skill_level, 2);
    if let Some(state) = player_ai.boss_blue_fury_mut() {
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
    }
    let state_now_ms = runtime.now_milliseconds();
    let state = BossBlueFuryState::new(state_now_ms, keep_time_ms, damage_factor, weak_time_ms);
    let previous = game
        .find_player_mut(player_id)
        .and_then(CPlayer::take_boss_blue_fury_state);
    let identity = game
        .find_player(player_id)
        .map(|player| player.shape().identity())
        .unwrap_or(ShapeIdentity {
            object_type: PLAYER_TYPE,
            id: player_id,
            ex_id: CGuid::GUID_INVALID,
        });
    let (tile_x, tile_y) = game
        .find_player(player_id)
        .map(|player| {
            (
                player.shape().get_tile_x().unwrap_or_default(),
                player.shape().get_tile_y().unwrap_or_default(),
            )
        })
        .unwrap_or_default();
    if let Some(previous) = previous {
        send_boss_blue_fury_state_visual(
            game, region_id, identity, tile_x, tile_y, previous, false, state_now_ms,
        );
        if previous.control_locked() {
            if let Some(player) = game.find_player_mut(player_id) {
                player.set_skill_moveable(true);
                player.set_skill_fightable(true);
            }
        }
    }
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(false);
        player.set_skill_fightable(false);
        player.begin_boss_blue_fury_state(state);
    }
    send_boss_blue_fury_state_visual(
        game, region_id, identity, tile_x, tile_y, state, true, state_now_ms,
    );
    let _ = game.update_player_properties(player_id);
    if let Some(kernel) = player_ai.boss_blue_fury_mut() {
        let _ = kernel.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_boss_blue_fury(game, player_id, player_ai, runtime);
    player_terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_owned_boss_blue_fury(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
) -> bool {
    let Some((source, cast, last_used_ms)) = region
        .find_monster_by_id(monster_id)
        .map(|monster| {
            (
                monster.move_shape().shape().clone(),
                monster.base_attack_cast(),
                monster.skill_last_used_ms(BOSS_BLUE_FURY_SKILL_ID),
            )
        })
    else {
        return false;
    };

    if cast.is_none() {
        let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity)
        else {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        };
        if !approach_attack_range(
            game,
            region,
            monster_id,
            MonsterTraceTarget::Shape(target.view),
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
            now_ms,
        ) {
            return true;
        }
        if last_used_ms != 0
            && !time_reached(
                now_ms,
                last_used_ms,
                properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
            )
        {
            return true;
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.begin_base_attack_cast(
                self_identity(monster_id),
                BOSS_BLUE_FURY_SKILL_ID,
                skill_level,
                now_ms,
            );
        }
        send_cast_start(game, region, &source, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение ярости синего босса проверено выше");
    if cast.dispatch().skill_id != BOSS_BLUE_FURY_SKILL_ID {
        return false;
    }
    if !time_reached(
        now_ms,
        cast.started_at_ms(),
        properties.query_property(SKILL_USAGE_DELAY_TIME),
    ) {
        return true;
    }

    send_cast_fire(game, region, &source, skill_level);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
    }

    let state = BossBlueFuryState::new(
        now_ms,
        properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
        properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR) as i32,
        properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME_MODIFIER),
    );
    let previous = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let previous = monster.move_shape_mut().take_boss_blue_fury_state();
        if previous.is_some_and(BossBlueFuryState::control_locked) {
            monster.move_shape_mut().set_moveable(true);
            monster.move_shape_mut().set_fightable(true);
        }
        previous
    });
    if let Some(previous) = previous {
        send_owned_state_visual(game, region, &source, previous.skill_id(), false, 0, 0);
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().set_moveable(false);
        monster.move_shape_mut().set_fightable(false);
        monster.move_shape_mut().begin_boss_blue_fury_state(state);
    }
    send_owned_state_visual(
        game,
        region,
        &source,
        state.skill_id(),
        true,
        state.client_time(|| now_ms),
        0,
    );

    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}
