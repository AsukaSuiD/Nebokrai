//! Снежная буря `CSnowStorm` (`0x193`).
//! Успешный Begin возвращает Begun до первого AI. Повторная проверка,
//! расход ресурсов и эффекты AI выполняются после постановки Attack в том
//! же Run; исходный отсчёт Begin сохраняется общим kernel.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/snowstorm.cpp`. Здесь находятся три достигнутых варианта
//! цели, проверки пути, задержка повторного использования, расход MP, стадии
//! `SkillExecutionKernel`,
//! визуальный пакет и построение `CSnowStormPhalanx`. Player-owner расходует
//! MP и блокирует движение; подтверждённый monster-owner пропускает оба
//! player-only эффекта и создаёт область с нулевым element modifier. `CGame`
//! только разрешает владельцев, регистрирует область и выполняет доставку.
//! Player и monster ветви используют абсолютный срок `CSkill::IsRestored`, а
//! задержка и lifetime области сохраняют elapsed-семантику.
//! End (0x005AE7A0) снимает один запрет движения и вызывает CSummonSkill::End
//! также для монстра, хотя его Begin не запрещает движение. Общая политика
//! CMonster сохраняет это при завершении, отмене и Stiffen, не удаляя
//! независимую SnowStormPhalanx и не создавая область повторно.
//! Часы reuse читаются внутри End после регистрации области и очистки cast,
//! отдельно от времени проверки задержки и начала жизни SnowStormPhalanx.

use super::baseattack::time_reached;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::snowstormphalanx::CSnowStormPhalanx;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::ai::monsterai::schedule_attack_interval;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::summonskill::{finish_summon_skill};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;
use crate::setup::monsterlist::MonsterProperties;

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

fn send_monster_visual(game: &CGame, region: &CServerRegion, monster_id: i32, skill_level: u16, action: u8, destination: Option<(i32, i32)>) {
    let Some(monster) = region.find_monster_by_id(monster_id) else { return };
    let mut message = CMessage::new(EFFECT_MESSAGE);
    message.add_byte(action); message.add_long(SNOW_STORM_SKILL_ID as i32); message.base_mut().add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE); message.add_long(monster_id);
    if action == 1 { message.add_long(monster.move_shape().shape().get_direction()); }
    else { let Some((x, y)) = destination else { return }; message.add_long(0); message.add_long(0); message.add_long(x); message.add_long(y); }
    let _ = game.send_game_shape_around(region, monster.move_shape().shape(), None, &message);
}

