//! Ярость CFury (0x1A3), gameserver.exe/GameServer.pdb,
//! appserver/skills/fury.cpp. Общая RP-подготовка обслуживает также
//! CRageBreak из appserver/skills/ragebreak.cpp.
//!
//! Игрок и монстр используют одно зарегистрированное исполнение. Begin
//! создаёт visual до проверки U; RP расходуется только первым AI игрока,
//! затем задаются прерываемость и абсолютная задержка. Нулевая стоимость
//! RP допустима. Отказ Begin не публикует дополнительный visual2.
//!
//! Наличие первого RageBreak продлевает только его таймер и завершает
//! навык: Fury, Cure и пересчёт свойств в этой ветви не выполняются.
//! Иначе Fury накапливается через Begin(U,U) и append, конфликтующие
//! состояния снимаются по живым позициям, затем добавляется Cure и
//! пересчитываются свойства. Все callbacks видят опубликованный AI/регион.
//! Полный End принадлежит захваченному экземпляру навыка.

use std::ops::ControlFlow;

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME};
use super::curestate::{CureState, begin_primary_cure_state};
use super::furystate::{FuryState, begin_primary_fury_state};
use super::kernel::{SkillStage, SkillTermination, skill_is_restored};
use super::ragebreakstate::{RAGE_BREAK_STATE_ID, RageBreakState};
use super::skillbaseproperties::CSkillBaseProperties;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, StateSkillVisualTarget, end_state_skill,
    execute_owned_state_skill, execute_player_state_skill, finish_player_state_skill,
    publish_state_skill_visual, state_skill_outcome,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner,
};

pub(crate) const FURY_SKILL_ID: u32 = 0x1a3;
const PLAYER_TYPE: i32 = 400;
const RP_LOSS: u32 = 3;
const CAN_BREAK: u32 = 10_006;
const PERSIST: u32 = 10_002;
const ATTACK_GAIN: u32 = 105;
const CONFLICTING_STATES: [u32; 9] = [
    0x138, 0xd2, 0xc9, 0x67, 0x192, 0x191, 0x198, 0x199, 0x1a6,
];

fn participant(game: &CGame, source: (i32, ShapeIdentity)) -> Option<(i32, ShapeIdentity)> {
    let shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
    Some((shape.get_region_id(), shape.identity()))
}

pub(crate) fn remove_reached_conflict_states(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity,
) {
    let mut position = 0;
    loop {
        let Some(shape) = resolve_state_move_shape(game, region_id, holder) else { return; };
        if position >= shape.state_slot_count() { break; }
        if shape.state_at(position).is_some_and(|(_, state)| CONFLICTING_STATES.contains(&state.state_id())) {
            let _ = end_and_destroy_state_at(game, region_id, holder, position);
        }
        position += 1;
    }
}

fn fail_rp(
    game: &mut CGame, address: RegisteredSkill, source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
) {
    game.update_registered_skill_visual(address, 8);
    if source.1.object_type == PLAYER_TYPE {
        game.send_skill_system_info_with_unsigned(
            source.1.id, b"GS0289", properties.query_property(RP_LOSS),
        );
    }
}

/// Различие двух RP-навыков касается только первого Check, не расхода в AI.
pub(super) enum RageRpPolicy {
    AllowZero,
    RequirePositive,
}

pub(super) struct RageSkillEffect {
    pub(super) source: (i32, ShapeIdentity),
    pub(super) properties: CSkillBaseProperties,
}

pub(super) fn check_rage_skill_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, address: RegisteredSkill, policy: RageRpPolicy, runtime: &mut Runtime,
) -> bool {
    let Some(skill) = game.registered_skill(address) else { return false; };
    let Some(source) = participant(game, skill.lifecycle().user()) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return false;
    };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(address, 13);
        if source.1.object_type == PLAYER_TYPE {
            game.send_skill_system_info(source.1.id, b"GS0278");
        }
        return false;
    }
    if source.1.object_type != PLAYER_TYPE { return true; }
    // RageBreak проверяет стоимость до чтения RP, затем запрашивает её вновь.
    // Fury сразу читает RP и только после этого делает единственный запрос.
    if matches!(policy, RageRpPolicy::RequirePositive) && properties.query_property(RP_LOSS) == 0 {
        return false;
    }
    let Some(rp) = game.find_player(source.1.id).map(|player| player.rp()) else { return false; };
    if (u32::from(rp).wrapping_sub(properties.query_property(RP_LOSS)) as i32) < 0 {
        fail_rp(game, address, source, &properties);
        return false;
    }
    if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) {
        user.set_moveable(false);
    }
    true
}

