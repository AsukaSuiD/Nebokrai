//! Сбор душ `CSoulCollect` (`0x13B`).
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/soulcollect.cpp`.
//!
//! Общий зарегистрированный Begin создаёт visual loop1 перед Check. Таблица и
//! reuse проверяются для любого U; только Player проходит MP/Move0-ветку,
//! причём нулевая цена молча отвергается. Первый AI разрешает U, затем S, не
//! проверяет смерть, у Player повторно списывает MP и публикует состояния,
//! задаёт CAN и visual0. После unsigned-срока `start + delay` visual1
//! передаёт накопление `SoulCollectState`; helper сам сохраняет первый
//! типизированный слот, Begin(U,U), AddSoul и state-visual. Его результат не
//! меняет исходный End(1). Один owner обслуживает Player и Monster; factory
//! USER_OR_SUFFERER_RESET_PHASE очищает фазу и возвращает движение свежему U
//! либо S при общем End.

use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{check_cast_mana, spend_cast_mana, terminal};
use super::soulcollectstate::{SoulCollectState, add_soul_collect};
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, StateSkillVisualTarget, end_state_skill,
    execute_owned_state_skill,
};
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner,
};

pub(crate) const SOUL_COLLECT_SKILL_ID: u32 = 0x13b;
const PLAYER_TYPE: i32 = 400;
const PARAMETER_PERCENT: u32 = 20_020;

fn resolved_user(game: &CGame, skill: &MoveShapeSkill) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, region, identity)?.shape();
    Some((source.get_region_id(), source.identity()))
}

fn resolved_source(game: &CGame, skill: &MoveShapeSkill) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = skill.lifecycle().user();
    resolve_state_move_shape(game, region, identity).or_else(|| {
        let (region, identity) = resolve_skill_sufferer(game, skill.lifecycle())?;
        resolve_state_move_shape(game, region, identity)
    }).map(|source| (source.shape().get_region_id(), source.shape().identity()))
}

fn check_soul_collect_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
) -> bool {
    let Some((region, identity)) = original_user else { return false; };
    let Some(source) = resolve_state_move_shape(game, region, identity)
        .map(|source| (source.shape().get_region_id(), source.shape().identity()))
    else { return false; };
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return false;
    };
    if !skill_is_restored(
        skill.last_used_ms(), properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
        runtime.now_milliseconds(),
    ) {
        if let Some(player) = player {
            game.update_registered_skill_visual(instance, 13);
            game.send_skill_system_info(player, b"GS0278");
        }
        return false;
    }
    check_cast_mana(game, instance, source, &properties)
}

fn run_soul_collect_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(source) = resolved_source(game, skill) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);

    if stage == SkillStage::Begin {
        if !spend_cast_mana(game, instance, player, &properties) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(skill) = game.registered_skill_mut(instance) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        skill.lifecycle_mut().set_available(properties.query_property(SKILL_USAGE_CAN_BE_BREAKED) != 0);
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }

    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(properties.query_property(SKILL_USAGE_DELAY_TIME)) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    game.update_registered_skill_visual(instance, 1);
    let _ = add_soul_collect(game, source, |game| {
        let level = game.registered_skill(instance)?.level();
        Some(SoulCollectState::new(level, properties.query_property(PARAMETER_PERCENT)))
    }, &mut || runtime.now_milliseconds());
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_soul_collect<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != SOUL_COLLECT_SKILL_ID {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::SelfCast,
        |game, instance, _, runtime| check_soul_collect_cast(game, instance, original_user, runtime),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_soul_collect_ai,
    )
}

struct SoulCollectSkill;

impl RegisteredStateSkill for SoulCollectSkill {
    const ID: u32 = SOUL_COLLECT_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::SelfCast;
    const VISUAL_FAILURES: &'static [u32] = &[2, 7, 13];
    const VISUAL_TARGET: StateSkillVisualTarget = StateSkillVisualTarget::User;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, _begin_target: StateSkillBeginTarget,
        runtime: &mut Runtime,
    ) -> bool {
        let user = game.registered_skill(instance).and_then(|skill| resolved_user(game, skill));
        check_soul_collect_cast(game, instance, user, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = run_soul_collect_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => end_state_skill(game, instance, 0, runtime),
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse =>
                end_state_skill(game, instance, 1, runtime),
            _ => outcome,
        }
    }
}

pub(crate) fn execute_owned_monster_soul_collect<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<SoulCollectSkill, Runtime>(
        game, owner, monster_id, target, skill_level, runtime,
    )
}