/// Object-target ветвь `CSnowStorm` для monster-owner-а. В отличие от игрока
/// она не расходует MP и не блокирует движение; область получает нулевой
/// element modifier, как исходный non-player dynamic-cast path.
pub(crate) fn execute_owned_monster_snow_storm<Runtime: GameMainLoopRuntime>(game: &mut CGame, region: &mut CServerRegion, monster_id: i32, target: ShapeIdentity, skill_level: u16, properties: &super::skillbaseproperties::CSkillBaseProperties, property: &MonsterProperties, now_ms: u32, runtime: &mut Runtime, entry: &mut Option<i32>) -> bool {
    let Some((source, cast)) = region.find_monster_by_id(monster_id).map(|monster| (monster.move_shape().shape().clone(), monster.current_active_attack_cast(game.skill_factory()))) else { return false };
    let master = MasterInfo { master_type: MONSTER_TYPE, master_id: monster_id, ..MasterInfo::default() };
    let Some(target_view) = resolve_monster_snow_storm_target(game, region, target) else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) { monster.clear_ai_target(game.skill_factory()); }
        return true;
    };
    let (Ok(source_x), Ok(source_y)) = (source.get_tile_x(), source.get_tile_y()) else { return true };
    let (target_x, target_y) = target_view;
    let path = region.straight_skill_path(source_x, source_y, target_x, target_y, None);
    if cast.is_none() {
        let interval = if region.find_monster_by_id(monster_id).is_some_and(|monster| monster.is_tamed()) { region.find_monster_by_id(monster_id).map(|monster| monster.pet_attack_properties(property).attack_interval).unwrap_or(property.attack_speed) } else { property.attack_speed };
        if schedule_attack_interval(property.ai, interval).is_some_and(|interval| region.find_monster_by_id_mut(monster_id).is_none_or(|monster| !monster.begin_ai_attack_attempt(now_ms, interval))) { return true; }
        let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        let last_used = region.find_monster_by_id(monster_id).map(|monster| monster.skill_last_used_ms(SNOW_STORM_SKILL_ID, game.skill_factory())).unwrap_or_default();
        if !crate::gameserver::appserver::skills::kernel::skill_is_restored(
                last_used, reuse, now_ms,
            )
        {
            return true;
        }
        let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
        if (maximum != 0 && path.len() > maximum as usize) || path.iter().any(|cell| cell.2 == 2) {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) { monster.clear_ai_target(game.skill_factory()); }
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) { monster.move_shape_mut().shape_mut().set_direction(direction); monster.begin_base_attack_cast(target, SNOW_STORM_SKILL_ID, skill_level, now_ms, game.skill_factory()); }
        send_monster_visual(game, region, monster_id, skill_level, 1, None);
        return true;
    }
    let cast = cast.expect("активная снежная буря проверена выше");
    if cast.dispatch().skill_id != SNOW_STORM_SKILL_ID || cast.dispatch().target != target { return false; }
    if !time_reached(now_ms, cast.started_at_ms(), properties.query_property(SKILL_USAGE_DELAY_TIME)) { return true; }
    send_monster_visual(game, region, monster_id, skill_level, 2, Some((target_x, target_y)));
    let summon_id = game.allocate_summon_shape_id();
    let started_at_ms = runtime.now_milliseconds();
    let mut phalanx = CSnowStormPhalanx::new(summon_id, master, started_at_ms, properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME), i32::from(skill_level), properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY), properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32, properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32, 0, properties.query_property(SKILL_USAGE_CONST));
    phalanx.shape_mut().set_region_id(region.id);
    let initialized = phalanx.initialize(target_x, target_y, &mut |maximum| game.skill_random_below(maximum));
    let (area_width, area_height) = game.area_dimensions();
    let summoned = initialized && region.add_snow_storm_phalanx(phalanx, target_x, target_y, area_width, area_height, started_at_ms, runtime).is_ok();
    if summoned { *entry = Some(summon_id); }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SNOW_STORM_SKILL_ID, SkillStage::Check, SkillStage::Calculate, game.skill_factory());
        let _ = monster.advance_base_attack_cast(SNOW_STORM_SKILL_ID, SkillStage::Calculate, SkillStage::Attack, game.skill_factory());
        let _ = monster.advance_base_attack_cast(SNOW_STORM_SKILL_ID, SkillStage::Attack, SkillStage::Apply, game.skill_factory());
        let _ = monster.finish_base_attack_cast_with_clock(SNOW_STORM_SKILL_ID, game.skill_factory(), || runtime.now_milliseconds());
    }
    true
}

fn resolve_monster_snow_storm_target(game: &CGame, region: &CServerRegion, target: ShapeIdentity) -> Option<(i32, i32)> {
    match target.object_type {
        PLAYER_TYPE => { let player = game.find_player(target.id).filter(|player| player.server_region_id() == Some(region.id) && !player.is_dead())?; Some((player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?)) }
        MONSTER_TYPE => { let monster = region.find_monster_by_id(target.id).filter(|monster| monster.hit_points() != 0)?; Some((monster.move_shape().shape().get_tile_x().ok()?, monster.move_shape().shape().get_tile_y().ok()?)) }
        _ => None,
    }
}

fn restore_player_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player_snow_storm<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) {
    restore_player_movement(game, player_id);
    finish_summon_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| player_ai.mark_skill_used(SNOW_STORM_SKILL_ID, now_ms));
}

fn abort_player_snow_storm(game: &mut CGame, player_id: i32) {
    restore_player_movement(game, player_id);
}