/// Общая подготовка Fury/RageBreak заканчивается visual1. Таблица этого AI
/// и его U передаются обработчику состояний, не разрешаясь заново по ID навыка.
pub(super) fn prepare_rage_skill_effect<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
) -> ControlFlow<QueuedSkillExecutionOutcome, RageSkillEffect> {
    let Some(skill) = game.registered_skill(address) else {
        return ControlFlow::Break(state_skill_outcome(QueuedSkillExecutionState::Rejected));
    };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return ControlFlow::Break(state_skill_outcome(QueuedSkillExecutionState::Pending));
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return ControlFlow::Break(end_state_skill(game, address, 0, runtime));
    };
    let Some(source) = participant(game, skill.lifecycle().user()) else {
        return ControlFlow::Break(end_state_skill(game, address, 0, runtime));
    };
    if game.move_shape_health(source.0, source.1) == Some(0) {
        // Смерть U завершает оба навыка с AfterUse, хотя усиления не создаются.
        game.update_registered_skill_visual(address, 2);
        return ControlFlow::Break(end_state_skill(game, address, 1, runtime));
    }
    if game.registered_skill(address).and_then(MoveShapeSkill::execution_stage) == Some(SkillStage::Begin) {
        if source.1.object_type == PLAYER_TYPE {
            let Some(rp) = game.find_player(source.1.id).map(|player| player.rp()) else {
                return ControlFlow::Break(end_state_skill(game, address, 0, runtime));
            };
            let remaining = u32::from(rp).wrapping_sub(properties.query_property(RP_LOSS));
            if (remaining as i32) < 0 {
                fail_rp(game, address, source, &properties);
                return ControlFlow::Break(end_state_skill(game, address, 0, runtime));
            }
            if let Some(player) = game.find_player_mut(source.1.id) {
                player.set_rp(remaining as u16);
            }
            let _ = game.publish_player_states(source.1.id);
        }
        if let Some(skill) = game.registered_skill_mut(address) {
            skill.lifecycle_mut().set_available(properties.query_property(CAN_BREAK) != 0);
        }
        game.update_registered_skill_visual(address, 0);
        if let Some(skill) = game.registered_skill_mut(address) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    if game.registered_skill(address).and_then(MoveShapeSkill::execution_stage) != Some(SkillStage::Check) {
        return ControlFlow::Break(state_skill_outcome(QueuedSkillExecutionState::Pending));
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(address).map(|skill| skill.lifecycle().started_at_ms()) else {
        return ControlFlow::Break(state_skill_outcome(QueuedSkillExecutionState::Rejected));
    };
    if started.wrapping_add(delay) > runtime.now_milliseconds() {
        return ControlFlow::Break(state_skill_outcome(QueuedSkillExecutionState::Pending));
    }
    game.update_registered_skill_visual(address, 1);
    ControlFlow::Continue(RageSkillEffect { source, properties })
}

struct Fury;

impl RegisteredStateSkill for Fury {
    const ID: u32 = FURY_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::Fury;
    const VISUAL_FAILURES: &'static [u32] = &[2, 7, 8, 13];
    const VISUAL_DWORD_FAILURES: &'static [u32] = &[8];
    const VISUAL_TARGET: StateSkillVisualTarget = StateSkillVisualTarget::User;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, _begin_target: StateSkillBeginTarget,
        runtime: &mut Runtime,
    ) -> bool {
        check_rage_skill_cast(game, address, RageRpPolicy::AllowZero, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let RageSkillEffect { source, properties } = match prepare_rage_skill_effect(game, address, runtime) {
            ControlFlow::Continue(effect) => effect,
            ControlFlow::Break(outcome) => return outcome,
        };

        // Это именно Restart таймера, а не повторный Begin состояния:
        // ни нового visual, ни пересчёта свойств здесь нет.
        if let Some((_, key)) = resolve_state_move_shape(game, source.0, source.1)
            .and_then(|shape| shape.find_state_position(|state| state.state_id() == RAGE_BREAK_STATE_ID))
        {
            let now = runtime.now_milliseconds();
            if let Some(state) = resolve_state_move_shape_mut(game, source.0, source.1)
                .and_then(|shape| shape.applied_state_mut::<RageBreakState>(key))
            {
                state.restart_timer(now);
            }
            return end_state_skill(game, address, 1, runtime);
        }

        let keep = properties.query_property(PERSIST);
        let gain = properties.query_property(ATTACK_GAIN) as i32;
        let fury = FuryState::new(keep, gain);
        let _ = begin_primary_fury_state(
            game, source.0, source.1, Some(source), Some(source), fury,
            &mut || runtime.now_milliseconds(),
        );
        remove_reached_conflict_states(game, source.0, source.1);
        let cure = CureState::new(properties.query_property(PERSIST));
        let _ = begin_primary_cure_state(
            game, source.0, source.1, Some(source), Some(source), cure,
            &mut || runtime.now_milliseconds(),
        );
        let _ = game.update_move_shape_properties(source.0, source.1);
        end_state_skill(game, address, 1, runtime)
    }
}

pub(crate) fn publish_fury_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    publish_state_skill_visual::<Fury>(game, skill, mode);
}

pub(crate) const fn is_fury_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    match dispatch {
        PlayerSkillDispatch::SelfTarget { skill_id, .. }
        | PlayerSkillDispatch::Point { skill_id, .. }
        | PlayerSkillDispatch::Object { skill_id, .. } => skill_id == FURY_SKILL_ID,
    }
}

pub(crate) fn cancel_player_fury<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI,
    nonzero_end: bool, runtime: &mut Runtime,
) -> bool {
    finish_player_state_skill::<Fury, Runtime>(
        game, player_id, player_ai, i32::from(nonzero_end), SkillTermination::Cancelled, runtime,
    )
}

pub(crate) fn execute_player_fury<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_state_skill::<Fury, Runtime>(game, player_id, dispatch, player_ai, runtime)
}

pub(crate) fn execute_owned_fury<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<Fury, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}
