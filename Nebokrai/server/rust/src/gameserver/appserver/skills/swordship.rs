//! Исполнение семейства `CSwordship`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `swordship.cpp`, `swordship2.cpp`, `swordship3.cpp` и `swordship4.cpp`.
//! Все четыре навыка создают на владельце постоянное состояние со значениями
//! `SKILL_USAGE_TARGET_MIN_ATK_GAIN` и `SKILL_USAGE_TARGET_MAX_ATK_GAIN`.
//! Состояния разных ID сосуществуют, повторный навык заменяет только прежнее
//! состояние того же ID. Расходов, задержки, времени восстановления, RNG и
//! отдельного сетевого эффекта в подтверждённом пути нет.

use super::kernel::{SkillExecutionKernel, SkillStage};
use super::swordshipstate::SwordshipState;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
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

pub(crate) fn execute_player_auto_start_swordship<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    runtime: &mut Runtime,
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
    let _ = game.update_player_properties(player_id, runtime);
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

    if player_ai.swordship().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(Some(skill_id));
        }
        player_ai.begin_swordship(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if player_ai
        .swordship()
        .is_none_or(|state| state.dispatch() != dispatch)
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let skill_level = game
        .find_player(player_id)
        .map_or(0, |player| player.learned_skill_level(skill_id));
    let Some(properties) = game.skill_base_properties(skill_id, skill_level) else {
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_skill_moveable(true);
            player.set_current_skill_id(None);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let minimum_attack_gain = properties.query_property(SKILL_USAGE_TARGET_MIN_ATK_GAIN) as i32;
    let maximum_attack_gain = properties.query_property(SKILL_USAGE_TARGET_MAX_ATK_GAIN) as i32;

    let state = SwordshipState::new(skill_id, minimum_attack_gain, maximum_attack_gain);
    if let Some(player) = game.find_player_mut(player_id) {
        let _ = player.replace_swordship_state(state);
    }
    let _ = game.publish_player_states(player_id);
    if let Some(execution) = player_ai.swordship_mut() {
        let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.set_current_skill_id(None);
    }
    terminal(QueuedSkillExecutionState::Completed)
}
