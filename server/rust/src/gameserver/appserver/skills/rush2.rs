//! Второй прямой рывок CRush2 (0x7C).
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/rush2.cpp.
//! Геометрия, проверка меча, расход ресурсов и visual совпадают с Rush;
//! дальность попадания проверяется строго. AddRushState2-форма перенесена
//! буквально в Zone `skills/rush.rs` (порция №5 «player melee»; основание и
//! машинные статусы см. там). Здесь — тонкие делегации с прежними
//! сигнатурами; потребители не меняются.

use super::rush::execute_rush;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use nebokrai_zone::skills::rush;

pub(crate) use nebokrai_zone::skills::rush::RUSH_2_SKILL_ID;

pub(crate) const fn is_rush_2_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    rush::is_rush_2_dispatch(dispatch)
}

pub(crate) fn execute_player_rush_2<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_rush_2_dispatch(dispatch) {
        return QueuedSkillExecutionOutcome { state: QueuedSkillExecutionState::Rejected, first_contact: false };
    }
    execute_rush(game, player_id, instance, dispatch, runtime)
}
