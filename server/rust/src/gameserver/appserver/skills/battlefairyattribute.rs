//! Тонкий путь к Check/AI атрибутного октета Po/Yu в Zone.
//!
//! Источник: gameserver.exe `4F5C98E0…` + GameServer.pdb (RSDS match),
//! appserver/skills/{pojia,pobing,pomo,pofa,yujia,yubing,yumo,yufa}.cpp.
//! Тела перенесены буквально в `nebokrai_zone::skills::battlefairyattribute`
//! порцией №6b (описание 167/167-скелета, двойного списания Yumo и швов — в
//! шапке Zone-файла). Здесь реэкспорт типа и таблицы для `pojia.rs`..`yufa.rs`
//! и делегация исполнения с прежней сигнатурой; потребители не меняются.

use super::battlefairyskill::battle_fairy_outcome;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
};

pub(crate) use nebokrai_zone::skills::{
    BattleFairyAttributeSkill, battle_fairy_attribute_definition as definition,
};

pub(crate) fn execute_battle_fairy_attribute<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    battle_fairy_outcome(nebokrai_zone::skills::execute_battle_fairy_attribute(
        game, player_id, instance, dispatch, runtime,
        |runtime: &mut Runtime| runtime.now_milliseconds(),
    ))
}
