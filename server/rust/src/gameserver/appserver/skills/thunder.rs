//! Тонкий путь к CThunder (0x21F) и общему пути семьи громовых облаков в Zone.
//!
//! Источник: gameserver.exe `4F5C98E0…` + GameServer.pdb (RSDS match),
//! appserver/skills/thunder.cpp. Тела Check/AI/Summon перенесены буквально в
//! `nebokrai_zone::skills::thunder` порцией T1 (истинные RVA-якоря, статусы
//! MATCH/UNKNOWN и fix №2 круговой доставки `0xBF918` — в шапке Zone-файла).
//! Здесь — реэкспорт оставшихся прежних имён (ID навыка и x87-адаптер для
//! bloodloss*; `thunder_base_damage` и константа damage-factor ушли в Zone
//! вместе с фалангой порцией T2), фасадная реализация
//! hub-трейта `SummonCloudGame` над прежними методами `CGame` и делегация с
//! прежней сигнатурой; конструктор `CThunderPhalanx`, его Initialize(RNG) до
//! допуска региона и входное сообщение `0xBF502` остаются прежними фасадами
//! через callback `ThunderSummon`. Потребители (очередь Game, thunderphalanx,
//! thunder2phalanx, tianhuophalanx, bloodloss*) не меняются.

use super::battlefairyskill::battle_fairy_outcome;
use super::thunderphalanx::CThunderPhalanx;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome,
};
use crate::nets::netserver::message::CMessage;
use nebokrai_zone::skills::SummonCloudGame;

pub(crate) use nebokrai_zone::skills::THUNDER_SKILL_ID;
pub(super) use nebokrai_zone::combat::truncate_original_i64_low;

impl SummonCloudGame for CGame {
    /// Второй аргумент исходного `SendToAround` (exclude-player) — UNKNOWN;
    /// прежняя реконструкция до унификации никого не исключала.
    fn send_summon_cloud_goods_update_around(&mut self, player_id: i32, message: &CMessage) {
        let _ = self.send_player_shape_around(player_id, None, message);
    }

    fn set_summon_cloud_user_direction(&mut self, player_id: i32, direction: i32) {
        if let Some(player) = self.find_player_mut(player_id) {
            player.movement_shape_mut().set_direction(direction);
        }
    }
}

pub(crate) fn execute_battle_fairy_thunder<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, begin_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    battle_fairy_outcome(nebokrai_zone::skills::execute_battle_fairy_thunder(
        game, player_id, instance, dispatch, begin_target, runtime,
        |runtime: &mut Runtime| runtime.now_milliseconds(),
        |game, runtime, request: nebokrai_zone::skills::ThunderSummon| {
            let mut phalanx = CThunderPhalanx::new(
                request.id, request.master, request.started_at_ms, request.lifetime_ms,
                request.skill_level, request.frequency_ms, request.minimum_attack,
                request.maximum_attack, request.element_modifier, request.target_count, request.cch,
            );
            phalanx.set_center(request.center_x, request.center_y);
            phalanx.initialize(&mut |maximum| game.skill_random_below(maximum));
            let Some(region) = nebokrai_zone::skills::summon_user_region(game, request.source) else { return; };
            if game.add_thunder_phalanx(
                region, phalanx, request.center_x, request.center_y, request.started_at_ms, runtime,
            ).is_some_and(|result| result.is_ok())
            {
                let _ = game.send_thunder_phalanx_entry(region, request.id, runtime);
            }
        },
    ))
}
