//! Зарегистрированный вход взаимно исключающих закалок CCallosity/CCallosity2.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/callosity.cpp
//! и callosity2.cpp. Различаются только ID, таблица свойств и восстановление.
//!
//! Attack Begin сохраняет исходного U, ранний отсчёт и loop1 visual. Check
//! проверяет reuse и ненулевые цены MP/RP до Move0; отказ получает End(0).
//! Каждый AI сохраняет таблицу свойств и своего U через callbacks. Смерть U
//! даёт visual2/End(1), нехватка ресурсов — visual7/8 и End(0). Первый AI
//! всегда выполняет GetMP→query→SetMP, затем GetRP→query→SetRP, включая нулевые
//! цены. Signed wrapping-проверка RP не откатывает уже списанный MP.
//! CAN предшествует visual0 и condition; отдельного OnChangeStates здесь нет.
//!
//! Абсолютный unsigned срок start+delay предшествует visual1. Затем завершается
//! первый непустой ID75/7D без RTTI/ended-фильтра; только после этого frozen
//! таблица отдаёт persist и WORD factor. Новый экземпляр получает Begin(U,U)
//! до append, а UpdateProperty выполняется и при отказе Begin. Общий End
//! сбрасывает phase/active, возвращает движение свежему U и завершает State
//! с настоящим аргументом. Исполнение публикуется целиком общим владельцем.
//! Непроверенный native доступ к ресурсам CPlayer заменён безопасным отказом
//! для чужого CMoveShape; без чтения ресурсов Check сохраняет общий Move0.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
pub(crate) use nebokrai_zone::effects::CALLOSITY_2_SKILL_ID;
use super::callositystate::{CallosityFamilyState, replace_callosity_state};
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) use nebokrai_zone::effects::CALLOSITY_SKILL_ID;
pub(crate) const SKILL_USAGE_USER_RP_LOSE: u32 = 3;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const TARGET_BLAST_COEFFICIENT_GAIN: u32 = 125;
const STATE_PERSIST_TIME: u32 = 10_002;

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn resource_failure(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    properties: &CSkillBaseProperties, usage: u32,
) {
    let (mode, text) = if usage == USER_MP_LOSE { (7, &b"GS0288"[..]) } else { (8, &b"GS0289"[..]) };
    game.update_registered_skill_visual(instance, mode);
    let amount = properties.query_property(usage);
    game.send_skill_system_info_with_unsigned(player_id, text, amount);
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: (i32, ShapeIdentity), runtime: &mut Runtime,
) -> bool {
    let Some(source) = resolve_state_move_shape(game, original_user.0, original_user.1) else { return false; };
    let source = (source.shape().get_region_id(), source.shape().identity());
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        if source.1.object_type == PLAYER_TYPE { game.send_skill_system_info(source.1.id, b"GS0278"); }
        return false;
    }
    if properties.query_property(USER_MP_LOSE) != 0 {
        if source.1.object_type != PLAYER_TYPE { return false; }
        let Some(mana) = game.find_player(source.1.id).map(CPlayer::mana) else { return false; };
        let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
        if (remaining as i32) < 0 {
            resource_failure(game, instance, source.1.id, &properties, USER_MP_LOSE);
            return false;
        }
    }
    if properties.query_property(SKILL_USAGE_USER_RP_LOSE) != 0 {
        if source.1.object_type != PLAYER_TYPE { return false; }
        let Some(rp) = game.find_player(source.1.id).map(CPlayer::rp) else { return false; };
        let remaining = u32::from(rp).wrapping_sub(properties.query_property(SKILL_USAGE_USER_RP_LOSE));
        if (remaining as i32) < 0 {
            resource_failure(game, instance, source.1.id, &properties, SKILL_USAGE_USER_RP_LOSE);
            return false;
        }
    }
    let Some(source) = resolve_state_move_shape_mut(game, source.0, source.1) else { return false; };
    source.set_moveable(false);
    true
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region, identity)
        .map(|source| (source.shape().get_region_id(), source.shape().identity()))
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if game.move_shape_health(source.0, source.1) == Some(0) {
        game.update_registered_skill_visual(instance, 2);
        return terminal(QueuedSkillExecutionState::RejectedAfterUse);
    }
    if stage == SkillStage::Begin {
        if source.1.object_type != PLAYER_TYPE { return terminal(QueuedSkillExecutionState::Rejected); }
        let Some(mana) = game.find_player(source.1.id).map(CPlayer::mana) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
        if (remaining as i32) < 0 {
            resource_failure(game, instance, source.1.id, &properties, USER_MP_LOSE);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(player) = game.find_player_mut(source.1.id) else { return terminal(QueuedSkillExecutionState::Rejected); };
        player.set_mana(remaining);
        let Some(rp) = game.find_player(source.1.id).map(CPlayer::rp) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let remaining = u32::from(rp).wrapping_sub(properties.query_property(SKILL_USAGE_USER_RP_LOSE));
        if (remaining as i32) < 0 {
            resource_failure(game, instance, source.1.id, &properties, SKILL_USAGE_USER_RP_LOSE);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(player) = game.find_player_mut(source.1.id) else { return terminal(QueuedSkillExecutionState::Rejected); };
        player.set_rp(remaining as u16);
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    game.update_registered_skill_visual(instance, 1);
    let _ = replace_callosity_state(game, source, || {
        let keep = properties.query_property(STATE_PERSIST_TIME) as i32;
        let factor = properties.query_property(TARGET_BLAST_COEFFICIENT_GAIN) as u16;
        CallosityFamilyState::new(skill_id, factor, 0, keep)
    }, &mut || runtime.now_milliseconds());
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_callosity<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !matches!(dispatch.skill_id(), CALLOSITY_SKILL_ID | CALLOSITY_2_SKILL_ID) {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::SelfCast,
        |game, instance, _player_id, runtime| original_user
            .is_some_and(|source| check_cast(game, instance, source, runtime)),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_ai,
    )
}
