//! Боевой клич CRoar (0x83).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/roar.cpp.
//! Тела Check/AI и клеточного обхода перенесены буквально в Zone
//! `skills/roar.rs` (порция №6c «self/zone-касты»; основание и машинные
//! статусы см. там). Здесь — тонкие делегации с прежними сигнатурами:
//! зарегистрированный вход идёт общим playercast (visual2 по отказу Check —
//! обёртка этого владельца), швы Zone реализованы над `CGame` в
//! `skills/selfcast.rs`; потребители не меняются.

use super::kernel::SkillExecutionKernel;
use super::playercast::execute_registered_player_cast;
use super::selfcast::selfcast_outcome;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    game_tick_milliseconds,
};
use nebokrai_zone::skills::roar;

pub(crate) use nebokrai_zone::skills::roar::ROAR_SKILL_ID;

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32, _runtime: &mut Runtime,
) -> bool {
    let _ = _runtime;
    roar::check_cast(game, instance, player_id, game_tick_milliseconds)
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    selfcast_outcome(roar::run_ai(game, instance, runtime, game_tick_milliseconds))
}

pub(crate) fn execute_player_roar<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != ROAR_SKILL_ID {
        return QueuedSkillExecutionOutcome { state: QueuedSkillExecutionState::Rejected, first_contact: false };
    }
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::SelfCast,
        |game, instance, player_id, runtime| {
            let accepted = check_cast(game, instance, player_id, runtime);
            if !accepted { game.update_registered_skill_visual(instance, 2); }
            accepted
        },
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_ai,
    )
}
