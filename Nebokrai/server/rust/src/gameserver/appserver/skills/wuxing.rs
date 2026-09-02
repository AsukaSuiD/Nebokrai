//! Исполнение пяти постоянных состояний `CWuXing*`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `wuxing{metal,wood,water,fire,earth}.cpp`. Навык всегда предпочитает
//! собственного user-а sufferer-у, заменяет состояние того же ID в прежней
//! позиции, затем вызывает полный `UpdateProperty` и `RestoreHpMp`; базовое
//! завершение сохраняет отдельный reuse-clock конкретного элемента.

use super::baseattack::{time_reached, SKILL_USAGE_REUSE_DELAY_TIME};
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::stateskill::finish_state_skill;
use super::wuxingearth::WUXING_EARTH_SKILL_ID;
use super::wuxingfire::WUXING_FIRE_SKILL_ID;
use super::wuxingmetal::WUXING_METAL_SKILL_ID;
use super::wuxingstate::{kind_for_skill_id, WuXingKind, WuXingState, WuXingStateParameters};
use super::wuxingwater::WUXING_WATER_SKILL_ID;
use super::wuxingwood::WUXING_WOOD_SKILL_ID;
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::skills::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::states::summonskill::abort_skill;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

const TARGET_STR_GAIN: u32 = 101;
const TARGET_DEX_GAIN: u32 = 102;
const TARGET_CON_GAIN: u32 = 103;
const TARGET_INT_GAIN: u32 = 104;
const TARGET_DEF_GAIN: u32 = 109;
const TARGET_ELEMENT_RESISTANCE_GAIN: u32 = 112;
const TARGET_ELEMENT_MODIFY_GAIN: u32 = 115;
const TARGET_MIN_ATTACK_GAIN: u32 = 116;
const TARGET_MAX_ATTACK_GAIN: u32 = 117;
const MAX_HP_GAIN: u32 = 118;
const MAX_MP_GAIN: u32 = 119;
const BLAST_ATTACK_SCALE_FIX: u32 = 80_011;
const BLAST_DEFENSE_SCALE_FIX: u32 = 80_012;
const ELEMENT_BLAST_ATTACK_SCALE_FIX: u32 = 80_013;
const ELEMENT_BLAST_DEFENSE_SCALE_FIX: u32 = 80_014;
const FULL_MISS_SCALE_FIX: u32 = 80_015;
const CRITICAL_RATE_FIX: u32 = 80_016;
const RESUME_HP_PEACE_FIX: u32 = 80_017;
const RESUME_MP_PEACE_FIX: u32 = 80_018;
const RESUME_HP_FIGHT_FIX: u32 = 80_019;
const RESUME_MP_FIGHT_FIX: u32 = 80_020;
const RESTORED_HP_PEACE_FIX: u32 = 80_021;
const RESTORED_MP_PEACE_FIX: u32 = 80_022;
const RESTORED_HP_FIGHT_FIX: u32 = 80_023;
const RESTORED_MP_FIGHT_FIX: u32 = 80_024;

pub(crate) const fn is_wuxing_skill(skill_id: u32) -> bool {
    matches!(
        skill_id,
        WUXING_METAL_SKILL_ID
            | WUXING_WOOD_SKILL_ID
            | WUXING_WATER_SKILL_ID
            | WUXING_FIRE_SKILL_ID
            | WUXING_EARTH_SKILL_ID
    )
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false, killing_blow: None }
}

fn state_from_properties(
    skill_id: u32,
    properties: &CSkillBaseProperties,
) -> Option<WuXingState> {
    let kind = kind_for_skill_id(skill_id)?;
    let query = |usage| properties.query_property(usage);
    let maximum_hp = if kind == WuXingKind::Water {
        query(MAX_HP_GAIN) as i32
    } else {
        (query(MAX_HP_GAIN) as i16) as i32
    };
    Some(WuXingState::new(skill_id, kind, WuXingStateParameters {
        element_modify: query(TARGET_ELEMENT_MODIFY_GAIN) as i16,
        minimum_attack: query(TARGET_MIN_ATTACK_GAIN) as i16,
        maximum_attack: query(TARGET_MAX_ATTACK_GAIN) as i16,
        defense: query(TARGET_DEF_GAIN) as i16,
        element_resistance: query(TARGET_ELEMENT_RESISTANCE_GAIN) as i16,
        strength: query(TARGET_STR_GAIN) as i32,
        dexterity: query(TARGET_DEX_GAIN) as i32,
        constitution: query(TARGET_CON_GAIN) as i32,
        intelligence: query(TARGET_INT_GAIN) as i32,
        maximum_hp,
        maximum_mp: (kind == WuXingKind::Metal).then(|| query(MAX_MP_GAIN)).unwrap_or(0),
        blast_attack_scale_bits: (query(BLAST_ATTACK_SCALE_FIX) as i32 as f32).to_bits(),
        blast_defense_scale_bits: (query(BLAST_DEFENSE_SCALE_FIX) as i32 as f32).to_bits(),
        critical_rate_bits: (query(CRITICAL_RATE_FIX) as i32 as f32).to_bits(),
        element_blast_attack_scale_bits: (query(ELEMENT_BLAST_ATTACK_SCALE_FIX) as i32 as f32).to_bits(),
        element_blast_defense_scale_bits: (query(ELEMENT_BLAST_DEFENSE_SCALE_FIX) as i32 as f32).to_bits(),
        full_miss_scale_bits: (query(FULL_MISS_SCALE_FIX) as i32 as f32).to_bits(),
        resume_hp_peace: query(RESUME_HP_PEACE_FIX) as i32,
        resume_mp_peace: query(RESUME_MP_PEACE_FIX) as i32,
        resume_hp_fight: query(RESUME_HP_FIGHT_FIX) as i32,
        resume_mp_fight: query(RESUME_MP_FIGHT_FIX) as i32,
        restored_hp_peace: query(RESTORED_HP_PEACE_FIX) as i32,
        restored_mp_peace: query(RESTORED_MP_PEACE_FIX) as i32,
        restored_hp_fight: query(RESTORED_HP_FIGHT_FIX) as i32,
        restored_mp_fight: query(RESTORED_MP_FIGHT_FIX) as i32,
    }))
}

