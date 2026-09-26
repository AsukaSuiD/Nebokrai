//! Ярость CFury (0x1A3), gameserver.exe/GameServer.pdb,
//! appserver/skills/fury.cpp. Общая RP-подготовка обслуживает также
//! CRageBreak из appserver/skills/ragebreak.cpp. Сами RP-хелперы перенесены
//! буквально в `nebokrai_zone::skills::fury` порцией T4 (с приездом
//! CRageBreak хелпер стал разделяемым); здесь — делегации-пeреадресации на
//! них, реализация RP-шва и собственное тело Fury без изменений.
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

use super::curestate::{CureState, begin_primary_cure_state};
use super::furystate::{FuryState, begin_primary_fury_state};
use super::kernel::SkillTermination;
use super::ragebreakstate::{RAGE_BREAK_STATE_ID, RageBreakState};
use super::statecast::finish_state_cast;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, StateSkillVisualTarget, end_state_skill,
    execute_owned_state_skill, execute_player_state_skill, finish_player_state_skill,
    publish_state_skill_visual,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, ServerRegionOwner,
};

pub(super) use nebokrai_zone::skills::fury::{RageRpPolicy, RageSkillEffect};

pub(crate) const FURY_SKILL_ID: u32 = 0x1a3;
const PERSIST: u32 = 10_002;
const ATTACK_GAIN: u32 = 105;

/// RP-шов Zone яростных навыков над прежним `CPlayer` (реализация
/// единственная; Zone не зависит от старого пакета).
impl nebokrai_zone::skills::fury::RageCastPlayer for CPlayer {
    fn rp(&self) -> u16 { self.rp() }

    fn set_rp(&mut self, rp: u16) { self.set_rp(rp) }
}

pub(crate) fn remove_reached_conflict_states(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity,
) {
    nebokrai_zone::skills::fury::remove_reached_conflict_states(game, region_id, holder)
}

pub(super) fn check_rage_skill_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, address: RegisteredSkill, policy: RageRpPolicy, runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::skills::fury::check_rage_skill_cast(
        game, address, policy, &mut || runtime.now_milliseconds(),
    )
}

/// Делегация Zone-подготовки: точки прежнего `end_state_skill(0/1)` идут
/// тем же полным End через общий terminal statecast-адаптера.
pub(super) fn prepare_rage_skill_effect<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
) -> ControlFlow<QueuedSkillExecutionOutcome, RageSkillEffect> {
    match nebokrai_zone::skills::fury::prepare_rage_skill_effect(
        game, address, &mut || runtime.now_milliseconds(),
    ) {
        ControlFlow::Continue(effect) => ControlFlow::Continue(effect),
        ControlFlow::Break(outcome) => {
            ControlFlow::Break(finish_state_cast(game, address, outcome, runtime))
        }
    }
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
