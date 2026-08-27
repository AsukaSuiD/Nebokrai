//! Передача маны игрока боевому духу (`CLingzhishu`, навык `0x223`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/lingzhishu.cpp`. Начальная проверка допускает ровно цену
//! навыка, но повторная AI-проверка требует оставить одну MP. Это отличие
//! сохранено вариантом `Mana` общего узкого `battlefairytransfer` owner-а.

use super::battlefairytransfer::{
    execute_battle_fairy_transfer, BattleFairyTransferKind,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
};

pub(crate) const LINGZHISHU_SKILL_ID: u32 = 0x223;

pub(crate) fn execute_battle_fairy_lingzhishu<Runtime: GameMainLoopRuntime>(
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
        BattleFairyTransferKind::Mana,
        player_ai,
        runtime,
    )
}
