//! Тонкий путь к Check/AI и Summon CFatalBlow (0x21C) в Zone.
//!
//! Источник: gameserver.exe `4F5C98E0…` + GameServer.pdb (RSDS match),
//! appserver/skills/fatalblow.cpp (ctor `0x11E130`, Summon(shape,shape)
//! `0x51F640`). Тела перенесены буквально в `nebokrai_zone::skills::fatalblow`
//! порцией №6b (машинная цепочка summon-хелперов и якоря — в шапке
//! Zone-файла). Делегация сохраняет прежнюю сигнатуру: регистрация снаряда и
//! входное сообщение `0xBF502` выполняются прежними фасадами `CGame` через
//! callback `FatalBlowSummon`; потребители не меняются.

use super::battlefairyskill::battle_fairy_outcome;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
};

pub(crate) use nebokrai_zone::skills::FATAL_BLOW_SKILL_ID;

pub(crate) fn execute_battle_fairy_fatal_blow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    battle_fairy_outcome(nebokrai_zone::skills::execute_battle_fairy_fatal_blow(
        game, player_id, instance, dispatch, begin_target, runtime,
        |runtime: &mut Runtime| runtime.now_milliseconds(),
        |game, runtime, request| {
            let id = request.phalanx.shape().identity().id;
            if game.add_fatal_blow_phalanx(request.region_id, request.phalanx, request.started_at_ms, runtime)
                .is_some_and(|result| result.is_ok())
            {
                let _ = game.send_fatal_blow_phalanx_entry(request.region_id, id, runtime);
            }
        },
    ))
}
