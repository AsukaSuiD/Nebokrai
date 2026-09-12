//! Паутина CSpiderWeb (0x199) для игрока, монстра и питомца.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/spiderweb.cpp.
//!
//! Общий зарегистрированный Begin предшествует visual и CheckCast. Первый AI,
//! а не Begin, задаёт CAN_BE_BREAKED, направление и начальный эффект. Стадия
//! исполнения хранит отдельные concrete флаги работы/проверки/полёта;
//! lifecycle.available — независимый базовый флаг прерываемости.
//! Reuse, delay и flight сравнивают абсолютные wrapping-сроки DWORD.
//! Эффект выпуска заново разрешает S; игровой хвост использует U/S,
//! захваченные в начале этого AI. End очищает concrete поля,
//! снимает запрет движения и выполняет общий StateSkill End того же экземпляра.
//!
//! После полёта Cure и уровни читаются у фактической цели, включая постройки.
//! Payload создаётся до End первого прежнего состояния, его свежий слот
//! уничтожается, а новый Begin предшествует append в каноническую арену.
//! Поиск координатной S и GetBeAttackedPoint принадлежат общей базе;
//! отдельные пути, пакеты или копии lifecycle здесь не хранятся.

use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_TARGET_MAX_DISTANCE};
use super::blindstate::begin_primary_blind_state;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage, SkillTermination};
use super::skillbaseproperties::CSkillBaseProperties;
use super::skillfactory::SkillOwner;
use super::spiderwebstate::SpiderWebState;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterSkillCallOutcome, finish_monster_skill_call,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_owned_skill_begin_object, resolve_skill_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner,
};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

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
    flight: SpiderWebProgress,
}

impl PlayerSpiderWebExecutionState {
    fn before_check(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        let mut kernel = SkillExecutionKernel::begin(dispatch, started);
        kernel.clear_phase_for_end();
        Self { kernel, flight: SpiderWebProgress::new(0) }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }

