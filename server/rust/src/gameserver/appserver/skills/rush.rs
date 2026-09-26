//! Прямые рывки Rush/Rush2. Источник: gameserver.exe/GameServer.pdb,
//! appserver/skills/rush.cpp и общий предметный контракт rush2.cpp.
//! Тела Check/AI, AddRushState-форма и visual перенесены буквально в Zone
//! `skills/rush.rs` (порция №5 «player melee»; основание и машинные статусы
//! см. там). Здесь — тонкие делегации с прежними сигнатурами:
//! зарегистрированный вход идёт общим playercast, швы Zone реализованы над
//! `CGame` в `skills/dash.rs`; потребители не меняются.

use super::dash::dash_skill_outcome;
use super::kernel::RushExecutionState;
use super::playercast::execute_registered_player_cast;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    game_tick_milliseconds,
};
use nebokrai_zone::skills::rush;

pub(crate) use nebokrai_zone::skills::rush::RUSH_SKILL_ID;

pub(super) fn scaled_state_time(source_level: u8, target_level: u8, base_time: u32) -> u32 {
    rush::scaled_state_time(source_level, target_level, base_time)
}

pub(crate) fn publish_rush_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    rush::publish_rush_visual(game, skill, mode)
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    player_id: i32,
    runtime: &mut Runtime,
) -> bool {
    let _ = runtime;
    rush::check_cast(game, instance, player_id, game_tick_milliseconds)
}

fn rush_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    dash_skill_outcome(rush::run_ai(game, instance, runtime, game_tick_milliseconds))
}

pub(super) fn execute_rush<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, rush::visual_kind(dispatch.skill_id()),
        check_cast, |dispatch, started| RushExecutionState::begin(dispatch, started).into(), rush_ai,
    )
}

pub(crate) fn execute_player_rush<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != RUSH_SKILL_ID {
        return QueuedSkillExecutionOutcome { state: QueuedSkillExecutionState::Rejected, first_contact: false };
    }
    execute_rush(game, player_id, instance, dispatch, runtime)
}
