//! Правила и исполнение CPromotion (0x142) — усиление.
//!
//! Quirks: U без region-link завершает навык до проверки смерти S; после
//! delay повторного пути нет — visual(1) разрешает S заново с fallback на U,
//! наложение использует прежнюю S; первый прежний CPromotionState получает
//! только Restart; End возвращает движение фактическому U.
//!
//! Швы: hub `statecast::*` — у владельца старого пакета; чтение таблицы
//! свойств closure-ом по прецеденту `GodBlessGains`/`hearten_state`.
//!
//! Исходные владельцы PDB: `appserver/skills/promotion.cpp/.h`,
//! `promotionstate.cpp/.h`.
//! Доказательства: docs/reconstruction/gameserver-skills.md#promotion--cpromotion-0x142

use nebokrai_shared::runtime::get_line_direction;

use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;

use super::lifecycle::{SkillStage, skill_is_restored};
use super::promotionstate::begin_or_restart_promotion_state;
use super::statecast::{
    StateCastExecutionOutcome, StateCastGame, StateCastMoveShape, StateCastPlayer,
    StateCastVisualContract, StateCastVisualTarget, publish_state_cast_visual, state_cast_participant,
};
use super::visualeffect::SkillVisualEffectKind;

pub const PROMOTION_SKILL_ID: u32 = 0x142;
const MP_LOSS: u32 = 2;
const EM_MODIFIER: u32 = 20_015;
const HEAL_RECOVER_COEFFICIENT: u32 = 20_019;
const DELAY: u32 = 10_001;
const PERSIST: u32 = 10_002;
const MAX_DISTANCE: u32 = 5_003;
const REUSE: u32 = 10_005;
const CAN_BREAK: u32 = 10_006;

static PROMOTION_VISUAL: StateCastVisualContract = StateCastVisualContract {
    skill_id: PROMOTION_SKILL_ID,
    kind: SkillVisualEffectKind::Promotion,
    failures: &[2, 7, 10, 11, 13, 15],
    target: StateCastVisualTarget::SuffererOrUser,
};

pub fn publish_promotion_visual<Game: StateCastGame>(
    game: &Game,
    skill: &super::execution::RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
) {
    publish_state_cast_visual(game, skill, mode, &PROMOTION_VISUAL);
}

fn failure<Game: StateCastGame>(
    game: &mut Game, address: Game::SkillAddress, source: (i32, ShapeIdentity),
    mode: u32, text: &[u8], amount: Option<u32>,
) {
    game.update_registered_skill_visual(address, mode);
    if source.1.object_type == PLAYER_TYPE {
        if let Some(amount) = amount {
            game.send_skill_system_info_with_unsigned(source.1.id, text, amount);
        } else {
            game.send_skill_system_info(source.1.id, text);
        }
    }
}

pub fn check_cast<Game: StateCastGame>(
    game: &mut Game,
    address: Game::SkillAddress,
    begin_target: Option<(i32, ShapeIdentity)>,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(skill) = game.registered_skill(address) else { return false; };
    let Some(source) = state_cast_participant(game, skill.lifecycle().user()) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let Some(target) = begin_target.and_then(|target| state_cast_participant(game, target)) else {
        failure(game, address, source, 10, b"GS0286", None);
        return false;
    };
    let reuse = properties.query_property(REUSE);
    if !skill_is_restored(skill.last_used_ms(), reuse, now()) {
        failure(game, address, source, 13, b"GS0278", None);
        return false;
    }
    let path = game.skill_target_path(skill.lifecycle());
    if source != target && properties.query_property(MAX_DISTANCE) != 0
        && properties.query_property(MAX_DISTANCE) < path.len() as u32
    {
        failure(game, address, source, 11, b"GS0290", None);
        return false;
    }
    if source.1.object_type == PLAYER_TYPE {
        let cost = properties.query_property(MP_LOSS);
        if cost == 0 { return false; }
        let Some(mana) = game.find_player(source.1.id).map(|player| player.mana()) else { return false; };
        if (mana.wrapping_sub(properties.query_property(MP_LOSS)) as i32) < 0 {
            failure(game, address, source, 7, b"GS0288", Some(properties.query_property(MP_LOSS)));
            return false;
        }
        if let Some(user) = game.resolve_state_move_shape_mut(source.0, source.1) {
            user.set_moveable(false);
        }
    }
    true
}

