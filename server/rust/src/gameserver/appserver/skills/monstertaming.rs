//! Тонкий путь к приручению `CMonsterTaming` (ID `0xd4`) в Zone. Источник:
//! gameserver.exe/GameServer.pdb, владелец `appserver/skills/monstertaming.cpp`.
//! Player-путь Check/AI и жизненный цикл питомца перенесены буквально в
//! `nebokrai_zone::skills::monstertaming` (статусы VERIFIED/MATCH см. там,
//! тела `.local/recon-a2/out/`); там же исправления DIFF-T1 (нулевая
//! стоимость MP → молчаливый reject машинного jbe→ret0 вместо прежнего
//! допуска) и DIFF-T2 (терминальный кадр `{0xBFE01, 0, 2}` после каждого
//! CheckCastCondition-отказа всех трёх Begin, message-owner mode 2 по
//! jump-таблице 0x57BD74 и remap16 0x57BD98). Здесь — делегации с прежними
//! сигнатурами и re-export; внешние потребители (диспетчер `game.rs`) не
//! меняются. Часы — общий `game_tick_milliseconds` делегата старого main
//! loop (blanket `GameClockContext::now_milliseconds`, как у соседей).

use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    game_tick_milliseconds,
};
use nebokrai_zone::skills::{MonsterCombatOutcome, monstertaming as zone};

pub(crate) use nebokrai_zone::skills::MONSTER_TAMING_SKILL_ID;

fn queued_outcome(outcome: MonsterCombatOutcome) -> QueuedSkillExecutionOutcome {
    let state = match outcome {
        MonsterCombatOutcome::Begun => QueuedSkillExecutionState::Begun,
        MonsterCombatOutcome::Pending => QueuedSkillExecutionState::Pending,
        MonsterCombatOutcome::Completed => QueuedSkillExecutionState::Completed,
        MonsterCombatOutcome::Rejected => QueuedSkillExecutionState::Rejected,
        MonsterCombatOutcome::RejectedAfterUse => QueuedSkillExecutionState::RejectedAfterUse,
    };
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

pub(crate) fn execute_player_monster_taming<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    _player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    queued_outcome(zone::execute_player_monster_taming(
        game, player_id, dispatch, runtime, game_tick_milliseconds,
    ))
}

pub(crate) fn complete_player_monster_taming<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    zone::complete_player_monster_taming(game, player_id, player_ai, runtime)
}

pub(crate) fn cancel_player_monster_taming<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    player_ai: &mut CPlayerAI,
    _runtime: &mut Runtime,
) -> bool {
    zone::cancel_player_monster_taming(game, player_id, player_ai)
}
