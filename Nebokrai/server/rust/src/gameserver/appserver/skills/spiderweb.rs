//! Паутина паука `CSpiderWeb` (`0x199`) для игрока, монстра и питомца.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/spiderweb.cpp`. Объектный путь сохраняет reuse, две
//! проверки прямого пути и `BLOCK_UNFLY`, запрет движения на cast-delay,
//! flight-time на клетку, level-ограничение и точный пакет `0xBFE01`.
//! После полёта владелец проверяет `Cure`, вычисляет wrapping-длительность и
//! атомарно заменяет канонический `SpiderWebState`. `CGame` только разрешает
//! независимых владельцев, выполняет dispatch и доставку. Координатный
//! overload по точному EXE использует общий `CState::GetSufferer`: выбирает
//! первый `CMoveShape` клетки и затем проходит тот же объектный pipeline.
//

use super::baseattack::{SKILL_USAGE_DELAY_TIME, time_reached};
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_TARGET_MAX_DISTANCE};
use super::monsterattack::{owned_monster_attackable, resolve_owned_monster_attack_target};
use super::skillbaseproperties::CSkillBaseProperties;
use super::spiderwebstate::SpiderWebState;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    resolve_coordinate_sufferer, send_owned_state_visual,
};
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::skills::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use crate::gameserver::appserver::states::summonskill::abort_skill;
use crate::gameserver::appserver::skills::stateskill::finish_state_skill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, GamePlayerFightStatePhase, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, game_tick_milliseconds};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const BLOCK_UNFLY: u8 = 2;
const CURE_SKILL_ID: u32 = 0x131;
const SKILL_USAGE_REUSE_SKILL_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;
const SKILL_USAGE_CONST: u32 = 20_010;
pub(crate) const SPIDER_WEB_SKILL_ID: u32 = 0x199;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PlayerSpiderWebExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    missile_flying_time_ms: Option<u32>,
}

impl PlayerSpiderWebExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, now_ms: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, now_ms), missile_flying_time_ms: None }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpiderWebProgress {
    missile_flying_time_ms: u32,
}

impl SpiderWebProgress {
    pub(crate) const fn new(missile_flying_time_ms: u32) -> Self {
        Self {
            missile_flying_time_ms,
        }
    }

    pub(crate) const fn missile_flying_time_ms(self) -> u32 {
        self.missile_flying_time_ms
    }
}

fn send_cast_start(
    game: &CGame,
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(SPIDER_WEB_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "поля являются точным payload исходного выстрела")]
fn send_cast_fire(
    game: &CGame,
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_level: u16,
    target: ShapeIdentity,
    target_x: i32,
    target_y: i32,
    missile_flying_time_ms: u32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(SPIDER_WEB_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(target_x);
    message.add_long(target_y);
    message.add_ulong(missile_flying_time_ms);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn target_level(game: &CGame, region: &CServerRegion, target: ShapeIdentity) -> Option<i32> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(|player| i32::from(player.level())),
        MONSTER_TYPE => region.find_monster_by_id(target.id).and_then(|monster| {
            game.find_monster_property_by_origin_name(monster.base_property_key()?)
                .map(|property| property.level as i32)
        }),
        _ => None,
    }
}

fn target_has_cure(game: &CGame, region: &CServerRegion, target: ShapeIdentity) -> bool {
    match target.object_type {
        PLAYER_TYPE => game
            .find_player(target.id)
            .is_some_and(|player| player.has_state_by_skill_id(CURE_SKILL_ID)),
        MONSTER_TYPE => region
            .find_monster_by_id(target.id)
            .is_some_and(|monster| {
                monster
                    .move_shape()
                    .has_state_by_skill_id(CURE_SKILL_ID)
            }),
        _ => false,
    }
}

