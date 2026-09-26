//! Тонкий путь к круговой атаке `CMonsterRangeAttack` (ID `0x2ef`) в Zone.
//! Источник: gameserver.exe/GameServer.pdb, исходный owner
//! `appserver/skills/monsterrangeattack.cpp`. Тела Check/AI/Calc, маска 7x7
//! (x+7y, центр −3, IsAttackAble до dedup, запись цели после Attack),
//! player-путь (нулевой MP-cost — молчаливый отказ, reuse 13+GS1143,
//! failure 7+GS1144) перенесены буквально в
//! `nebokrai_zone::skills::monsterrangeattack` (статусы VERIFIED/MATCH см.
//! там, тела `.local/recon-a2/out/`). Здесь — делегации с прежними
//! сигнатурами и re-export; внешние потребители (диспетчер `game.rs`,
//! hub `monsterbaseattack.rs`) не меняются. Часы продолжения — общий
//! `game_tick_milliseconds` делегата старого main loop (blanket
//! `GameClockContext::now_milliseconds`, как у соседей).

use crate::gameserver::appserver::ai::monsterai::MonsterSkillCallOutcome;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner, game_tick_milliseconds,
};
use crate::nets::netserver::message::CMessage;
use nebokrai_zone::skills::{MonsterCombatOutcome, monsterrangeattack as zone};

pub(crate) use nebokrai_zone::skills::{
    MONSTER_RANGE_ATTACK_SKILL_ID, MonsterRangeAttackDispatch,
};

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

pub(crate) fn execute_player_monster_range_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    queued_outcome(zone::execute_player_monster_range_attack(
        game, player_id, dispatch, player_ai, runtime, game_tick_milliseconds,
    ))
}

pub(crate) fn finish_player_monster_range_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    ai: &mut CPlayerAI,
    runtime: &mut Runtime,
    success: bool,
) -> bool {
    zone::finish_player_monster_range_attack(game, player_id, ai, runtime, success)
}

/// Кадр выпуска монстровой круговой атаки; прежняя сигнатура без типа
/// стороны (сторона — монстр) сохранена для hub-потребителей.
pub(crate) fn range_attack_fire_message(
    skill_level: u16,
    monster_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> CMessage {
    const MONSTER_TYPE: i32 = 600;
    zone::range_attack_fire_message(skill_level, MONSTER_TYPE, monster_id, tile_x, tile_y)
}

pub(crate) fn range_attack_scope_cells() -> impl Iterator<Item = (i32, i32)> {
    zone::range_attack_scope_cells()
}

pub(crate) fn range_attack_cell_candidates(
    game: &CGame,
    region_owner: &ServerRegionOwner,
    monster_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> Vec<ShapeIdentity> {
    zone::range_attack_cell_candidates(game, region_owner, monster_id, tile_x, tile_y)
}

pub(crate) fn calculate_monster_range_attack(
    properties: &CSkillBaseProperties,
    skill_level: u16,
    monster_id: i32,
    random_below: &mut dyn FnMut(i32) -> i32,
) -> AttackInformation {
    zone::calculate_monster_range_attack(properties, skill_level, monster_id, random_below)
}

pub(crate) fn begin_owned_monster_range_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    started_at_ms: u32,
    _runtime: &mut Runtime,
) -> MonsterSkillCallOutcome {
    zone::begin_owned_monster_range_cast(
        game, region, monster_id, skill_level, properties, started_at_ms, game_tick_milliseconds,
    )
}

pub(crate) fn prepare_owned_monster_range_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    properties: &CSkillBaseProperties,
    dispatch: &mut Option<MonsterRangeAttackDispatch>,
    _runtime: &mut Runtime,
) -> bool {
    zone::prepare_owned_monster_range_cast(
        game, region, monster_id, properties, dispatch, game_tick_milliseconds,
    )
}

pub(crate) fn execute_owned_monster_range_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    dispatch: &MonsterRangeAttackDispatch,
    identity: ShapeIdentity,
    attacked: &[ShapeIdentity],
    runtime: &mut Runtime,
) -> bool {
    zone::execute_owned_monster_range_target(game, owner, dispatch, identity, attacked, runtime)
}