    pub(crate) fn prepare_derived_end(&mut self, _argument: i32) -> bool {
        self.flight = SpiderWebProgress::new(0);
        true
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpiderWebProgress {
    missile_flying_time_ms: u32,
}

impl SpiderWebProgress {
    pub(crate) const fn new(missile_flying_time_ms: u32) -> Self {
        Self { missile_flying_time_ms }
    }

    pub(crate) const fn missile_flying_time_ms(self) -> u32 {
        self.missile_flying_time_ms
    }
}

fn stage(skill: &MoveShapeSkill) -> Option<SkillStage> {
    skill.player_state::<PlayerSpiderWebExecutionState>().map(|state| state.kernel.stage())
        .or_else(|| skill.monster_kernel().map(|kernel| kernel.stage()))
}

fn advance(skill: &mut MoveShapeSkill, from: SkillStage, to: SkillStage) -> bool {
    if let Some(state) = skill.player_state_mut::<PlayerSpiderWebExecutionState>() {
        state.kernel.advance(from, to)
    } else {
        skill.monster_kernel_mut().is_some_and(|kernel| kernel.advance(from, to))
    }
}

fn flight(skill: &MoveShapeSkill) -> u32 {
    skill.player_state::<PlayerSpiderWebExecutionState>().map(|state| &state.flight)
        .or_else(|| skill.monster_progress::<SpiderWebProgress>())
        .map_or(0, |progress| progress.missile_flying_time_ms())
}

fn set_flight(skill: &mut MoveShapeSkill, milliseconds: u32) {
    if let Some(state) = skill.player_state_mut::<PlayerSpiderWebExecutionState>() {
        state.flight = SpiderWebProgress::new(milliseconds);
    } else {
        skill.set_monster_progress(SpiderWebProgress::new(milliseconds));
    }
}

pub(crate) fn publish_spider_web_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.owner() != SkillOwner::CSpiderWeb
        || skill.visual_effect().is_none_or(|effect| {
            effect.kind() != SkillVisualEffectKind::SpiderWeb || effect.is_ended()
        })
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region, identity).map(|shape| shape.shape()) else { return; };
    let mut message = CMessage::new(0x000b_fe01);
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 15) {
        if source.identity().object_type == 400 {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }
    let target = match mode {
        0 => None,
        1 => {
            let Some((region, identity)) = resolve_skill_sufferer(game, skill.lifecycle()) else { return; };
            let Some(target) = resolve_state_move_shape(game, region, identity) else { return; };
            Some(target.shape())
        }
        _ => return,
    };
    message.add_byte(if mode == 0 { 1 } else { 2 });
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if let Some(target) = target {
        message.add_long(target.identity().object_type);
        message.add_long(target.identity().id);
        message.add_long(target.get_tile_x().unwrap_or(i32::MIN));
        message.add_long(target.get_tile_y().unwrap_or(i32::MIN));
        message.add_ulong(flight(skill));
    } else {
        message.add_long(source.get_direction());
    }
    if let Some(region) = game.find_region(source.get_region_id()) {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}

fn install_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    user: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
    runtime: &mut Runtime,
) {
    let Some(shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    if shape.has_state_by_skill_id(CURE_SKILL_ID) { return; }
    let target_region = shape.shape().get_region_id();
    let Some(target_level) = game.move_shape_level(target_region, target.1) else { return; };
    let Some(source_level) = game.move_shape_level(user.0, user.1) else { return; };
    let multiplier = i32::from(source_level)
        .wrapping_sub(i32::from(target_level))
        .wrapping_add(properties.query_property(SKILL_USAGE_CONST) as i32)
        .max(1);
    let keep = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME).wrapping_mul(multiplier as u32);
    let state = SpiderWebState::new(0, keep);
    if let Some((position, _)) = resolve_state_move_shape(game, target_region, target.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == SPIDER_WEB_SKILL_ID))
    {
        let _ = end_and_destroy_state_at(game, target_region, target.1, position);
    }
    let _ = begin_primary_blind_state(
        game, target_region, target.1, Some(user), Some((target_region, target.1)),
        state, &mut || runtime.now_milliseconds(),
    );
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn end<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, address: RegisteredSkill, success: bool, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let termination = if success { SkillTermination::Completed } else { SkillTermination::Rejected };
    let _ = game.end_registered_instance(address, i32::from(success), termination, runtime);
    terminal(if success { QueuedSkillExecutionState::Completed } else { QueuedSkillExecutionState::Rejected })
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
) -> bool {
    let Some(skill) = game.registered_skill(address) else { return false; };
    let (region, identity) = skill.lifecycle().user();
    if resolve_state_move_shape(game, region, identity).is_none()
        || resolve_skill_sufferer(game, skill.lifecycle()).is_none()
    { return false; }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_SKILL_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(address, 13);
        return false;
    }
    let path = game.skill_target_path(skill.lifecycle());
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
        && properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) < path.len() as u32
    {
        game.update_registered_skill_visual(address, 11);
        return false;
    }
    if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
        game.update_registered_skill_visual(address, 15);
        return false;
    }
    if let Some(user) = resolve_state_move_shape_mut(game, region, identity) {
        user.set_moveable(false);
    }
    true
}

