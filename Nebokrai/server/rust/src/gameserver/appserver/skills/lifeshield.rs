//! Щит жизни CLifeShield (0x220), gameserver.exe/GameServer.pdb,
//! appserver/skills/lifeshield.cpp.
//!
//! Общий координатор боевой феи выполняет base Begin, создаёт visual и
//! завершает захваченный экземпляр навыка. Здесь находятся Check и AI:
//! успешный Check допускает первый AI отдельно, не сбрасывая начальные часы.
//! MP0 не требует предмета в Check, но первый AI при отсутствии боевого
//! духа остаётся в ожидании. Проверка MP использует знак DWORD-разности.
//!
//! Запись MP предшествует сериализации предмета. BF918 отправляется даже
//! при отказе сериализации, без отката частичных изменений; только затем
//! задаются прерываемость, visual0 и ожидание абсолютного срока.
//! Первый прежний щит проходит End и destructor свежего остатка позиции.
//! После этого читаются уровень и параметры нового щита: Begin(U,U) с
//! собственными часами и пакетом → append. Внешнего UpdateProperty нет.
//! Полный skill End, включая visual3 и AfterUse, остаётся у координатора.

use super::battlefairytransfer::send_goods_update;
use super::kernel::{
    BattleFairyExecution, SkillExecutionKernel, SkillStage, battle_fairy_mana_text_cost,
    skill_is_restored,
};
use super::lifeshieldstate::LifeShieldState;
use super::shieldstate::{DefenseShieldState, begin_primary_self_shield_state};
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::state_skill_outcome;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::BattleFairySkillDispatch;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

pub(crate) const LIFE_SHIELD_SKILL_ID: u32 = 544;
pub(crate) const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
pub(crate) const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
pub(crate) const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
pub(crate) const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
pub(crate) const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
pub(crate) const SKILL_USAGE_STATE_HP: u32 = 10_010;
pub(crate) const SKILL_USAGE_TARGET_HP_DECREASE_FACTOR: u32 = 20_024;
pub(crate) const SKILL_USAGE_TARGET_MP_DECREASE_FACTOR: u32 = 20_025;

fn fail_mana(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(
        player_id, b"ZHGS0052", battle_fairy_mana_text_cost(cost),
    );
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32, runtime: &mut Runtime,
) -> bool {
    if game.find_player(player_id).is_none() { return false; }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return false;
    };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        game.send_skill_system_info(player_id, b"ZHGS0048");
        return false;
    }
    if properties.query_property(SKILL_USAGE_USER_MP_LOSE) != 0 {
        let Some(current) = game.find_player(player_id)
            .and_then(|player| player.war_soul_mana(game.goods_factory()))
        else { return false; };
        let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
        if current.wrapping_sub(cost as i32) < 0 {
            fail_mana(game, instance, player_id, &properties);
            return false;
        }
    }
    true
}

pub(crate) fn execute_battle_fairy_life_shield<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: BattleFairySkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != LIFE_SHIELD_SKILL_ID {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    let Some(skill) = game.registered_skill(instance) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if let Some(previous) = skill.battle_fairy_dispatch() {
        if previous != dispatch {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        }
        return run_ai(game, instance, runtime);
    }
    if !check_cast(game, instance, player_id, runtime) {
        game.update_registered_skill_visual(instance, 2);
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    let kernel = SkillExecutionKernel::begin(dispatch, started);
    if !game.registered_skill_mut(instance).is_some_and(|skill| {
        skill.install_battle_fairy_execution(BattleFairyExecution::State(kernel))
    }) {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    state_skill_outcome(QueuedSkillExecutionState::Begun)
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    let (user_region, user) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, user_region, user)
        .map(|shape| (shape.shape().get_region_id(), shape.shape().identity()))
    else { return state_skill_outcome(QueuedSkillExecutionState::Rejected); };

    if skill.execution_stage() == Some(SkillStage::Begin) {
        if source.1.object_type != 400 {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        }
        let Some(current) = game.find_player(source.1.id)
            .and_then(|player| player.war_soul_mana(game.goods_factory()))
        else { return state_skill_outcome(QueuedSkillExecutionState::Pending); };
        let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
        if current.wrapping_sub(cost as i32) < 0 {
            fail_mana(game, instance, source.1.id, &properties);
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        }
        let factory = game.goods_factory().clone();
        let da_kong_key = game.globe_setup().da_kong_key();
        let Some((update, _encoded)) = game.find_player_mut(source.1.id)
            .and_then(|player| player.spend_war_soul_mana_record(cost, &factory, da_kong_key))
        else { return state_skill_outcome(QueuedSkillExecutionState::Pending); };
        send_goods_update(game, &update);
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_available(can_break != 0);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) {
        return state_skill_outcome(QueuedSkillExecutionState::Pending);
    }
    game.update_registered_skill_visual(instance, 1);
    if let Some((position, _)) = resolve_state_move_shape(game, source.0, source.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == LIFE_SHIELD_SKILL_ID))
    {
        let _ = end_and_destroy_state_at(game, source.0, source.1, position);
    }
    let Some(level) = game.registered_skill(instance).map(MoveShapeSkill::level) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    let mp_factor = properties.query_property(SKILL_USAGE_TARGET_MP_DECREASE_FACTOR) as u16;
    let hp_factor = properties.query_property(SKILL_USAGE_TARGET_HP_DECREASE_FACTOR) as u16;
    let life = properties.query_property(SKILL_USAGE_STATE_HP) as i32;
    let keep = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let state = LifeShieldState::new(keep, life, hp_factor, mp_factor, level);
    let _ = begin_primary_self_shield_state(
        game, source.0, source.1, Some(source), Some(source), DefenseShieldState::Life(state),
        &mut || runtime.now_milliseconds(),
    );
    state_skill_outcome(QueuedSkillExecutionState::Completed)
}
