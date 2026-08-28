//! Второй вариант области инь-ян `CYinYang2` (`0x146`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/yinyang2.cpp`. `Begin`, визуальный протокол, проверки,
//! AI и usage-свойства совпадают с `CYinYang`; общий cast-owner находится в
//! `yinyang.rs`. Отличающаяся постоянная маска 1×1 подтверждена глобалами
//! `0x006A4DFC/0x006A4DFD/0x006A4DFE` и выбирается владельцем формы по ID.

use super::yinyang::execute_player_yin_yang_family;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome};

pub(crate) const YIN_YANG_2_SKILL_ID: u32 = 0x146;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;

pub(crate) const fn is_yin_yang_2_target(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch,
        PlayerSkillDispatch::SelfTarget { skill_id: YIN_YANG_2_SKILL_ID, .. }
        | PlayerSkillDispatch::Point { skill_id: YIN_YANG_2_SKILL_ID, .. }
        | PlayerSkillDispatch::Object { skill_id: YIN_YANG_2_SKILL_ID, target: ShapeIdentity { object_type: PLAYER_TYPE | MONSTER_TYPE, .. } }
    )
}

pub(crate) fn execute_player_yin_yang_2<R: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI, runtime: &mut R,
) -> QueuedSkillExecutionOutcome {
    execute_player_yin_yang_family(game, player_id, dispatch, ai, runtime, YIN_YANG_2_SKILL_ID, true)
}
