//! Тонкий путь к Check/AI и Summon BFBaseAttack (0x224) в Zone.
//!
//! Источник: gameserver.exe `4F5C98E0…` + GameServer.pdb (RSDS match),
//! appserver/skills/battlefairybasemagic.cpp. Тела перенесены буквально в
//! `nebokrai_zone::skills::battlefairybasemagic` порцией №6b. Делегация
//! сохраняет прежнюю сигнатуру: регистрация снаряда выполняется прежним
//! фасадом `CGame` через callback `BattleFairyBaseMagicSummon`; потребители
//! не меняются.

use super::battlefairyskill::battle_fairy_outcome;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
};

pub(crate) use nebokrai_zone::skills::BATTLE_FAIRY_BASE_MAGIC_SKILL_ID;

pub(crate) fn execute_battle_fairy_base_magic<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    battle_fairy_outcome(nebokrai_zone::skills::execute_battle_fairy_base_magic(
        game, player_id, instance, dispatch, begin_target, runtime,
        |runtime: &mut Runtime| runtime.now_milliseconds(),
        |game, runtime, request| {
            let _ = game.add_battle_fairy_base_magic_phalanx(
                request.region_id, request.phalanx, request.started_at_ms, runtime,
            );
        },
    ))
}
