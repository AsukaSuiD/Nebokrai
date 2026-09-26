//! Щит жизни CLifeShield (0x220): Check/AI.
//!
//! Quirks: MP0 не требует предмета в Check, но первый AI при отсутствии
//! боевого духа остаётся в ожидании; запись MP предшествует сериализации
//! предмета и BF918 шлётся даже при её отказе без отката частичных изменений
//! (точечно); первый прежний щит проходит End и dtor свежего остатка позиции,
//! затем Begin(U,U) нового — без внешнего UpdateProperty.
//!
//! Швы: hub `battlefairyskill::BattleFairyGame`; списание MP с сериализацией —
//! шов `spend_battle_fairy_mana`; первичный Begin щита — hub самозащитных
//! состояний старого пакета (`begin_life_shield_state`); End CLifeShieldState
//! (Cure + обновление живой фигуры) — `lifeshieldstate.rs` старого пакета.
//!
//! Исходный владелец PDB: `appserver/skills/lifeshield.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#навыки-семейства-атрибутный-октет-bfbaseattack-transfer-fatalblow-lifeshield

use crate::content::CSkillBaseProperties;
use crate::content::goods::GAP_BF_MP;
use crate::effects::LifeShieldState;

pub use crate::effects::LIFE_SHIELD_SKILL_ID;

use super::battlefairy::battle_fairy_mana_text_cost;
use super::battlefairyskill::{
    BattleFairyGame, BattleFairyMoveShape, BattleFairySkillOutcome,
    execute_registered_battle_fairy_state, send_battle_fairy_goods_update,
};
use super::dispatch::BattleFairySkillDispatch;
use super::lifecycle::{SkillStage, skill_is_restored};

const SKILL_USAGE_USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
const SKILL_USAGE_STATE_HP: u32 = 10_010;
const SKILL_USAGE_TARGET_HP_DECREASE_FACTOR: u32 = 20_024;
const SKILL_USAGE_TARGET_MP_DECREASE_FACTOR: u32 = 20_025;

fn fail_mana<Game: BattleFairyGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(instance, 7);
    let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
    game.send_skill_system_info_with_unsigned(
        player_id, b"ZHGS0052", battle_fairy_mana_text_cost(cost),
    );
}

fn check_cast<Game: BattleFairyGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> bool {
    if game.find_player(player_id).is_none() { return false; }
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return false;
    };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, now(runtime)) {
        game.update_registered_skill_visual(instance, 13);
        game.send_skill_system_info(player_id, b"ZHGS0048");
        return false;
    }
    if properties.query_property(SKILL_USAGE_USER_MP_LOSE) != 0 {
        let Some(current) = game.battle_fairy_war_soul_addon(player_id, GAP_BF_MP)
        else { return false; };
        let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
        if current.wrapping_sub(cost as i32) < 0 {
            fail_mana(game, instance, player_id, &properties);
            return false;
        }
    }
    true
}

pub fn execute_battle_fairy_life_shield<Game: BattleFairyGame, Runtime>(
    game: &mut Game, player_id: i32, instance: Game::SkillAddress,
    dispatch: BattleFairySkillDispatch,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> BattleFairySkillOutcome {
    if dispatch.skill_id() != LIFE_SHIELD_SKILL_ID {
        return BattleFairySkillOutcome::Rejected;
    }
    execute_registered_battle_fairy_state(
        game, player_id, instance, dispatch, runtime, Some(2),
        || BattleFairySkillOutcome::Rejected,
        || BattleFairySkillOutcome::Begun,
        |game, instance, player_id, runtime| check_cast(game, instance, player_id, runtime, now),
        |game, instance, runtime| run_ai(game, instance, runtime, now),
    )
}

fn run_ai<Game: BattleFairyGame, Runtime>(
    game: &mut Game, instance: Game::SkillAddress,
    runtime: &mut Runtime,
    now: impl Fn(&mut Runtime) -> u32 + Copy,
) -> BattleFairySkillOutcome {
    let Some(skill) = game.registered_skill(instance) else {
        return BattleFairySkillOutcome::Rejected;
    };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return BattleFairySkillOutcome::Pending;
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return BattleFairySkillOutcome::Rejected;
    };
    let (user_region, user) = skill.lifecycle().user();
    let Some(source) = game.resolve_state_move_shape(user_region, user)
        .map(|shape| (shape.shape().get_region_id(), shape.shape().identity()))
    else { return BattleFairySkillOutcome::Rejected; };

    if game.registered_skill(instance).is_some_and(|skill| skill.execution_stage() == Some(SkillStage::Begin)) {
        if source.1.object_type != 400 {
            return BattleFairySkillOutcome::Rejected;
        }
        let Some(current) = game.battle_fairy_war_soul_addon(source.1.id, GAP_BF_MP)
        else { return BattleFairySkillOutcome::Pending; };
        let cost = properties.query_property(SKILL_USAGE_USER_MP_LOSE);
        if current.wrapping_sub(cost as i32) < 0 {
            fail_mana(game, instance, source.1.id, &properties);
            return BattleFairySkillOutcome::Rejected;
        }
        let Some((ex_id, payload)) = game.spend_battle_fairy_mana(source.1.id, cost)
        else { return BattleFairySkillOutcome::Pending; };
        send_battle_fairy_goods_update(game, source.1.id, ex_id, &payload);
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
        return BattleFairySkillOutcome::Rejected;
    };
    if now(runtime) < started.wrapping_add(delay) {
        return BattleFairySkillOutcome::Pending;
    }
    game.update_registered_skill_visual(instance, 1);
    let _ = game.battle_fairy_end_first_state(source.0, source.1, LIFE_SHIELD_SKILL_ID);
    let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else {
        return BattleFairySkillOutcome::Rejected;
    };
    let mp_factor = properties.query_property(SKILL_USAGE_TARGET_MP_DECREASE_FACTOR) as u16;
    let hp_factor = properties.query_property(SKILL_USAGE_TARGET_HP_DECREASE_FACTOR) as u16;
    let life = properties.query_property(SKILL_USAGE_STATE_HP) as i32;
    let keep = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let state = LifeShieldState::new(keep, life, hp_factor, mp_factor, level);
    let _ = game.begin_life_shield_state(
        source.0, source.1, Some(source), Some(source), state, &mut || now(runtime),
    );
    BattleFairySkillOutcome::Completed
}
