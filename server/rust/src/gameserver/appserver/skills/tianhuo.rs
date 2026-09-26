//! Тонкий путь к небесному огню CTianhuo (0x21A) в Zone.
//!
//! Источник: gameserver.exe `4F5C98E0…` + GameServer.pdb (RSDS match),
//! appserver/skills/tianhuo.cpp. Тело перенесено буквально в
//! `nebokrai_zone::skills::tianhuo` порцией T1; RVA-якоря, статусы и решение
//! точечной доставки `0xBF918` — в шапке Zone-файла. Здесь — реэкспорт
//! оставшейся прежней константы (константа damage-factor ушла в Zone вместе
//! с фалангой порцией T2) и делегация с прежней сигнатурой; конструктор
//! `CTianhuoPhalanx`, свёртка старой области и входное `0xBF502` остаются
//! прежними фасадами через callback `TianhuoSummon`. Потребители
//! (очередь Game, tianhuophalanx) не меняются.

use super::battlefairyskill::battle_fairy_outcome;
use super::tianhuophalanx::CTianhuoPhalanx;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
};

pub(crate) use nebokrai_zone::skills::TIANHUO_SKILL_ID;

pub(crate) fn execute_battle_fairy_tianhuo<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    battle_fairy_outcome(nebokrai_zone::skills::execute_battle_fairy_tianhuo(
        game, player_id, instance, dispatch, begin_target, runtime,
        |runtime: &mut Runtime| runtime.now_milliseconds(),
        |game, runtime, request: nebokrai_zone::skills::TianhuoSummon| {
            let mut phalanx = CTianhuoPhalanx::new(
                request.id, request.master, request.started_at_ms, request.lifetime_ms,
                request.skill_level, request.minimum_attack, request.maximum_attack,
                request.element_modifier,
            );
            phalanx.set_center(request.center_x, request.center_y);
            if game.add_tianhuo_phalanx(
                request.region_id, phalanx, request.center_x, request.center_y,
                request.started_at_ms, runtime,
            ).is_some_and(|result| result.is_ok())
            {
                let _ = game.send_tianhuo_phalanx_entry(request.region_id, request.id, runtime);
            }
        },
    ))
}
