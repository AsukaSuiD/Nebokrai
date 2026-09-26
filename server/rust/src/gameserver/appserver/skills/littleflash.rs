//! Малые рывки сквозь строй: LittleFlash и LittleFlash2.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/littleflash.cpp
//! и littleflash2.cpp. Тела Check/AI и visual перенесены буквально в Zone
//! `skills/littleflash.rs` (порция №5 «player melee»; основание и машинные
//! статусы см. там). Здесь — тонкие делегации с прежними сигнатурами:
//! зарегистрированный вход идёт общим playercast, швы Zone реализованы над
//! `CGame` в `skills/dash.rs`; потребители не меняются.

use super::dash::dash_skill_outcome;
use super::kernel::{LittleFlashExecutionState, PlayerSkillExecution};
use super::playercast::execute_registered_player_cast;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    game_tick_milliseconds,
};
use nebokrai_zone::skills::littleflash;

pub(crate) use nebokrai_zone::skills::littleflash::LITTLE_FLASH_SKILL_ID;

pub(crate) const fn is_little_flash_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    littleflash::is_little_flash_dispatch(dispatch)
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    player_id: i32,
    runtime: &mut Runtime,
) -> bool {
    let _ = runtime;
    littleflash::check_cast(game, instance, player_id, game_tick_milliseconds)
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    dash_skill_outcome(littleflash::run_ai(game, instance, runtime, game_tick_milliseconds))
}

pub(crate) fn execute_player_little_flash<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_little_flash_dispatch(dispatch) {
        return QueuedSkillExecutionOutcome { state: QueuedSkillExecutionState::Rejected, first_contact: false };
    }
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::LittleFlash,
        check_cast,
        |dispatch, started| PlayerSkillExecution::LittleFlash(LittleFlashExecutionState::begin(dispatch, started)),
        run_ai,
    )
}

pub(crate) fn publish_little_flash_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    littleflash::publish_little_flash_visual(game, skill, mode)
}