fn begin<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill_mut(address) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    skill.replace_visual_effect(SkillVisualEffect::new(SkillVisualEffectKind::SpiderWeb, 1));
    if !check_cast(game, address, runtime) {
        game.update_registered_skill_visual(address, 2);
        return end(game, address, false, runtime);
    }
    if let Some(skill) = game.registered_skill_mut(address) {
        set_flight(skill, 0);
        let _ = advance(skill, SkillStage::Idle, SkillStage::Begin);
    }
    terminal(QueuedSkillExecutionState::Begun)
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(address) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if stage(skill).is_none_or(|stage| stage == SkillStage::Idle) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return end(game, address, false, runtime);
    };
    let (region, identity) = skill.lifecycle().user();
    let user = resolve_state_move_shape(game, region, identity)
        .map(|shape| (shape.shape().get_region_id(), shape.shape().identity()));
    let target = resolve_skill_sufferer(game, skill.lifecycle()).and_then(|(region, identity)| {
        resolve_state_move_shape(game, region, identity)
            .map(|shape| (shape.shape().get_region_id(), shape.shape().identity()))
    });
    let (Some(user), Some(target)) = (user, target) else {
        return end(game, address, false, runtime);
    };
    if game.move_shape_health(target.0, target.1) == Some(0) {
        game.update_registered_skill_visual(address, 10);
        return end(game, address, false, runtime);
    }
    if game.registered_skill(address).and_then(stage) == Some(SkillStage::Begin) {
        if let Some(skill) = game.registered_skill_mut(address) {
            skill.lifecycle_mut().set_available(properties.query_property(SKILL_USAGE_CAN_BE_BREAKED) != 0);
        }
        let direction = (|| {
            let source = resolve_state_move_shape(game, user.0, user.1)?.shape();
            let target = resolve_state_move_shape(game, target.0, target.1)?.shape();
            Some(get_line_direction(
                source.get_tile_x().unwrap_or(i32::MIN), source.get_tile_y().unwrap_or(i32::MIN),
                target.get_tile_x().unwrap_or(i32::MIN), target.get_tile_y().unwrap_or(i32::MIN),
            ))
        })();
        if let Some(direction) = direction
            && let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1)
        {
            source.shape_mut().set_direction(direction);
        }
        game.update_registered_skill_visual(address, 0);
        if let Some(skill) = game.registered_skill_mut(address) {
            let _ = advance(skill, SkillStage::Begin, SkillStage::Check);
        }
    }
    if game.registered_skill(address).and_then(stage) == Some(SkillStage::Check) {
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let Some(started) = game.registered_skill(address).map(|skill| skill.lifecycle().started_at_ms()) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if runtime.now_milliseconds() < started.wrapping_add(delay) {
            return terminal(QueuedSkillExecutionState::Pending);
        }
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) {
            source.set_moveable(true);
        }
        let source_level = game.move_shape_level(user.0, user.1);
        let target_level = game.move_shape_level(target.0, target.1);
        let (Some(source_level), Some(target_level)) = (source_level, target_level) else {
            return end(game, address, false, runtime);
        };
        if u32::from(source_level) + 10 < u32::from(target_level) {
            game.update_registered_skill_visual(address, 2);
            return end(game, address, true, runtime);
        }
        let Some(skill) = game.registered_skill(address) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.skill_target_path(skill.lifecycle());
        if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
            && properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) < path.len() as u32
        {
            game.update_registered_skill_visual(address, 11);
            return end(game, address, false, runtime);
        }
        if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
            game.update_registered_skill_visual(address, 15);
            return end(game, address, false, runtime);
        }
        let milliseconds = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME)
            .wrapping_mul(path.len() as u32);
        if let Some(skill) = game.registered_skill_mut(address) {
            set_flight(skill, milliseconds);
        }
        game.update_registered_skill_visual(address, 1);
        if let Some(skill) = game.registered_skill_mut(address) {
            let _ = advance(skill, SkillStage::Check, SkillStage::Calculate);
            skill.lifecycle_mut().mark_prepared();
        }
    }
    let Some(skill) = game.registered_skill(address) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if stage(skill) != Some(SkillStage::Calculate) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let deadline = skill.lifecycle().started_at_ms().wrapping_add(flight(skill)).wrapping_add(delay);
    if runtime.now_milliseconds() < deadline {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    install_state(game, user, target, &properties, runtime);
    end(game, address, true, runtime)
}

pub(crate) const fn is_player_spider_web_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::Point { skill_id: SPIDER_WEB_SKILL_ID, .. }
            | PlayerSkillDispatch::Object { skill_id: SPIDER_WEB_SKILL_ID, .. }
            | PlayerSkillDispatch::SelfTarget { skill_id: SPIDER_WEB_SKILL_ID, .. })
}

