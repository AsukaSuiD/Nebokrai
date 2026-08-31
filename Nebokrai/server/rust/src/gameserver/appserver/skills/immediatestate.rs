//! Общий короткий runtime немедленных состояний игрока.
//!
//! Владелец объединяет только подтверждённую одинаковую последовательность
//! `Begin → Check → Calculate → Attack → Apply`. Идентификатор usage,
//! формула значения и конкретное каноническое состояние остаются у пяти
//! навыков семейства; `CGame` предоставляет player owner, свойства навыка и
//! фактическую публикацию состояния.

use super::enlargefullmiss::{ENLARGE_FULL_MISS_SKILL_ID, SKILL_USAGE_FULL_MISS_GAIN};
use super::enlargefullmissstate::EnlargeFullMissState;
use super::enlargemaxhp::{ENLARGE_MAX_HP_SKILL_ID, SKILL_USAGE_MAX_HP_GAIN};
use super::enlargemaxhpstate::EnlargeMaxHpState;
use super::enlargemaxmp::{ENLARGE_MAX_MP_SKILL_ID, SKILL_USAGE_MAX_MP_GAIN};
use super::enlargemaxmpstate::EnlargeMaxMpState;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::origin::{ORIGIN_SKILL_ID, SKILL_USAGE_ELEMENT_MODIFY_GAIN};
use super::originstate::OriginState;
use super::taiji::{SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN, TAIJI_SKILL_ID};
use super::wuxing::{execute_player_wuxing, is_wuxing_skill};
use super::taijistate::TaiJiState;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

enum ImmediateStateKind {
    TaiJi,
    EnlargeFullMiss,
    EnlargeMaxHp,
    EnlargeMaxMp,
    Origin,
}

pub(crate) const fn is_immediate_state_skill(skill_id: u32) -> bool {
    matches!(
        skill_id,
        TAIJI_SKILL_ID
            | ENLARGE_MAX_HP_SKILL_ID
            | ENLARGE_MAX_MP_SKILL_ID
            | ENLARGE_FULL_MISS_SKILL_ID
            | ORIGIN_SKILL_ID
    ) || is_wuxing_skill(skill_id)
}

pub(crate) fn execute_player_auto_start_immediate_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    runtime: &mut Runtime,
) -> bool {
    if is_wuxing_skill(skill_id) {
        return super::wuxing::execute_player_auto_start_wuxing(
            game, player_id, skill_id, runtime,
        );
    }
    let skill_level = game
        .find_player(player_id)
        .map_or(0, |player| player.learned_skill_level(skill_id));
    let Some(properties) = game.skill_base_properties(skill_id, skill_level) else {
        return false;
    };
    let (usage, state_kind) = match skill_id {
        TAIJI_SKILL_ID => (SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN, ImmediateStateKind::TaiJi),
        ENLARGE_FULL_MISS_SKILL_ID => (SKILL_USAGE_FULL_MISS_GAIN, ImmediateStateKind::EnlargeFullMiss),
        ENLARGE_MAX_HP_SKILL_ID => (SKILL_USAGE_MAX_HP_GAIN, ImmediateStateKind::EnlargeMaxHp),
        ENLARGE_MAX_MP_SKILL_ID => (SKILL_USAGE_MAX_MP_GAIN, ImmediateStateKind::EnlargeMaxMp),
        ORIGIN_SKILL_ID => (SKILL_USAGE_ELEMENT_MODIFY_GAIN, ImmediateStateKind::Origin),
        _ => return false,
    };
    let gain = properties.query_property(usage) as i32;
    if let Some(player) = game.find_player_mut(player_id) {
        match state_kind {
            ImmediateStateKind::TaiJi => { let _ = player.replace_taiji_state(TaiJiState::new(gain)); }
            ImmediateStateKind::EnlargeFullMiss => { let _ = player.replace_enlarge_full_miss_state(EnlargeFullMissState::new(gain)); }
            ImmediateStateKind::EnlargeMaxHp => { let _ = player.replace_enlarge_max_hp_state(EnlargeMaxHpState::new(gain)); }
            ImmediateStateKind::EnlargeMaxMp => { let _ = player.replace_enlarge_max_mp_state(EnlargeMaxMpState::new(gain)); }
            ImmediateStateKind::Origin => { let _ = player.replace_origin_state(OriginState::new(gain)); }
        }
    }
    let _ = game.publish_player_states(player_id);
    let _ = game.update_player_properties(player_id, runtime);
    true
}

pub(crate) fn execute_player_immediate_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let dispatch_skill_id = match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id,
    };
    if is_wuxing_skill(dispatch_skill_id) {
        return execute_player_wuxing(game, player_id, dispatch, player_ai, runtime);
    }
    let terminal = |state| QueuedSkillExecutionOutcome {
        state,
        first_contact: false,
        killing_blow: None,
    };
    let skill_id = match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. }
            if is_immediate_state_skill(skill_id) => skill_id,
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    if game.find_player(player_id).is_none() {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    if player_ai.immediate_state().is_none() {
        let started_at_ms = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(Some(skill_id));
        }
        player_ai.begin_immediate_state(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if player_ai
        .immediate_state()
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
    let (usage, state_kind) = match skill_id {
        TAIJI_SKILL_ID => (
            SKILL_USAGE_TARGET_ELEMENT_RESISTANT_GAIN,
            ImmediateStateKind::TaiJi,
        ),
        ENLARGE_FULL_MISS_SKILL_ID => (
            SKILL_USAGE_FULL_MISS_GAIN,
            ImmediateStateKind::EnlargeFullMiss,
        ),
        ENLARGE_MAX_HP_SKILL_ID => (SKILL_USAGE_MAX_HP_GAIN, ImmediateStateKind::EnlargeMaxHp),
        ENLARGE_MAX_MP_SKILL_ID => (SKILL_USAGE_MAX_MP_GAIN, ImmediateStateKind::EnlargeMaxMp),
        ORIGIN_SKILL_ID => (SKILL_USAGE_ELEMENT_MODIFY_GAIN, ImmediateStateKind::Origin),
        _ => unreachable!(),
    };
    let gain = properties.query_property(usage) as i32;
    let _state_started_at_ms = runtime.now_milliseconds();
    if let Some(player) = game.find_player_mut(player_id) {
        match state_kind {
            ImmediateStateKind::TaiJi => {
                let _ = player.replace_taiji_state(TaiJiState::new(gain));
            }
            ImmediateStateKind::EnlargeFullMiss => {
                let _ = player.replace_enlarge_full_miss_state(EnlargeFullMissState::new(gain));
            }
            ImmediateStateKind::EnlargeMaxHp => {
                let _ = player.replace_enlarge_max_hp_state(EnlargeMaxHpState::new(gain));
            }
            ImmediateStateKind::EnlargeMaxMp => {
                let _ = player.replace_enlarge_max_mp_state(EnlargeMaxMpState::new(gain));
            }
            ImmediateStateKind::Origin => {
                let _ = player.replace_origin_state(OriginState::new(gain));
            }
        }
    }
    let _ = game.publish_player_states(player_id);
    if let Some(state) = player_ai.immediate_state_mut() {
        let _ = state.advance(SkillStage::Begin, SkillStage::Check);
        let _ = state.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = state.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = state.advance(SkillStage::Attack, SkillStage::Apply);
    }
    if let Some(player) = game.find_player_mut(player_id) {
        player.set_skill_moveable(true);
        player.set_current_skill_id(None);
    }
    let _last_used_at_ms = runtime.now_milliseconds();
    terminal(QueuedSkillExecutionState::Completed)
}
