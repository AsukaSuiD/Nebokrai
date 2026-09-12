//! Общий зарегистрированный вход активных навыков игрока.
//! Источник: gameserver.exe/GameServer.pdb, appserver/states/skill.cpp,
//! attackskill.cpp и совместимые Begin владельцев Flash, LittleFlash, Rush
//! и ArmyBreak. Игровые проверки и AI остаются у конкретных навыков.
//!
//! База Begin, материализация, visual и End используют один поколенческий
//! ключ. Проверка видит данные исполнения с выключенной фазой; успешный
//! Begin включает её без второго отсчёта времени. Общий End выполняется
//! до возврата расписанию, которое только освобождает данные и ту же команду.
//! Удалённый callback-ом экземпляр не заменяется новым совпадением ID.

use super::kernel::{PlayerSkillExecution, SkillStage, SkillTermination};
use super::stateskill::state_skill_outcome;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

fn finish_outcome<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, outcome: QueuedSkillExecutionOutcome,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let end = match outcome.state {
        QueuedSkillExecutionState::Rejected => Some((0, SkillTermination::Rejected)),
        QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse => {
            Some((1, SkillTermination::Completed))
        }
        QueuedSkillExecutionState::Begun | QueuedSkillExecutionState::Pending => None,
    };
    if let Some((argument, termination)) = end {
        let _ = game.end_registered_instance(instance, argument, termination, runtime);
    }
    outcome
}

pub(super) fn execute_registered_player_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime, visual_kind: SkillVisualEffectKind,
    check: impl FnOnce(&mut CGame, RegisteredSkill, i32, &mut Runtime) -> bool,
    materialize: impl FnOnce(PlayerSkillDispatch, u32) -> PlayerSkillExecution,
    run_ai: impl FnOnce(&mut CGame, RegisteredSkill, &mut Runtime) -> QueuedSkillExecutionOutcome,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if skill.id() != dispatch.skill_id() {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    if let Some(previous) = skill.player_dispatch() {
        if previous != dispatch {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        }
        let outcome = run_ai(game, instance, runtime);
        return finish_outcome(game, instance, outcome, runtime);
    }
    if !game.begin_registered_player_skill_with_combat(
        instance, player_id, dispatch, runtime.now_milliseconds(),
    ) {
        return finish_outcome(game, instance, state_skill_outcome(QueuedSkillExecutionState::Rejected), runtime);
    }
    let Some(skill) = game.registered_skill_mut(instance) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    skill.replace_visual_effect(SkillVisualEffect::new(visual_kind, 1));
    let mut execution = materialize(dispatch, skill.lifecycle().started_at_ms());
    execution.kernel_mut().clear_phase_for_end();
    if !skill.install_player_execution(execution) {
        return finish_outcome(game, instance, state_skill_outcome(QueuedSkillExecutionState::Rejected), runtime);
    }
    if !check(game, instance, player_id, runtime) {
        return finish_outcome(game, instance, state_skill_outcome(QueuedSkillExecutionState::Rejected), runtime);
    }
    if let Some(skill) = game.registered_skill_mut(instance) {
        let _ = skill.advance_execution(SkillStage::Idle, SkillStage::Begin);
    }
    state_skill_outcome(QueuedSkillExecutionState::Begun)
}
