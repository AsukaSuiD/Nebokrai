//! Исполнение семейства `CSwordship`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `swordship.cpp`, `swordship2.cpp`, `swordship3.cpp` и `swordship4.cpp`.
//! Все четыре навыка создают на владельце постоянное состояние со значениями
//! `SKILL_USAGE_TARGET_MIN_ATK_GAIN` и `SKILL_USAGE_TARGET_MAX_ATK_GAIN`.
//! Состояния разных ID сосуществуют, повторный навык заменяет только прежнее
//! состояние того же ID, после чего обе достигнутые ветви пересчитывают
//! canonical свойства игрока. Расходов, задержки, времени восстановления, RNG
//! и отдельного сетевого эффекта в подтверждённом пути нет.
//! Monster auto-start использует те же состояния: `AI` по VA
//! `0x00580820/0x00558810/0x0054B900/0x0054B680` не ограничивает тип владельца
//! и завершает навык через `End(0)`. Поэтому фон не меняет reuse-clock и
//! очередь завершения активной атаки. Monster attack getters применяют
//! прибавки из canonical состояния при чтении свойств.

use super::kernel::{SkillExecutionKernel, SkillStage};
use super::stateskill::finish_state_skill;
use super::swordshipstate::SwordshipState;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) const SWORDSHIP_SKILL_ID: u32 = 0x6f;
pub(crate) const SWORDSHIP_2_SKILL_ID: u32 = 0xe0;
pub(crate) const SWORDSHIP_3_SKILL_ID: u32 = 0xe8;
pub(crate) const SWORDSHIP_4_SKILL_ID: u32 = 0xe9;

const SKILL_USAGE_TARGET_MIN_ATK_GAIN: u32 = 0x74;
const SKILL_USAGE_TARGET_MAX_ATK_GAIN: u32 = 0x75;

pub(crate) const fn is_swordship_skill(skill_id: u32) -> bool {
    matches!(
        skill_id,
        SWORDSHIP_SKILL_ID | SWORDSHIP_2_SKILL_ID | SWORDSHIP_3_SKILL_ID | SWORDSHIP_4_SKILL_ID
    )
}

pub(crate) fn execute_monster_auto_start_swordship(
    game: &CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    skill_id: u32,
    skill_level: i32,
) -> bool {
    if !is_swordship_skill(skill_id) {
        return false;
    }
    let Some(properties) = game.skill_base_properties(skill_id, skill_level) else {
        return false;
    };
    let Some(monster) = region.find_monster_by_id_mut(monster_id) else {
        return false;
    };
    let state = SwordshipState::new(
        skill_id,
        properties.query_property(SKILL_USAGE_TARGET_MIN_ATK_GAIN) as i32,
        properties.query_property(SKILL_USAGE_TARGET_MAX_ATK_GAIN) as i32,
    );
    let _ = monster.move_shape_mut().replace_swordship_state(state);
    let _ = game.publish_owned_monster_states(region, monster_id);
    true
}

pub(crate) fn execute_player_auto_start_swordship<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    _runtime: &mut Runtime,
) -> bool {
    if !is_swordship_skill(skill_id) {
        return false;
    }
    let skill_level = game
        .find_player(player_id)
        .map_or(0, |player| player.learned_skill_level(skill_id));
    let Some(properties) = game.skill_base_properties(skill_id, skill_level) else {
        return false;
    };
    let state = SwordshipState::new(
        skill_id,
        properties.query_property(SKILL_USAGE_TARGET_MIN_ATK_GAIN) as i32,
        properties.query_property(SKILL_USAGE_TARGET_MAX_ATK_GAIN) as i32,
    );
    if let Some(player) = game.find_player_mut(player_id) {
        let _ = player.replace_swordship_state(state);
    }
    let _ = game.publish_player_states(player_id);
    let _ = game.update_player_properties(player_id);
    true
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    }
}

pub(crate) fn execute_player_swordship<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let skill_id = match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. }
            if is_swordship_skill(skill_id) => skill_id,
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    if game.find_player(player_id).is_none() {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai.player_skill_execution(skill_id).is_none() {
        let started_at_ms = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(Some(skill_id));
        }
        player_ai.begin_player_skill_execution(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if player_ai
        .player_skill_execution(skill_id)
        .is_none_or(|state| state.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let skill_level = game
        .find_player(player_id)
        .map_or(0, |player| player.learned_skill_level(skill_id));
    let Some(properties) = game.skill_base_properties(skill_id, skill_level) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let minimum_attack_gain = properties.query_property(SKILL_USAGE_TARGET_MIN_ATK_GAIN) as i32;
    let maximum_attack_gain = properties.query_property(SKILL_USAGE_TARGET_MAX_ATK_GAIN) as i32;

    let state = SwordshipState::new(skill_id, minimum_attack_gain, maximum_attack_gain);
    if let Some(player) = game.find_player_mut(player_id) {
        let _ = player.replace_swordship_state(state);
    }
    let _ = game.publish_player_states(player_id);
    let _ = game.update_player_properties(player_id);
    if let Some(execution) = player_ai.player_skill_execution_mut(skill_id) {
        let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_state_skill(game, player_id, player_ai, runtime, |_, _| {});
    terminal(QueuedSkillExecutionState::Completed)
}