fn install_state(
    game: &mut CGame,
    region: &mut CServerRegion,
    target: ShapeIdentity,
    state: SpiderWebState,
    now_milliseconds: impl FnMut() -> u32,
) {
    let installed = match target.object_type {
        PLAYER_TYPE => game.find_player_mut(target.id).and_then(|player| {
            let previous = player.replace_spider_web_state(state);
            if previous.is_some() {
                player.set_skill_moveable(true);
                player.set_skill_fightable(true);
            }
            player.set_skill_moveable(false);
            player.set_skill_fightable(false);
            Some((
                previous,
                player.shape().clone(),
            ))
        }),
        MONSTER_TYPE => region.find_monster_by_id_mut(target.id).and_then(|monster| {
            let previous = monster.move_shape_mut().replace_spider_web_state(state);
            if previous.is_some() {
                monster.move_shape_mut().set_moveable(true);
                monster.move_shape_mut().set_fightable(true);
            }
            monster.move_shape_mut().set_moveable(false);
            monster.move_shape_mut().set_fightable(false);
            Some((
                previous,
                monster.move_shape().shape().clone(),
            ))
        }),
        _ => None,
    };
    let Some((previous, shape)) = installed else {
        return;
    };
    if let Some(previous) = previous {
        send_owned_state_visual(game, region, &shape, previous.skill_id(), false, 0, 0);
    }
    send_owned_state_visual(
        game,
        region,
        &shape,
        state.skill_id(),
        true,
        state.client_time(now_milliseconds),
        0,
    );
}

fn player_terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

pub(crate) const fn is_player_spider_web_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(
        dispatch,
        PlayerSkillDispatch::Point { skill_id: SPIDER_WEB_SKILL_ID, .. }
            | PlayerSkillDispatch::Object {
                skill_id: SPIDER_WEB_SKILL_ID,
                target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. },
            }
    )
}

fn player_target(game: &CGame, region_id: i32, dispatch: PlayerSkillDispatch) -> Option<ShapeIdentity> {
    match dispatch {
        PlayerSkillDispatch::Object { target, .. } => Some(target),
        PlayerSkillDispatch::Point { skill_id, x, y } if skill_id == SPIDER_WEB_SKILL_ID => {
            let target = resolve_coordinate_sufferer(game, region_id, x, y)?;
            matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE).then_some(target)
        }
        _ => None,
    }
}

fn send_player_failure(game: &CGame, player_id: i32, action: u8) {
    game.send_self_state_skill_failure(0x000b_fe01, player_id, action);
}

fn reject_player_begin(game: &CGame, player_id: i32, action: Option<u8>) -> QueuedSkillExecutionOutcome {
    if let Some(action) = action {
        send_player_failure(game, player_id, action);
    }
    send_player_failure(game, player_id, 2);
    player_terminal(QueuedSkillExecutionState::Rejected)
}

fn send_player_start(game: &mut CGame, player_id: i32, skill_level: i32) {
    let Some(direction) = game.find_player(player_id).map(|player| player.shape().get_direction()) else { return };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(SPIDER_WEB_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(direction);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn send_player_fire(
    game: &mut CGame,
    player_id: i32,
    skill_level: i32,
    target: ShapeIdentity,
    target_x: i32,
    target_y: i32,
    missile_flying_time_ms: u32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(SPIDER_WEB_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(PLAYER_TYPE);
    message.add_long(player_id);
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(target_x);
    message.add_long(target_y);
    message.add_ulong(missile_flying_time_ms);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

fn restore_player_movement(game: &mut CGame, player_id: i32) {
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
    }
}

fn finish_player_spider_web<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) {
    restore_player_movement(game, player_id);
    finish_state_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_spider_web_used(now_ms);
    });
}

fn abort_player_spider_web(game: &mut CGame, player_id: i32) {
    restore_player_movement(game, player_id);
    abort_skill(game, player_id);
}

pub(crate) fn cancel_player_spider_web<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    _runtime: &mut Runtime,
) -> bool {
    let Some(dispatch) = player_ai.spider_web().map(|state| state.kernel().dispatch()) else { return false };
    abort_player_spider_web(game, player_id);
    player_ai.finish_player_skill(dispatch, SkillTermination::Cancelled)
}

