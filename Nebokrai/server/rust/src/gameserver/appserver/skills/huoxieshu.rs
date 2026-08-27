//! Передача здоровья игрока боевому духу (`CHuoxieshu`, навык `0x222`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/huoxieshu.cpp`. Навык оставляет игроку минимум одно HP;
//! отличающаяся формула принадлежит варианту `Health`, а общий pipeline пары
//! `Huoxieshu/Lingzhishu` находится в узком `battlefairytransfer` owner-е.

use super::battlefairytransfer::{
    execute_battle_fairy_transfer, BattleFairyTransferKind,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
};

pub(crate) const HUOXIESHU_SKILL_ID: u32 = 0x222;

pub(crate) fn execute_battle_fairy_huoxieshu<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: BattleFairySkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_battle_fairy_transfer(
        game,
        player_id,
        dispatch,
        BattleFairyTransferKind::Health,
        player_ai,
        runtime,
    )
}
