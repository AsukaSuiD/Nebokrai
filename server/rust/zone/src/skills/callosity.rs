//! Зарегистрированный вход взаимно исключающих закалок CCallosity/CCallosity2
//! (один файл на оба cpp: различаются ID, таблица свойств и восстановление;
//! методы состояний попарно folded — см. `skills/callositystate.rs`).
//!
//! Quirks: первый AI всегда выполняет GetMP→query→SetMP, затем GetRP→query→
//! SetRP, включая нулевые цены; signed-проверка RP не откатывает уже списанный
//! MP; первый непустой ID75/7D завершается без RTTI/ended-фильтра; новый
//! экземпляр получает Begin(U,U) до append, а UpdateProperty выполняется и
//! при отказе Begin; непроверенный native доступ к ресурсам CPlayer заменён
//! безопасным отказом для чужого CMoveShape.
//!
//! Швы: hub `selfcast::SelfCastGame`; вход и захват исходного U — у
//! `playercast` делегата; замена состояния — `skills/callositystate.rs`.
//!
//! Исходные владельцы PDB: `appserver/skills/{callosity,callosity2}.cpp`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#roarcpillarcallosityenergyholding--self-касты

use crate::content::CSkillBaseProperties;
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;

use super::callositystate::{CallosityFamilyState, replace_callosity_state};
use super::dispatch::PlayerSkillDispatch;
use super::lifecycle::{SkillStage, skill_is_restored};
use super::selfcast::{SelfCastExecutionOutcome, SelfCastGame, SelfCastMoveShape, SelfCastPlayer};

pub use crate::effects::{CALLOSITY_2_SKILL_ID, CALLOSITY_SKILL_ID};

pub const SKILL_USAGE_USER_RP_LOSE: u32 = 3;
const USER_MP_LOSE: u32 = 2;
const TARGET_BLAST_COEFFICIENT_GAIN: u32 = 125;
const STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_DELAY_TIME: u32 = 10_001;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

fn resource_failure<Game: SelfCastGame>(
    game: &mut Game, instance: Game::SkillAddress, player_id: i32,
    properties: &CSkillBaseProperties, usage: u32,
) {
    let (mode, text) = if usage == USER_MP_LOSE { (7, &b"GS0288"[..]) } else { (8, &b"GS0289"[..]) };
    game.update_registered_skill_visual(instance, mode);
    let amount = properties.query_property(usage);
    game.send_skill_system_info_with_unsigned(player_id, text, amount);
}

pub fn check_cast<Game: SelfCastGame>(
    game: &mut Game, instance: Game::SkillAddress, original_user: (i32, ShapeIdentity),
    now_milliseconds: fn() -> u32,
) -> bool {
    let Some(source) = game.resolve_state_move_shape(original_user.0, original_user.1) else { return false; };
    let source = (source.shape().get_region_id(), source.shape().identity());
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        if source.1.object_type == PLAYER_TYPE { game.send_skill_system_info(source.1.id, b"GS0278"); }
        return false;
    }
    if properties.query_property(USER_MP_LOSE) != 0 {
        if source.1.object_type != PLAYER_TYPE { return false; }
        let Some(mana) = game.find_player(source.1.id).map(|player| player.mana()) else { return false; };
        let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
        if (remaining as i32) < 0 {
            resource_failure(game, instance, source.1.id, &properties, USER_MP_LOSE);
            return false;
        }
    }
    if properties.query_property(SKILL_USAGE_USER_RP_LOSE) != 0 {
        if source.1.object_type != PLAYER_TYPE { return false; }
        let Some(rp) = game.find_player(source.1.id).map(|player| player.rp()) else { return false; };
        let remaining = u32::from(rp).wrapping_sub(properties.query_property(SKILL_USAGE_USER_RP_LOSE));
        if (remaining as i32) < 0 {
            resource_failure(game, instance, source.1.id, &properties, SKILL_USAGE_USER_RP_LOSE);
            return false;
        }
    }
    let Some(source) = game.resolve_state_move_shape_mut(source.0, source.1) else { return false; };
    source.set_moveable(false);
    true
}

pub fn run_ai<Game: SelfCastGame>(
    game: &mut Game, instance: Game::SkillAddress, now_milliseconds: fn() -> u32,
) -> SelfCastExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return SelfCastExecutionOutcome::Rejected; };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return SelfCastExecutionOutcome::Pending;
    };
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else {
        return SelfCastExecutionOutcome::Rejected;
    };
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = game.resolve_state_move_shape(region, identity)
        .map(|source| (source.shape().get_region_id(), source.shape().identity()))
    else { return SelfCastExecutionOutcome::Rejected; };
    if game.move_shape_health(source.0, source.1) == Some(0) {
        game.update_registered_skill_visual(instance, 2);
        return SelfCastExecutionOutcome::RejectedAfterUse;
    }
    if stage == SkillStage::Begin {
        if source.1.object_type != PLAYER_TYPE { return SelfCastExecutionOutcome::Rejected; }
        let Some(mana) = game.find_player(source.1.id).map(|player| player.mana()) else { return SelfCastExecutionOutcome::Rejected; };
        let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
        if (remaining as i32) < 0 {
            resource_failure(game, instance, source.1.id, &properties, USER_MP_LOSE);
            return SelfCastExecutionOutcome::Rejected;
        }
        let Some(player) = game.find_player_mut(source.1.id) else { return SelfCastExecutionOutcome::Rejected; };
        player.set_mana(remaining);
        let Some(rp) = game.find_player(source.1.id).map(|player| player.rp()) else { return SelfCastExecutionOutcome::Rejected; };
        let remaining = u32::from(rp).wrapping_sub(properties.query_property(SKILL_USAGE_USER_RP_LOSE));
        if (remaining as i32) < 0 {
            resource_failure(game, instance, source.1.id, &properties, SKILL_USAGE_USER_RP_LOSE);
            return SelfCastExecutionOutcome::Rejected;
        }
        let Some(player) = game.find_player_mut(source.1.id) else { return SelfCastExecutionOutcome::Rejected; };
        player.set_rp(remaining as u16);
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return SelfCastExecutionOutcome::Rejected; };
        skill.lifecycle_mut().set_available(can_break != 0);
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return SelfCastExecutionOutcome::Rejected;
    };
    if now_milliseconds() < started.wrapping_add(delay) { return SelfCastExecutionOutcome::Pending; }
    game.update_registered_skill_visual(instance, 1);
    let _ = replace_callosity_state(game, source, || {
        let keep = properties.query_property(STATE_PERSIST_TIME) as i32;
        let factor = properties.query_property(TARGET_BLAST_COEFFICIENT_GAIN) as u16;
        CallosityFamilyState::new(skill_id, factor, 0, keep)
    }, &mut || now_milliseconds());
    SelfCastExecutionOutcome::Completed
}

/// Дисциплина dispatch-а пары сохранена у зарегистрированного входа делегата.
pub const fn is_callosity_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    matches!(dispatch.skill_id(), CALLOSITY_SKILL_ID | CALLOSITY_2_SKILL_ID)
}
