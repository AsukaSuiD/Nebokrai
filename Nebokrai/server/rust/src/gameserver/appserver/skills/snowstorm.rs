//! Снежная буря `CSnowStorm` (`0x193`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/snowstorm.cpp`. Здесь находятся три достигнутых варианта
//! цели, проверки пути, задержка повторного использования, расход MP, стадии
//! `SkillExecutionKernel`,
//! визуальный пакет и построение `CSnowStormPhalanx`. `CGame` только разрешает
//! владельцев, регистрирует область и выполняет фактическую доставку.

use super::baseattack::time_reached;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::snowstormphalanx::CSnowStormPhalanx;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

pub(crate) const SNOW_STORM_SKILL_ID: u32 = 0x193;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_MIN_ATTACK: u32 = 20_001;
const SKILL_USAGE_MAX_ATTACK: u32 = 20_002;
const SKILL_USAGE_CONST: u32 = 20_010;
const SKILL_USAGE_SUMMONED_LIFETIME: u32 = 30_001;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn target_position(game: &CGame, region_id: i32, player_id: i32, dispatch: PlayerSkillDispatch) -> Option<(i32, i32, Option<ShapeIdentity>)> {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { .. } => game.find_player(player_id).and_then(CPlayer::shape_view).map(|shape| (shape.tile_x, shape.tile_y, None)),
        PlayerSkillDispatch::Point { x, y, .. } => Some((x, y, None)),
        PlayerSkillDispatch::Object { target, .. } => game.base_magic_target_view(region_id, target).map(|shape| (shape.tile_x, shape.tile_y, Some(target))),
    }
}

fn send_error(game: &CGame, player_id: i32, code: u8) {
    game.send_self_state_skill_failure(EFFECT_MESSAGE, player_id, code);
}

fn send_visual(game: &mut CGame, player_id: i32, skill_level: i32, action: u8, destination: Option<(i32, i32)>) {
    let Some(player) = game.find_player(player_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(SNOW_STORM_SKILL_ID as i32);
    message.base_mut().add_short(skill_level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    if action == 1 {
        message.add_long(player.shape().get_direction());
    } else {
        let Some((x, y)) = destination else { return };
        message.add_long(0);
        message.add_long(0);
        message.add_long(x);
        message.add_long(y);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn finish(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.set_current_skill_id(None);
    }
}

pub(crate) fn execute_player_snow_storm<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    let skill_id = match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
    };
    if skill_id != SNOW_STORM_SKILL_ID { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let skill_level = player.learned_skill_level(skill_id);
    let Some(region_id) = player.server_region_id() else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(properties) = game.skill_base_properties(skill_id, skill_level) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let cooldown_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let mp_loss = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let frequency_ms = properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY);
    let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let minimum_attack = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum_attack = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let target_count = properties.query_property(SKILL_USAGE_CONST);

    if player_ai.snow_storm().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if player_ai.snow_storm_last_used_ms() != 0 && !time_reached(cooldown_now_ms, player_ai.snow_storm_last_used_ms(), cooldown_ms) {
            send_error(game, player_id, 0x0d);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some((target_x, target_y, target)) = target_position(game, region_id, player_id, dispatch) else {
            send_error(game, player_id, 10);
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if target.is_some_and(|identity| game.periodic_state_target_dead(region_id, identity)) {
            send_error(game, player_id, 10);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(source) = game.find_player(player_id).and_then(CPlayer::shape_view) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let path = game.base_magic_path(region_id, source.tile_x, source.tile_y, target_x, target_y, None);
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_error(game, player_id, 0x0b);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if path.iter().any(|cell| cell.2 == 2) {
            send_error(game, player_id, 0x0f);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if mp_loss == 0 || player.mana() < mp_loss {
            send_error(game, player_id, 7);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(skill_id));
        }
        player_ai.begin_snow_storm(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if player_ai.snow_storm().is_none_or(|execution| execution.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai.snow_storm().is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let Some((target_x, target_y, target)) = target_position(game, region_id, player_id, dispatch) else { finish(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); };
        if target.is_some_and(|identity| game.periodic_state_target_dead(region_id, identity)) {
            send_error(game, player_id, 10);
            finish(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if current_mana < mp_loss {
            send_error(game, player_id, 7);
            finish(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_mana(current_mana.wrapping_sub(mp_loss));
            let source = player.shape_view();
            if let Some(source) = source { player.movement_shape_mut().set_direction(get_line_direction(source.tile_x, source.tile_y, target_x, target_y)); }
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        let _ = game.update_player_criminal_state(player_id, GamePlayerFightStatePhase::MoveShapeAi, runtime);
        send_visual(game, player_id, skill_level, 1, None);
        if let Some(execution) = player_ai.snow_storm_mut() { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }
    }

    let started_at_ms = player_ai.snow_storm().map(SkillExecutionKernel::started_at_ms).expect("выполнение снежной бури создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) { return terminal(QueuedSkillExecutionState::Pending); }
    let Some((target_x, target_y, target)) = target_position(game, region_id, player_id, dispatch) else { finish(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); };
    if target.is_some_and(|identity| game.periodic_state_target_dead(region_id, identity)) {
        send_error(game, player_id, 10);
        finish(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    send_visual(game, player_id, skill_level, 2, Some((target_x, target_y)));
    let Some(player) = game.find_player(player_id) else { finish(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); };
    let master = MasterInfo { master_type: PLAYER_TYPE, master_id: player_id, master_guild_id: player.faction_id(), master_team_id: player.team_id(), master_union_id: player.union_id(), master_country_id: i32::from(player.country()), permitted_to_kill_player: i32::from(player.pk_permissions().player), permitted_to_kill_teammate: i32::from(player.pk_permissions().teammate), permitted_to_kill_guild_member: i32::from(player.pk_permissions().guild_member), permitted_to_kill_criminal: i32::from(player.pk_permissions().criminal) };
    let element_modifier = player.combat_properties().element_modify;
    let summon_id = game.allocate_summon_shape_id();
    let summon_started_at_ms = runtime.now_milliseconds();
    let mut phalanx = CSnowStormPhalanx::new(summon_id, master, summon_started_at_ms, lifetime_ms, skill_level, frequency_ms, minimum_attack, maximum_attack, element_modifier, target_count);
    phalanx.shape_mut().set_region_id(region_id);
    let initialized = phalanx.initialize(target_x, target_y, &mut |maximum| game.skill_random_below(maximum));
    let summoned = initialized
        && game.add_snow_storm_phalanx(region_id, phalanx, target_x, target_y, summon_started_at_ms, runtime).is_some_and(|result| result.is_ok());
    if summoned { let _ = game.send_snow_storm_phalanx_entry(region_id, summon_id, runtime); }
    if let Some(execution) = player_ai.snow_storm_mut() {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    player_ai.mark_snow_storm_used(runtime.now_milliseconds());
    finish(game, player_id);
    terminal(if summoned { QueuedSkillExecutionState::Completed } else { QueuedSkillExecutionState::Rejected })
}

pub(crate) const fn is_snow_storm_target(dispatch: PlayerSkillDispatch) -> bool {
    match dispatch {
        PlayerSkillDispatch::Point { skill_id: SNOW_STORM_SKILL_ID, .. } => true,
        PlayerSkillDispatch::Object { skill_id: SNOW_STORM_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. }, .. } => true,
        _ => false,
    }
}
