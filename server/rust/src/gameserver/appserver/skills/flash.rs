//! Рывок CFlash (0x69). Источник: gameserver.exe/GameServer.pdb,
//! appserver/skills/flash.cpp. Тела Check/AI, helpers и visual перенесены
//! буквально в Zone `skills/flash.rs` (порция №5 «player melee»; основание и
//! машинные статусы см. там). Здесь — тонкие делегации с прежними
//! сигнатурами: зарегистрированный вход идёт общим playercast, швы Zone
//! реализованы над `CGame` в `skills/dash.rs`; потребители не меняются.

use super::dash::dash_skill_outcome;
use super::kernel::FlashExecutionState;
use super::playercast::execute_registered_player_cast;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{ShapeIdentity, ShapeView};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    game_tick_milliseconds,
};
use nebokrai_zone::skills::dash::DashSkillGame;
use nebokrai_zone::skills::flash;

pub(crate) use nebokrai_zone::skills::flash::FLASH_SKILL_ID;

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    player_id: i32,
    runtime: &mut Runtime,
) -> bool {
    let _ = runtime;
    flash::check_cast(game, instance, player_id, game_tick_milliseconds)
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    dash_skill_outcome(flash::run_ai(game, instance, runtime, game_tick_milliseconds))
}

pub(crate) fn execute_player_flash<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != FLASH_SKILL_ID {
        return QueuedSkillExecutionOutcome { state: QueuedSkillExecutionState::Rejected, first_contact: false };
    }
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::Flash,
        check_cast, |dispatch, started| FlashExecutionState::begin(dispatch, started).into(), run_ai,
    )
}

pub(crate) fn publish_flash_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    flash::publish_flash_visual(game, skill, mode)
}

pub(crate) fn master_info(player: &CPlayer) -> MasterInfo {
    flash::master_info(player)
}

pub(super) fn target_level(game: &CGame, region_id: i32, target: ShapeIdentity) -> Option<u8> {
    flash::target_level(game, region_id, target)
}

pub(super) fn cell_views(game: &CGame, region_id: i32, x: i32, y: i32) -> Vec<ShapeView> {
    game.dash_cell_views(region_id, x, y)
}
