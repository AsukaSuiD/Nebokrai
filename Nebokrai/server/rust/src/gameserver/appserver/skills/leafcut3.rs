//! Вариант 3 удара листвы без расхода RP: старое состояние завершается
//! до чтения боевых свойств, новое добавляется в конец общей арены.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/leafcut3.cpp.

use super::leafcut::{cancel_leaf_cut_family, execute_leaf_cut_family};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome};

pub(crate) const LEAF_CUT_3_SKILL_ID: u32 = 0x8f;

pub(crate) fn is_leaf_cut_3_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == LEAF_CUT_3_SKILL_ID
}

pub(crate) fn cancel_player_leaf_cut_3<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    cancel_leaf_cut_family::<LEAF_CUT_3_SKILL_ID, Runtime>(game, player_id, ai, runtime)
}

pub(crate) fn execute_player_leaf_cut_3<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_leaf_cut_family::<LEAF_CUT_3_SKILL_ID, Runtime>(game, player_id, dispatch, ai, runtime)
}
