//! Второй божественный гром `CGodThunder2` (`0x143`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/godthunder2.cpp`. Его `Begin`, `CheckCastCondition`, AI,
//! визуальный протокол и набор usage-свойств совпадают с `CGodThunder`; общий
//! семейный cast-owner находится в `godthunder.rs`. Отличающиеся маска 7×7,
//! war-soul-first обход и concrete phalanx остаются у этого варианта.

use super::godthunder::execute_player_god_thunder_family;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome};

pub(crate) const GOD_THUNDER_2_SKILL_ID: u32 = 0x143;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

pub(crate) const fn is_god_thunder_2_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::SelfTarget { skill_id: GOD_THUNDER_2_SKILL_ID, .. }
        | PlayerSkillDispatch::Point { skill_id: GOD_THUNDER_2_SKILL_ID, .. }
        | PlayerSkillDispatch::Object { skill_id: GOD_THUNDER_2_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } }
    )
}

pub(crate) fn execute_player_god_thunder_2<R: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI, runtime: &mut R,
) -> QueuedSkillExecutionOutcome {
    execute_player_god_thunder_family(
        game, player_id, dispatch, ai, runtime, GOD_THUNDER_2_SKILL_ID, true,
    )
}