pub(crate) fn execute_player_auto_start_wuxing<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    skill_id: u32,
    runtime: &mut Runtime,
) -> bool {
    let skill_level = game.find_player(player_id).map_or(0, |player| player.learned_skill_level(skill_id));
    let state = game
        .skill_base_properties(skill_id, skill_level)
        .and_then(|properties| state_from_properties(skill_id, properties));
    let Some(state) = state else {
        return false;
    };
    if let Some(player) = game.find_player_mut(player_id) {
        let _ = player.replace_wuxing_state(state);
    }
    if game.update_player_properties(player_id).is_some() {
        let _ = game.restore_player_hp_mp_states(player_id);
    }
    let used_at_ms = runtime.now_milliseconds();
    if let Some(player) = game.find_player_mut(player_id) {
        player
            .player_ai_mut()
            .mark_immediate_state_used(skill_id, used_at_ms);
    }
    true
}

pub(crate) fn execute_player_wuxing<Runtime: GameMainLoopRuntime>(
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
            if is_wuxing_skill(skill_id) => skill_id,
        _ => return terminal(QueuedSkillExecutionState::Rejected),
    };
    let Some(_kind) = kind_for_skill_id(skill_id) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.find_player(player_id).is_none() {
        return terminal(QueuedSkillExecutionState::Rejected);
    }

    let skill_level = game
        .find_player(player_id)
        .map_or(0, |player| player.learned_skill_level(skill_id));
    let Some(properties) = game.skill_base_properties(skill_id, skill_level).cloned() else {
        if player_ai.immediate_state().is_some() {
            abort_skill(game, player_id);
        }
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);

    if player_ai.immediate_state().is_none() {
        let cooldown_now_ms = runtime.now_milliseconds();
        let last_used_ms = player_ai.immediate_state_last_used_ms(skill_id);
        if last_used_ms != 0
            && !time_reached(cooldown_now_ms, last_used_ms, reuse_delay_ms)
        {
            game.send_base_magic_failure(player_id, 0x0d);
            game.send_skill_system_info(player_id, b"GS0278");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let started_at_ms = runtime.now_milliseconds();
        game.enter_player_combat_state(player_id);
        if let Some(player) = game.find_player_mut(player_id) {
            player.set_current_skill_id(Some(skill_id));
        }
        player_ai.begin_immediate_state(SkillExecutionKernel::begin(dispatch, started_at_ms));
    } else if player_ai.immediate_state().is_none_or(|state| state.dispatch() != dispatch) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let state = state_from_properties(skill_id, &properties)
        .expect("WuXing ID проверен до чтения свойств");

    if let Some(player) = game.find_player_mut(player_id) {
        let _ = player.replace_wuxing_state(state);
    }
    if game.update_player_properties(player_id).is_some() {
        let _ = game.restore_player_hp_mp_states(player_id);
    }
    if let Some(execution) = player_ai.immediate_state_mut() {
        let _ = execution.advance(SkillStage::Begin, SkillStage::Check);
        let _ = execution.advance(SkillStage::Check, SkillStage::Calculate);
        let _ = execution.advance(SkillStage::Calculate, SkillStage::Attack);
        let _ = execution.advance(SkillStage::Attack, SkillStage::Apply);
    }
    finish_state_skill(game, player_id, player_ai, runtime, |player_ai, now_ms| {
        player_ai.mark_immediate_state_used(skill_id, now_ms);
    });
    terminal(QueuedSkillExecutionState::Completed)
}
