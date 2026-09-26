//! Тонкий путь к CLeiming2 (0x21B) в Zone.
//!
//! Источник: gameserver.exe `4F5C98E0…` + GameServer.pdb (RSDS match),
//! appserver/skills/thunder2.cpp. Тело перенесено буквально в
//! `nebokrai_zone::skills::leiming2` порцией T1 (общие Check/AI с CThunder и
//! собственный Summon с AddElementAtk-слагаемым; RVA-якоря, статусы и fix №2
//! круговой доставки `0xBF918` — в шапках Zone `skills/thunder.rs` и
//! `skills/leiming2.rs`). Здесь — реэкспорт оставшейся прежней константы
//! (константа damage-factor ушла в Zone вместе с фалангой порцией T2) и
//! делегация с прежней сигнатурой; конструктор `CLeimingPhalanx2`,
//! допуск региона и
//! входное сообщение `0xBF502` остаются прежними фасадами через callback
//! `Leiming2Summon`. Потребители (очередь Game, thunder2phalanx) не меняются.

use super::battlefairyskill::battle_fairy_outcome;
use super::thunder2phalanx::CLeimingPhalanx2;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
};

pub(crate) use nebokrai_zone::skills::LEIMING2_SKILL_ID;

pub(crate) fn execute_battle_fairy_leiming2<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    battle_fairy_outcome(nebokrai_zone::skills::execute_battle_fairy_leiming2(
        game, player_id, instance, dispatch, begin_target, runtime,
        |runtime: &mut Runtime| runtime.now_milliseconds(),
        |game, runtime, request: nebokrai_zone::skills::Leiming2Summon| {
            let mut phalanx = CLeimingPhalanx2::new(
                request.id, request.master, request.started_at_ms, request.lifetime_ms,
                request.skill_level, request.minimum_attack, request.maximum_attack,
                request.element_modifier, request.cch,
            );
            phalanx.set_center(request.center_x, request.center_y);
            let Some(region) = nebokrai_zone::skills::summon_user_region(game, request.source) else { return; };
            if game.add_leiming2_phalanx(
                region, phalanx, request.center_x, request.center_y, request.started_at_ms, runtime,
            ).is_some_and(|result| result.is_ok())
            {
                let _ = game.send_leiming2_phalanx_entry(region, request.id, runtime);
            }
        },
    ))
}
