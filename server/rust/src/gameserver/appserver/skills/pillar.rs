//! Защитная стойка CPillar (0x74).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/pillar.cpp.
//! Тела Check/AI перенесены буквально в Zone `skills/pillar.rs` (порция №6c
//! «self/zone-касты»; основание и машинные статусы см. там). Здесь — тонкие
//! делегации с прежними сигнатурами: зарегистрированный вход идёт общим
//! playercast с захватом исходного U, швы Zone реализованы над `CGame` в
//! `skills/selfcast.rs`; потребители не меняются.

use super::kernel::SkillExecutionKernel;
use super::playercast::execute_registered_player_cast;
use super::selfcast::selfcast_outcome;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    game_tick_milliseconds,
};
use nebokrai_zone::skills::pillar;

pub(crate) use nebokrai_zone::skills::PILLAR_SKILL_ID;

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: (i32, ShapeIdentity),
    _runtime: &mut Runtime,
) -> bool {
    let _ = _runtime;
    pillar::check_cast(game, instance, original_user, game_tick_milliseconds)
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, _runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let _ = _runtime;
    selfcast_outcome(pillar::run_ai(game, instance, game_tick_milliseconds))
}

pub(crate) fn execute_player_pillar<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != PILLAR_SKILL_ID {
        return QueuedSkillExecutionOutcome { state: QueuedSkillExecutionState::Rejected, first_contact: false };
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
