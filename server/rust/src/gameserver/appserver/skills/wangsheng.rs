//! Тонкий путь к Check/AI восстановления здоровья CWangsheng (0x221) в Zone.
//!
//! Источник: gameserver.exe `4F5C98E0…` + GameServer.pdb (RSDS match),
//! appserver/skills/wangsheng.cpp/.h. Тела перенесены буквально в
//! `nebokrai_zone::skills::wangsheng` порцией №6b (числовое правило HP было в
//! Zone ранее). Здесь делегация с прежней сигнатурой; потребители не меняются.

use super::battlefairyskill::battle_fairy_outcome;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
};

pub(crate) use nebokrai_zone::skills::WANGSHENG_SKILL_ID;

pub(crate) fn execute_battle_fairy_wangsheng<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    battle_fairy_outcome(nebokrai_zone::skills::execute_battle_fairy_wangsheng(
        game, player_id, instance, dispatch, runtime,
        |runtime: &mut Runtime| runtime.now_milliseconds(),
    ))
}
