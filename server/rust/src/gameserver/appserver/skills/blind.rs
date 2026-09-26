//! Ослепление CBlind (0x76). Источник: gameserver.exe/GameServer.pdb,
//! appserver/skills/blind.cpp. Тела kernel-входа, visual и AddBlindState
//! перенесены буквально в Zone `skills/blind.rs` (порция №6c «self/zone-касты»;
//! основание и машинные статусы см. там). Здесь — тонкие делегации с прежними
//! сигнатурами: швы Zone реализованы над `CGame` в `skills/selfcast.rs`;
//! потребители не меняются.

use super::selfcast::selfcast_outcome;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, game_tick_milliseconds,
};
use nebokrai_zone::skills::blind;

pub(crate) use nebokrai_zone::skills::blind::BLIND_SKILL_ID;

pub(crate) const fn is_blind_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    blind::is_blind_dispatch(dispatch)
}

pub(crate) fn complete_player_blind<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime,
) -> bool {
    blind::complete_blind(game, player_id, ai)
}

pub(crate) fn cancel_player_blind<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, _runtime: &mut Runtime,
) -> bool {
    blind::cancel_blind(game, player_id, ai)
}

pub(crate) fn publish_blind_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    blind::publish_blind_visual(game, skill, mode)
}

pub(crate) fn execute_player_blind<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    selfcast_outcome(blind::run_blind(game, player_id, dispatch, ai, runtime, game_tick_milliseconds))
}
