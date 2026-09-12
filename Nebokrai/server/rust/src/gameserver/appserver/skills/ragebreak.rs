//! CRageBreak (0x6E): gameserver.exe + GameServer.pdb, appserver/skills/ragebreak.cpp.
//! Общий stateskill обслуживает три Begin, registered visual и полный End;
//! RP-подготовка в fury.rs проверяет U и срок эффекта; предметный хвост
//! накладывает усиление и затем Cure.
//! Источник не обязан быть игроком: RTTI ограничивает только RP и сообщения.
//! Состояния принадлежат арене источника и не завершаются вместе с навыком.

use std::ops::ControlFlow;

use super::curestate::{CURE_STATE_SKILL_ID, CureState, begin_primary_cure_state};
use super::fury::{
    RageRpPolicy, RageSkillEffect, check_rage_skill_cast, prepare_rage_skill_effect,
    remove_reached_conflict_states,
};
use super::kernel::SkillTermination;
use super::ragebreakstate::{RageBreakState, begin_primary_rage_break_state};
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, StateSkillVisualTarget, end_state_skill,
    execute_owned_state_skill, execute_player_state_skill, finish_player_state_skill,
    publish_state_skill_visual, state_skill_outcome,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, end_move_shape_state, resolve_state_move_shape,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner,
};

pub(crate) const RAGE_BREAK_SKILL_ID: u32 = 0x6e;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_ATTACK_GAIN: u32 = 105;

pub(crate) fn is_rage_break_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == RAGE_BREAK_SKILL_ID
}

struct RageBreak;

impl RegisteredStateSkill for RageBreak {
    const ID: u32 = RAGE_BREAK_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::RageBreak;
    const VISUAL_FAILURES: &'static [u32] = &[2, 7, 8, 13];
    const VISUAL_DWORD_FAILURES: &'static [u32] = &[8];
    const VISUAL_TARGET: StateSkillVisualTarget = StateSkillVisualTarget::User;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, _target: StateSkillBeginTarget,
        runtime: &mut Runtime,
    ) -> bool {
        check_rage_skill_cast(game, instance, RageRpPolicy::RequirePositive, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let effect = match prepare_rage_skill_effect(game, instance, runtime) {
            ControlFlow::Continue(effect) => effect,
            ControlFlow::Break(outcome) => return outcome,
        };
        match apply_rage_break_effect(game, effect, runtime) {
            Some(argument) => end_state_skill(game, instance, argument, runtime),
            None => state_skill_outcome(QueuedSkillExecutionState::Pending),
        }
    }
}

pub(crate) fn publish_rage_break_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    publish_state_skill_visual::<RageBreak>(game, skill, mode);
}

pub(crate) fn cancel_player_rage_break<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI,
    nonzero_end: bool, runtime: &mut Runtime,
) -> bool {
    finish_player_state_skill::<RageBreak, Runtime>(
        game, player_id, player_ai, i32::from(nonzero_end), SkillTermination::Cancelled, runtime,
    )
}

pub(crate) fn execute_player_rage_break<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_state_skill::<RageBreak, Runtime>(game, player_id, dispatch, player_ai, runtime)
}

pub(crate) fn execute_owned_monster_rage_break<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<RageBreak, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}

fn apply_rage_break_effect<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, effect: RageSkillEffect, runtime: &mut Runtime,
) -> Option<i32> {
    let RageSkillEffect { source, properties } = effect;
    let previous = resolve_state_move_shape(game, source.0, source.1)?
        .find_state_position(|state| state.state_id() == RAGE_BREAK_SKILL_ID);
    if let Some((position, _)) = previous {
        let _ = end_and_destroy_state_at(game, source.0, source.1, position);
    }
    let keep = properties.query_property(STATE_PERSIST_TIME);
    let gain = properties.query_property(TARGET_ATTACK_GAIN) as i32;
    let state = RageBreakState::new(keep, gain);
    let _ = begin_primary_rage_break_state(
        game, source.0, source.1, Some(source), Some(source), state,
        &mut || runtime.now_milliseconds(),
    );
    remove_reached_conflict_states(game, source.0, source.1);

    // Для прежнего Cure вызывается только первый End, без внешнего destructor;
    // новый экземпляр всегда добавляется в хвост после позднего запроса срока.
    let previous = resolve_state_move_shape(game, source.0, source.1)?
        .find_state_position(|state| state.state_id() == CURE_STATE_SKILL_ID);
    if let Some((_, key)) = previous { let _ = end_move_shape_state(game, source.0, source.1, key); }
    let cure = CureState::new(properties.query_property(STATE_PERSIST_TIME));
    let _ = begin_primary_cure_state(
        game, source.0, source.1, Some(source), Some(source), cure,
        &mut || runtime.now_milliseconds(),
    );
    let _ = game.update_move_shape_properties(source.0, source.1);
    Some(1)
}