pub fn run_promotion_ai<Game: StateCastGame>(
    game: &mut Game,
    address: Game::SkillAddress,
    now: &mut dyn FnMut() -> u32,
) -> StateCastExecutionOutcome {
    let Some(skill) = game.registered_skill(address) else {
        return StateCastExecutionOutcome::Rejected;
    };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return StateCastExecutionOutcome::Pending;
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return StateCastExecutionOutcome::EndRejected;
    };
    let source = state_cast_participant(game, skill.lifecycle().user());
    let target = game.resolve_skill_sufferer(skill.lifecycle())
        .and_then(|target| state_cast_participant(game, target));
    let (Some(source), Some(target)) = (source, target) else {
        return StateCastExecutionOutcome::EndRejected;
    };
    if game.resolve_state_move_shape(source.0, source.1)
        .is_none_or(|shape| !shape.shape().is_assigned_to_server_region())
    {
        return StateCastExecutionOutcome::EndRejected;
    }
    if game.move_shape_health(target.0, target.1) == Some(0) {
        failure(game, address, source, 10, b"GS0285", None);
        return StateCastExecutionOutcome::EndRejected;
    }
    if game.registered_skill(address).and_then(|skill| skill.execution_stage()) == Some(SkillStage::Begin) {
        if source.1.object_type == PLAYER_TYPE {
            let Some(mana) = game.find_player(source.1.id).map(|player| player.mana()) else {
                return StateCastExecutionOutcome::EndRejected;
            };
            let remaining = mana.wrapping_sub(properties.query_property(MP_LOSS));
            if (remaining as i32) < 0 {
                failure(game, address, source, 7, b"GS0288", Some(properties.query_property(MP_LOSS)));
                return StateCastExecutionOutcome::EndRejected;
            }
            if let Some(player) = game.find_player_mut(source.1.id) {
                player.set_mana(remaining);
            }
            game.publish_player_states(source.1.id);
        }
        if let Some(skill) = game.registered_skill_mut(address) {
            skill.lifecycle_mut().set_available(properties.query_property(CAN_BREAK) != 0);
        }
        let direction = (|| {
            let user = game.resolve_state_move_shape(source.0, source.1)?.shape();
            let target = game.resolve_state_move_shape(target.0, target.1)?.shape();
            Some(get_line_direction(
                user.get_tile_x().unwrap_or(i32::MIN), user.get_tile_y().unwrap_or(i32::MIN),
                target.get_tile_x().unwrap_or(i32::MIN), target.get_tile_y().unwrap_or(i32::MIN),
            ))
        })();
        if let Some(direction) = direction
            && let Some(user) = game.resolve_state_move_shape_mut(source.0, source.1)
        {
            user.shape_mut().set_direction(direction);
        }
        game.update_registered_skill_visual(address, 0);
        if let Some(skill) = game.registered_skill_mut(address) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(skill) = game.registered_skill(address) else {
        return StateCastExecutionOutcome::Rejected;
    };
    if skill.execution_stage() != Some(SkillStage::Check) {
        return StateCastExecutionOutcome::Pending;
    }
    let delay = properties.query_property(DELAY);
    let started = skill.lifecycle().started_at_ms();
    if now() < started.wrapping_add(delay) {
        return StateCastExecutionOutcome::Pending;
    }
    game.update_registered_skill_visual(address, 1);
    if begin_or_restart_promotion_state(
        game, Some(source), target, || {
            let heal_factor = properties.query_property(HEAL_RECOVER_COEFFICIENT) as u16;
            let magic_factor = properties.query_property(EM_MODIFIER) as u16;
            let keep = properties.query_property(PERSIST);
            (keep, magic_factor, heal_factor)
        },
        now,
    ) != Some(false) {
        game.update_move_shape_properties(source.0, source.1);
    }
    StateCastExecutionOutcome::EndCompleted
}
