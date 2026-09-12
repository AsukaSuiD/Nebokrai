//! Усиление CPromotion (0x142), gameserver.exe/GameServer.pdb,
//! appserver/skills/promotion.cpp; состояние принадлежит promotionstate.cpp.
//!
//! Зарегистрированный Begin предшествует visual loop1 и CheckCast. Нулевая
//! стоимость MP у игрока — тихий отказ CheckCast; положительная проверяется
//! по знаку DWORD-разности и только при успехе запрещает движение. Монстр
//! MP не проверяет и движение в Begin не запрещает. Первый AI повторно
//! расходует MP игрока, публикует OnChangeStates, задаёт CAN_BE_BREAKED,
//! направление и visual(0). Concrete-флаги работы/проверки выражены фазой,
//! независимо от базовой прерываемости. Reuse/delay — wrapping-сроки DWORD.
//!
//! AI сохраняет U/S и owner свойств до callbacks; U без region-link завершает
//! навык до проверки смерти S. После delay нет повторного пути: visual(1)
//! разрешает S заново с fallback на U, наложение использует прежнюю S.
//! Первый прежний CPromotionState получает только Restart; иначе новый
//! primary Begin предшествует append, затем UpdateProperty источника.
//! End очищает concrete фазу, возвращает движение actual U
//! и завершает тот же зарегистрированный экземпляр через CStateSkill.
//! Каноническая арена состояния и его codec не дублируются в исполнении.

use super::kernel::{skill_is_restored, SkillStage, SkillTermination};
use super::promotionstate::begin_or_restart_promotion_state;
use super::stateskill::{
    RegisteredStateSkill, end_state_skill, execute_owned_state_skill, execute_player_state_skill,
    finish_player_state_skill, publish_state_skill_visual, state_skill_outcome,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner,
};
use crate::public::tools::get_line_direction;

pub(crate) const PROMOTION_SKILL_ID: u32 = 0x142;
const PLAYER_TYPE: i32 = 400;
const MP_LOSS: u32 = 2;
const EM_MODIFIER: u32 = 20_015;
const HEAL_RECOVER_COEFFICIENT: u32 = 20_019;
const DELAY: u32 = 10_001;
const PERSIST: u32 = 10_002;
const MAX_DISTANCE: u32 = 5_003;
const REUSE: u32 = 10_005;
const CAN_BREAK: u32 = 10_006;

fn participant(game: &CGame, value: (i32, ShapeIdentity)) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, value.0, value.1)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

pub(crate) fn publish_promotion_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    publish_state_skill_visual::<Promotion>(game, skill, mode);
}

fn failure(
    game: &mut CGame, address: RegisteredSkill, source: (i32, ShapeIdentity),
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

struct Promotion;

impl RegisteredStateSkill for Promotion {
    const ID: u32 = PROMOTION_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::Promotion;
    const VISUAL_FALLBACK_TO_USER: bool = true;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
    ) -> bool {
        let Some(skill) = game.registered_skill(address) else { return false; };
        let Some(source) = participant(game, skill.lifecycle().user()) else { return false; };
        let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
        let Some(target) = resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|target| participant(game, target))
        else {
            failure(game, address, source, 10, b"GS0286", None);
            return false;
        };
        let reuse = properties.query_property(REUSE);
        if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
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
            if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) {
                user.set_moveable(false);
            }
        }
        true
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let Some(skill) = game.registered_skill(address) else {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        };
        if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
            return state_skill_outcome(QueuedSkillExecutionState::Pending);
        }
        let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
            return end_state_skill(game, address, 0, runtime);
        };
        let source = participant(game, skill.lifecycle().user());
        let target = resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|target| participant(game, target));
        let (Some(source), Some(target)) = (source, target) else {
            return end_state_skill(game, address, 0, runtime);
        };
        if resolve_state_move_shape(game, source.0, source.1)
            .is_none_or(|shape| !shape.shape().is_assigned_to_server_region())
        {
            return end_state_skill(game, address, 0, runtime);
        }
        if game.move_shape_health(target.0, target.1) == Some(0) {
            failure(game, address, source, 10, b"GS0285", None);
            return end_state_skill(game, address, 0, runtime);
        }
        if game.registered_skill(address).and_then(MoveShapeSkill::execution_stage) == Some(SkillStage::Begin) {
            if source.1.object_type == PLAYER_TYPE {
                let Some(mana) = game.find_player(source.1.id).map(|player| player.mana()) else {
                    return end_state_skill(game, address, 0, runtime);
                };
                let remaining = mana.wrapping_sub(properties.query_property(MP_LOSS));
                if (remaining as i32) < 0 {
                    failure(game, address, source, 7, b"GS0288", Some(properties.query_property(MP_LOSS)));
                    return end_state_skill(game, address, 0, runtime);
                }
                if let Some(player) = game.find_player_mut(source.1.id) {
                    player.set_mana(remaining);
                }
                let _ = game.publish_player_states(source.1.id);
            }
            if let Some(skill) = game.registered_skill_mut(address) {
                skill.lifecycle_mut().set_available(properties.query_property(CAN_BREAK) != 0);
            }
            let direction = (|| {
                let user = resolve_state_move_shape(game, source.0, source.1)?.shape();
                let target = resolve_state_move_shape(game, target.0, target.1)?.shape();
                Some(get_line_direction(
                    user.get_tile_x().unwrap_or(i32::MIN), user.get_tile_y().unwrap_or(i32::MIN),
                    target.get_tile_x().unwrap_or(i32::MIN), target.get_tile_y().unwrap_or(i32::MIN),
                ))
            })();
            if let Some(direction) = direction
                && let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1)
            {
                user.shape_mut().set_direction(direction);
            }
            game.update_registered_skill_visual(address, 0);
            if let Some(skill) = game.registered_skill_mut(address) {
                let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
            }
        }
        let Some(skill) = game.registered_skill(address) else {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        };
        if skill.execution_stage() != Some(SkillStage::Check) {
            return state_skill_outcome(QueuedSkillExecutionState::Pending);
        }
        let delay = properties.query_property(DELAY);
        let started = skill.lifecycle().started_at_ms();
        if runtime.now_milliseconds() < started.wrapping_add(delay) {
            return state_skill_outcome(QueuedSkillExecutionState::Pending);
        }
        game.update_registered_skill_visual(address, 1);
        if begin_or_restart_promotion_state(
            game, Some(source), target, || {
                let heal_factor = properties.query_property(HEAL_RECOVER_COEFFICIENT) as u16;
                let magic_factor = properties.query_property(EM_MODIFIER) as u16;
                let keep = properties.query_property(PERSIST);
                (keep, magic_factor, heal_factor)
            },
            &mut || runtime.now_milliseconds(),
        ) != Some(false) {
            let _ = game.update_move_shape_properties(source.0, source.1);
        }
        end_state_skill(game, address, 1, runtime)
    }
}

pub(crate) fn complete_player_promotion<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> bool {
    finish_player_state_skill::<Promotion, Runtime>(
        game, player_id, ai, 1, SkillTermination::Completed, runtime,
    )
}

pub(crate) fn cancel_player_promotion<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, nonzero_end: bool, runtime: &mut Runtime,
) -> bool {
    finish_player_state_skill::<Promotion, Runtime>(
        game, player_id, ai, i32::from(nonzero_end), SkillTermination::Cancelled, runtime,
    )
}

pub(crate) fn execute_player_promotion<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_state_skill::<Promotion, Runtime>(game, player_id, dispatch, ai, runtime)
}

pub(crate) fn execute_owned_monster_promotion<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<Promotion, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}