fn player_target_level(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<i32> {
    let region = game.find_region(region_id)?;
    target_level(game, region.base(), target)
}

fn player_target_has_cure(game: &CGame, region_id: i32, target: ShapeIdentity) -> bool {
    game.find_region(region_id)
        .is_none_or(|region| target_has_cure(game, region.base(), target))
}

pub(crate) fn execute_player_spider_web<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_player_spider_web_dispatch(dispatch) {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some((region_id, source_x, source_y, source_level, skill_level)) = game
        .find_player(player_id)
        .and_then(|player| {
            Some((
                player.server_region_id()?,
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
                i32::from(player.level()),
                player.learned_skill_level(SPIDER_WEB_SKILL_ID),
            ))
        })
    else {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(target) = player_target(game, region_id, dispatch) else {
        return reject_player_begin(game, player_id, None);
    };
    let Some(properties) = game.skill_base_properties(SPIDER_WEB_SKILL_ID, skill_level).cloned() else {
        if player_ai.spider_web().is_some() {
            abort_player_spider_web(game, player_id);
        }
        return player_terminal(QueuedSkillExecutionState::Rejected);
    };
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_SKILL_DELAY_TIME);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let missile_step_ms = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME);
    let state_lifetime_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let duration_constant = properties.query_property(SKILL_USAGE_CONST);
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    let now_ms = runtime.now_milliseconds();

    let Some(initial_target) = game.base_magic_target_view(region_id, target) else {
        return reject_player_begin(game, player_id, None);
    };
    if player_ai.spider_web().is_none() {
        if player_ai.spider_web_last_used_ms() != 0
            && !time_reached(now_ms, player_ai.spider_web_last_used_ms(), reuse_delay_ms)
        {
            return reject_player_begin(game, player_id, Some(0x0d));
        }
        let path = game.base_magic_path(
            region_id,
            source_x,
            source_y,
            initial_target.tile_x,
            initial_target.tile_y,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            return reject_player_begin(game, player_id, Some(0x0b));
        }
        if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
            return reject_player_begin(game, player_id, Some(0x0f));
        }
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(false);
            player.set_current_skill_id(Some(SPIDER_WEB_SKILL_ID));
        }
        player_ai.begin_spider_web(PlayerSpiderWebExecutionState::begin(dispatch, now_ms));
    } else if player_ai
        .spider_web()
        .is_none_or(|state| state.kernel().dispatch() != dispatch)
    {
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }

    if game.periodic_state_target_dead(region_id, target) {
        send_player_failure(game, player_id, 10);
        abort_player_spider_web(game, player_id);
        return player_terminal(QueuedSkillExecutionState::Rejected);
    }
    if player_ai
        .spider_web()
        .is_some_and(|state| state.kernel().stage() == SkillStage::Begin)
    {
        if let Some(player) = game.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(get_line_direction(
                source_x,
                source_y,
                initial_target.tile_x,
                initial_target.tile_y,
            ));
        }
        let _ = game.update_player_current_state(player_id, GamePlayerFightStatePhase::MoveShapeAi);
        send_player_start(game, player_id, skill_level);
        if let Some(state) = player_ai.spider_web_mut() {
            let _ = state.kernel_mut().advance(SkillStage::Begin, SkillStage::Check);
        }
    }
    let started_at_ms = player_ai
        .spider_web()
        .map(|state| state.kernel().started_at_ms())
        .expect("выполнение паутины хранит время начала");
    if player_ai
        .spider_web()
        .is_some_and(|state| state.missile_flying_time_ms.is_none())
    {
        if !time_reached(runtime.now_milliseconds(), started_at_ms, delay_ms) {
            return player_terminal(QueuedSkillExecutionState::Pending);
        }
        restore_player_movement(game, player_id);
        let Some(target_view) = game.base_magic_target_view(region_id, target) else {
            abort_player_spider_web(game, player_id);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        };
        let Some(target_level) = player_target_level(game, region_id, target) else {
            abort_player_spider_web(game, player_id);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        };
        if source_level.wrapping_add(10) < target_level {
            send_player_failure(game, player_id, 2);
            finish_player_spider_web(game, player_id, player_ai, runtime);
            return player_terminal(QueuedSkillExecutionState::Completed);
        }
        let Some((live_source_x, live_source_y)) = game.find_player(player_id).and_then(|player| {
            Some((player.shape().get_tile_x().ok()?, player.shape().get_tile_y().ok()?))
        }) else {
            abort_player_spider_web(game, player_id);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.base_magic_path(
            region_id,
            live_source_x,
            live_source_y,
            target_view.tile_x,
            target_view.tile_y,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            send_player_failure(game, player_id, 0x0b);
            abort_player_spider_web(game, player_id);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
            send_player_failure(game, player_id, 0x0f);
            abort_player_spider_web(game, player_id);
            return player_terminal(QueuedSkillExecutionState::Rejected);
        }
        let missile_flying_time_ms = missile_step_ms.wrapping_mul(path.len() as u32);
        send_player_fire(
            game,
            player_id,
            skill_level,
            target,
            target_view.tile_x,
            target_view.tile_y,
            missile_flying_time_ms,
        );
        if let Some(state) = player_ai.spider_web_mut() {
            state.missile_flying_time_ms = Some(missile_flying_time_ms);
            let _ = state.kernel_mut().advance(SkillStage::Check, SkillStage::Calculate);
        }
    }

    let missile_flying_time_ms = player_ai
        .spider_web()
        .and_then(|state| state.missile_flying_time_ms)
        .expect("выстрел паутины хранит время полёта");
    if !time_reached(
        runtime.now_milliseconds(),
        started_at_ms,
        delay_ms.wrapping_add(missile_flying_time_ms),
    ) {
        return player_terminal(QueuedSkillExecutionState::Pending);
    }
    let target_level = player_target_level(game, region_id, target).unwrap_or(1);
    if !player_target_has_cure(game, region_id, target) {
        let duration_multiplier = source_level
            .wrapping_sub(target_level)
            .wrapping_add(duration_constant as i32)
            .max(1);
        let keep_time_ms = state_lifetime_ms.wrapping_mul(duration_multiplier as u32);
        let state_now_ms = runtime.now_milliseconds();
        let state = SpiderWebState::new(state_now_ms, keep_time_ms);
        if let Some(mut owner) = game.take_region_owner(region_id) {
            install_state(game, owner.base_mut(), target, state, || runtime.now_milliseconds());
            game.restore_region_owner(owner);
        }
    }
    if let Some(player) = game.find_player_mut(player_id) {
        player.movement_shape_mut().set_action(1);
    }
    if let Some(state) = player_ai.spider_web_mut() {
        let _ = state.kernel_mut().advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.kernel_mut().advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_player_spider_web(game, player_id, player_ai, runtime);
    player_terminal(QueuedSkillExecutionState::Completed)
}

fn cancel_cast(region: &mut CServerRegion, monster_id: i32) {
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().set_moveable(true);
        monster.cancel_base_attack_cast();
    }
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт исходного навыка")]
pub(crate) fn execute_owned_spider_web(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
) -> bool {
    let Some((source_shape, source_property, source_master, source_tamed, cast, last_used_ms, attack_interval)) =
        region.find_monster_by_id(monster_id).and_then(|monster| {
            let source_property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?.clone();
            let attack_interval = if monster.is_tamed() {
                monster.pet_attack_properties(&source_property).attack_interval
            } else {
                source_property.attack_speed
            };
            Some((
                monster.move_shape().shape().clone(),
                source_property,
                monster.master_info(),
                monster.is_tamed(),
                monster.base_attack_cast(),
                monster.skill_last_used_ms(SPIDER_WEB_SKILL_ID),
                attack_interval,
            ))
        })
    else {
        return false;
    };
    let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity) else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
            monster.clear_ai_target();
        }
        return true;
    };
    if target.dead
        || target.god
        || target.city_dead
        || !owned_monster_attackable(
            game,
            region.id,
            &source_property,
            source_tamed,
            source_master,
            target_identity,
            &target,
        )
    {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
            monster.clear_ai_target();
        }
        return true;
    }
    let (Ok(source_x), Ok(source_y), Ok(target_x), Ok(target_y)) = (
        source_shape.get_tile_x(),
        source_shape.get_tile_y(),
        target.shape.get_tile_x(),
        target.shape.get_tile_y(),
    ) else {
        return true;
    };
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);

    if cast.is_none() {
        if !approach_attack_range(
            game,
            region,
            monster_id,
            MonsterTraceTarget::Shape(target.view),
            maximum_distance,
            now_ms,
        ) {
            return true;
        }
        let schedule_ready = schedule_attack_interval(source_property.ai, attack_interval)
            .is_none_or(|interval| {
                region
                    .find_monster_by_id_mut(monster_id)
                    .is_some_and(|monster| monster.begin_ai_attack_attempt(now_ms, interval))
            });
        if !schedule_ready {
            return true;
        }
        let reuse_delay = properties.query_property(SKILL_USAGE_REUSE_SKILL_DELAY_TIME);
        if !crate::gameserver::appserver::skills::kernel::skill_is_restored(
                last_used_ms,
                reuse_delay,
                now_ms,
            )
        {
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(
                target_identity,
                SPIDER_WEB_SKILL_ID,
                skill_level,
                now_ms,
            );
        }
        let source = region
            .find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape())
            .unwrap_or(&source_shape);
        send_cast_start(game, region, source, monster_id, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение паутины проверено выше");
    if cast.dispatch().skill_id != SPIDER_WEB_SKILL_ID
        || cast.dispatch().target != target_identity
    {
        return false;
    }
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    if cast.stage() == SkillStage::Check {
        if !time_reached(now_ms, cast.started_at_ms(), delay_ms) {
            return true;
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
        }
        let Some(target_level) = target_level(game, region, target_identity) else {
            cancel_cast(region, monster_id);
            return true;
        };
        if (source_property.level as i32).wrapping_add(10) < target_level {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                let _ = monster.finish_base_attack_cast(now_ms);
            }
            return true;
        }
        let path = region.straight_skill_path(source_x, source_y, target_x, target_y, None);
        if (maximum_distance != 0 && path.len() > maximum_distance as usize)
            || path.iter().any(|cell| cell.2 == BLOCK_UNFLY)
        {
            cancel_cast(region, monster_id);
            return true;
        }
        let missile_flying_time_ms = properties
            .query_property(SKILL_USAGE_MISSILE_FLYING_TIME)
            .wrapping_mul(path.len() as u32);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
            monster.set_spider_web_progress(SpiderWebProgress::new(missile_flying_time_ms));
        }
        send_cast_fire(
            game,
            region,
            &source_shape,
            monster_id,
            skill_level,
            target_identity,
            target_x,
            target_y,
            missile_flying_time_ms,
        );
        return true;
    }

    let Some(progress) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| monster.spider_web_progress())
    else {
        cancel_cast(region, monster_id);
        return true;
    };
    if !time_reached(
        now_ms,
        cast.started_at_ms(),
        delay_ms.wrapping_add(progress.missile_flying_time_ms()),
    ) {
        return true;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
    }
    if !target_has_cure(game, region, target_identity) {
        let target_level = target_level(game, region, target_identity).unwrap_or(1);
        let duration_multiplier = (source_property.level as i32)
            .wrapping_sub(target_level)
            .wrapping_add(properties.query_property(SKILL_USAGE_CONST) as i32)
            .max(1);
        let keep_time_ms = properties
            .query_property(SKILL_USAGE_STATE_PERSIST_TIME)
            .wrapping_mul(duration_multiplier as u32);
        install_state(
            game,
            region,
            target_identity,
            SpiderWebState::new(now_ms, keep_time_ms),
            game_tick_milliseconds,
        );
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().shape_mut().set_action(1);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}