pub(crate) fn complete_player_snow_storm<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.player_skill_execution(SNOW_STORM_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false };
    finish_player_snow_storm(game, player_id, player_ai, runtime);
    player_ai.finish_player_skill(dispatch, SkillTermination::Completed)
}

pub(crate) fn cancel_player_snow_storm<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI, _runtime: &mut Runtime) -> bool {
    let Some(dispatch) = player_ai.player_skill_execution(SNOW_STORM_SKILL_ID).map(SkillExecutionKernel::dispatch) else { return false };
    abort_player_snow_storm(game, player_id);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_snow_storm<Runtime: GameMainLoopRuntime>(game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, player_ai: &mut CPlayerAI, runtime: &mut Runtime) -> QueuedSkillExecutionOutcome {
    let skill_id = match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. } | PlayerSkillDispatch::Point { skill_id, .. } | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
    };
    if skill_id != SNOW_STORM_SKILL_ID { return terminal(QueuedSkillExecutionState::Rejected); }
    let Some(player) = game.find_player(player_id) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let skill_level = player.learned_skill_level(skill_id, game.skill_factory());
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

    if player_ai.player_skill_execution(SNOW_STORM_SKILL_ID).is_none() {
        let started_at_ms = runtime.now_milliseconds();
        let cooldown_now_ms = runtime.now_milliseconds();
        if !skill_is_restored(player_ai.skill_last_used_ms(SNOW_STORM_SKILL_ID), cooldown_ms, cooldown_now_ms) {
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
        player_ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, started_at_ms));
        return terminal(QueuedSkillExecutionState::Begun);
    } else if player_ai.player_skill_execution(SNOW_STORM_SKILL_ID).is_none_or(|execution| execution.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai.player_skill_execution(SNOW_STORM_SKILL_ID).is_some_and(|execution| execution.stage() == SkillStage::Begin) {
        let Some((target_x, target_y, target)) = target_position(game, region_id, player_id, dispatch) else { abort_player_snow_storm(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); };
        if target.is_some_and(|identity| game.periodic_state_target_dead(region_id, identity)) {
            send_error(game, player_id, 10);
            abort_player_snow_storm(game, player_id);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let current_mana = game.find_player(player_id).map_or(0, CPlayer::mana);
        if current_mana < mp_loss {
            send_error(game, player_id, 7);
            abort_player_snow_storm(game, player_id);
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
        if let Some(execution) = player_ai.player_skill_execution_mut(SNOW_STORM_SKILL_ID) { let _ = execution.advance(SkillStage::Begin, SkillStage::Check); }
    }

    let started_at_ms = player_ai.player_skill_execution(SNOW_STORM_SKILL_ID).map(SkillExecutionKernel::started_at_ms).expect("выполнение снежной бури создано или восстановлено");
    if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) { return terminal(QueuedSkillExecutionState::Pending); }
    let Some((target_x, target_y, target)) = target_position(game, region_id, player_id, dispatch) else { abort_player_snow_storm(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); };
    if target.is_some_and(|identity| game.periodic_state_target_dead(region_id, identity)) {
        send_error(game, player_id, 10);
        abort_player_snow_storm(game, player_id);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    send_visual(game, player_id, skill_level, 2, Some((target_x, target_y)));
    let Some(player) = game.find_player(player_id) else { abort_player_snow_storm(game, player_id); return terminal(QueuedSkillExecutionState::Rejected); };
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
    if let Some(execution) = player_ai.player_skill_execution_mut(SNOW_STORM_SKILL_ID) {
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_snow_storm(game, player_id, player_ai, runtime);
    terminal(if summoned { QueuedSkillExecutionState::Completed } else { QueuedSkillExecutionState::Rejected })
}

pub(crate) const fn is_snow_storm_target(dispatch: PlayerSkillDispatch) -> bool {
    match dispatch {
        PlayerSkillDispatch::Point { skill_id: SNOW_STORM_SKILL_ID, .. } => true,
        PlayerSkillDispatch::Object { skill_id: SNOW_STORM_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. }, .. } => true,
        _ => false,
    }
}
