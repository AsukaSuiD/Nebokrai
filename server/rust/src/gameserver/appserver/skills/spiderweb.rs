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
use super::kernel::{skill_is_restored, PlayerSkillExecution, SkillExecutionKernel, SkillStage, SkillTermination};
use super::skillbaseproperties::CSkillBaseProperties;
use super::spiderwebstate::SpiderWebState;
use super::stateskill::{
    RegisteredStateSkill, end_state_skill, execute_owned_state_skill, execute_player_state_skill,
    finish_player_state_skill, publish_state_skill_visual, state_skill_outcome as terminal,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_skill_sufferer,
    resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner,
};
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
    publish_state_skill_visual::<SpiderWebSkill>(game, skill, mode);
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

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, address: RegisteredSkill,
    begin_target: super::stateskill::StateSkillBeginTarget, runtime: &mut Runtime,
) -> bool {
    let Some(skill) = game.registered_skill(address) else { return false; };
    let (region, identity) = skill.lifecycle().user();
    if resolve_state_move_shape(game, region, identity).is_none()
        || begin_target.resolve(game, skill, false).is_none()
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

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(address) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return end_state_skill(game, address, 0, runtime);
    };
    let (region, identity) = skill.lifecycle().user();
    let user = resolve_state_move_shape(game, region, identity)
        .map(|shape| (shape.shape().get_region_id(), shape.shape().identity()));
    let target = resolve_skill_sufferer(game, skill.lifecycle()).and_then(|(region, identity)| {
        resolve_state_move_shape(game, region, identity)
            .map(|shape| (shape.shape().get_region_id(), shape.shape().identity()))
    });
    let (Some(user), Some(target)) = (user, target) else {
        return end_state_skill(game, address, 0, runtime);
    };
    if game.move_shape_health(target.0, target.1) == Some(0) {
        game.update_registered_skill_visual(address, 10);
        return end_state_skill(game, address, 0, runtime);
    }
    if game.registered_skill(address).and_then(MoveShapeSkill::execution_stage) == Some(SkillStage::Begin) {
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
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    if game.registered_skill(address).and_then(MoveShapeSkill::execution_stage) == Some(SkillStage::Check) {
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
            return end_state_skill(game, address, 0, runtime);
        };
        if u32::from(source_level) + 10 < u32::from(target_level) {
            game.update_registered_skill_visual(address, 2);
            return end_state_skill(game, address, 1, runtime);
        }
        let Some(skill) = game.registered_skill(address) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let path = game.skill_target_path(skill.lifecycle());
        if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0
            && properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) < path.len() as u32
        {
            game.update_registered_skill_visual(address, 11);
            return end_state_skill(game, address, 0, runtime);
        }
        if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
            game.update_registered_skill_visual(address, 15);
            return end_state_skill(game, address, 0, runtime);
        }
        let milliseconds = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME)
            .wrapping_mul(path.len() as u32);
        if let Some(skill) = game.registered_skill_mut(address) {
            set_flight(skill, milliseconds);
        }
        game.update_registered_skill_visual(address, 1);
        if let Some(skill) = game.registered_skill_mut(address) {
            let _ = skill.advance_execution(SkillStage::Check, SkillStage::Calculate);
            skill.lifecycle_mut().mark_prepared();
        }
    }
    let Some(skill) = game.registered_skill(address) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if skill.execution_stage() != Some(SkillStage::Calculate) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let deadline = skill.lifecycle().started_at_ms().wrapping_add(flight(skill)).wrapping_add(delay);
    if runtime.now_milliseconds() < deadline {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    install_state(game, user, target, &properties, runtime);
    end_state_skill(game, address, 1, runtime)
}

struct SpiderWebSkill;

impl RegisteredStateSkill for SpiderWebSkill {
    const ID: u32 = SPIDER_WEB_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::SpiderWeb;

    fn visual_flight_time(skill: &MoveShapeSkill) -> Option<u32> {
        Some(flight(skill))
    }

    fn player_execution(dispatch: PlayerSkillDispatch, started: u32) -> PlayerSkillExecution {
        PlayerSpiderWebExecutionState::before_check(dispatch, started).into()
    }

    fn prepare_monster(skill: &mut MoveShapeSkill) {
        set_flight(skill, 0);
    }

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, begin_target: super::stateskill::StateSkillBeginTarget, runtime: &mut Runtime,
    ) -> bool {
        check_cast(game, address, begin_target, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        run_ai(game, address, runtime)
    }
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
    finish_player_state_skill::<SpiderWebSkill, Runtime>(
        game, player_id, ai, i32::from(nonzero_end), SkillTermination::Cancelled, runtime,
    )
}

pub(crate) fn execute_player_spider_web<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_state_skill::<SpiderWebSkill, Runtime>(game, player_id, dispatch, ai, runtime)
}

pub(crate) fn execute_owned_spider_web<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target_identity: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<SpiderWebSkill, Runtime>(
        game, owner, monster_id, target_identity, skill_level, runtime,
    )
}