pub(crate) fn cancel_player_spider_web<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI,
    nonzero_end: bool, runtime: &mut Runtime,
) -> bool {
    let Some(address) = game.registered_player_skill(player_id, SPIDER_WEB_SKILL_ID) else { return false; };
    let Some(dispatch) = game.registered_skill(address).and_then(MoveShapeSkill::player_dispatch) else { return false; };
    game.with_published_player_ai(player_id, ai, |game| {
        let _ = game.end_registered_instance(
            address, i32::from(nonzero_end), SkillTermination::Cancelled, runtime,
        );
    });
    game.finish_registered_player_command(Some(address), ai, dispatch, SkillTermination::Cancelled)
}

pub(crate) fn execute_player_spider_web<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_player_spider_web_dispatch(dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(address) = game.registered_player_skill(player_id, SPIDER_WEB_SKILL_ID) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(skill) = game.registered_skill(address) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if skill.player_state::<PlayerSpiderWebExecutionState>().is_none() {
        let started = skill.lifecycle().started_at_ms();
        if !game.begin_player_skill_execution(
            player_id, PlayerSpiderWebExecutionState::before_check(dispatch, started),
        ) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        return game.with_published_player_ai(player_id, ai, |game| begin(game, address, runtime));
    }
    if skill.player_dispatch() != Some(dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    game.with_published_player_ai(player_id, ai, |game| run_ai(game, address, runtime))
}

pub(crate) fn execute_owned_spider_web<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target_identity: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    let Some(region_owner) = owner.as_mut() else { return false; };
    let Some(monster) = region_owner.base().find_monster_by_id(monster_id) else { return false; };
    let region_id = monster.move_shape().shape().get_region_id();
    let source = monster.move_shape().shape().identity();
    let cast = monster.current_active_attack_cast(game.skill_factory());
    if let Some(cast) = cast {
        if cast.dispatch().skill_id != SPIDER_WEB_SKILL_ID { return false; }
        return game.with_published_region(owner, |game| {
            let Some(address) = game.registered_move_shape_skill(region_id, source, SPIDER_WEB_SKILL_ID) else { return false; };
            let _ = run_ai(game, address, runtime);
            true
        }).unwrap_or(false);
    }
    let target_object = resolve_owned_skill_begin_object(game, region_owner.base(), target_identity);
    let started = runtime.now_milliseconds();
    if !region_owner.base_mut().find_monster_by_id_mut(monster_id).is_some_and(|monster| {
        monster.prepare_base_attack_cast(
            target_identity, SPIDER_WEB_SKILL_ID, skill_level, started, target_object, game.skill_factory(),
        )
    }) { return false; }
    let outcome = game.with_published_region(owner, |game| {
        let Some(address) = game.registered_move_shape_skill(region_id, source, SPIDER_WEB_SKILL_ID) else {
            return MonsterSkillCallOutcome::NotHandled;
        };
        let Some(skill) = game.registered_skill_mut(address) else {
            return MonsterSkillCallOutcome::NotHandled;
        };
        let Some(kernel) = skill.monster_kernel_mut() else {
            return MonsterSkillCallOutcome::NotHandled;
        };
        kernel.clear_phase_for_end();
        set_flight(skill, 0);
        if begin(game, address, runtime).state != QueuedSkillExecutionState::Begun {
            return MonsterSkillCallOutcome::BeginRejected;
        }
        if let Some(monster) = game.find_region_mut(region_id)
            .and_then(|region| region.base_mut().find_monster_by_id_mut(monster_id))
        {
            monster.enqueue_base_attack_cast(runtime.now_milliseconds());
        }
        MonsterSkillCallOutcome::Handled
    }).unwrap_or(MonsterSkillCallOutcome::Handled);
    let Some(region_owner) = owner.as_mut() else { return true; };
    finish_monster_skill_call(game, region_owner.base_mut(), monster_id, outcome, runtime)
}
