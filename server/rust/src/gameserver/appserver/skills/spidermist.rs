//! Тонкий путь к паучьему туману `CSpiderMist` (`0x198`) в Zone.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/spidermist.cpp.
//! Тела Check/AI/Summon, правила области и wire-кадр входного снимка
//! перенесены буквально в `nebokrai_zone::skills::spidermist` (основание и
//! статусы MATCH см. там): прежний поворот в Begin-стадии игрока и Begin
//! каста монстра машинно не существует — он перенесён в выпуск AI `0x5409B0`
//! (SetDir к клетке, SetMoveable(1) перед выпуском) или удалён для семьи.
//! Здесь — делегации с прежними сигнатурами и входной снимок области
//! поверх zone-конверта; внешние потребители не меняются.

use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::gameserver::game::ServerRegionOwner;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    game_tick_milliseconds,
};
use nebokrai_zone::skills::{SummonSkillOutcome, spidermist as zone};

pub(crate) use nebokrai_zone::skills::SPIDER_MIST_SKILL_ID;
pub(crate) use nebokrai_zone::skills::execution::SpiderMistProgress;

/// Входной снимок области `0x000BF502`: payload wire-конверта `summonshape`
/// (`include_child = true`), around-доставка от формы фаланги.
pub(crate) fn send_phalanx_entry(game: &CGame, region: &CServerRegion, phalanx_id: i32) {
    let Some(crate::gameserver::appserver::summonshape::SummonedSkillShape::SpiderMist(phalanx)) =
        region.find_skill_phalanx(phalanx_id)
    else {
        return;
    };
    let Some(payload) = phalanx.encode_client_snapshot() else { return };
    let message = zone::spider_mist_entry_message(phalanx.shape().identity(), &payload);
    let _ = game.send_game_shape_around(region, phalanx.shape(), None, &message);
}

pub(crate) const fn is_player_spider_mist_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    zone::is_player_spider_mist_dispatch(dispatch)
}

pub(crate) fn cancel_player_spider_mist<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    _runtime: &mut Runtime,
) -> bool {
    zone::cancel_player_spider_mist(game, player_id, player_ai)
}

pub(crate) fn execute_player_spider_mist<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    _player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let state = match zone::execute_player_spider_mist(
        game, player_id, dispatch, runtime, game_tick_milliseconds,
    ) {
        SummonSkillOutcome::Begun => QueuedSkillExecutionState::Begun,
        SummonSkillOutcome::Pending => QueuedSkillExecutionState::Pending,
        SummonSkillOutcome::Completed => QueuedSkillExecutionState::Completed,
        SummonSkillOutcome::Rejected => QueuedSkillExecutionState::Rejected,
    };
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт исходного навыка")]
pub(crate) fn execute_owned_spider_mist<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_owner: &mut ServerRegionOwner,
    monster_id: i32,
    target: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    zone::execute_owned_spider_mist(
        game, region_owner, monster_id, target, skill_level, properties, now_ms, runtime,
        game_tick_milliseconds,
    )
}
