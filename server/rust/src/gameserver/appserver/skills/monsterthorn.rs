//! Тонкий путь к шипастой атаке `CMonsterThorn` (ID `0x197`) в Zone.
//! Источник: gameserver.exe/GameServer.pdb, исходный владелец
//! `appserver/skills/monsterthorn.cpp`. Тела Check/AI/Attack/Calculate и
//! player-путь перенесены буквально в `nebokrai_zone::skills::monsterthorn`
//! (статусы VERIFIED/MATCH см. там, тела `.local/recon-a2/out/`): гейты
//! reuse/paths/BLOCK_UNFLY, S==NULL→fallback {0,0,fb_x,fb_y}, длинный путь →
//! End(0), блок после задержки → updateVE(15)+End(1), shared End 0x146090,
//! обязательный второй RNG crit-roll (`vt+0x114` монстра ≡ 0, источник cch —
//! глобалка 0xEF3E5C). Здесь — делегации с прежними сигнатурами и re-export;
//! внешние потребители (диспетчер `game.rs`, hub `monsterbaseattack.rs`)
//! не меняются. Часы — общий `game_tick_milliseconds` делегата старого main
//! loop (blanket `GameClockContext::now_milliseconds`, как у соседей).

use crate::gameserver::appserver::ai::monsterai::MonsterSkillCallOutcome;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner, game_tick_milliseconds,
};
use nebokrai_zone::skills::{MonsterCombatOutcome, monsterthorn as zone};

pub(crate) use nebokrai_zone::skills::MONSTER_THORN_SKILL_ID;

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

pub(crate) const fn is_player_monster_thorn_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    zone::is_player_monster_thorn_dispatch(dispatch)
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт исходного навыка")]
pub(crate) fn execute_owned_monster_thorn<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> MonsterSkillCallOutcome {
    zone::execute_owned_monster_thorn(
        game, owner, monster_id, target_identity, skill_level, properties, now_ms, runtime,
        game_tick_milliseconds,
    )
}

pub(crate) fn execute_player_monster_thorn<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    queued_outcome(zone::execute_player_monster_thorn(
        game, player_id, dispatch, player_ai, runtime, game_tick_milliseconds,
    ))
}

pub(crate) fn cancel_player_monster_thorn<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    zone::cancel_player_monster_thorn(game, player_id, ai, runtime)
}

pub(crate) fn complete_player_monster_thorn<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> bool {
    zone::complete_player_monster_thorn(game, player_id, ai, runtime)
}
