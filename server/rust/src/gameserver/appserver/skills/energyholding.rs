//! Накопление энергии CEnergyHolding (0x89).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/energyholding.cpp.
//! Тела Check/AI перенесены буквально в Zone `skills/energyholding.rs`
//! (порция №6c «self/zone-касты»; основание и машинные статусы см. там).
//! Здесь — тонкие делегации с прежними сигнатурами: зарегистрированный вход
//! идёт общим playercast, швы Zone реализованы над `CGame` в
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
use nebokrai_zone::skills::energyholding;

pub(crate) use nebokrai_zone::skills::energyholding::ENERGY_HOLDING_SKILL_ID;
// Безпотребительный `PARAMETER_PERCENT` со старого пути снят порцией №6c
// (ключ процента читают из Zone напрямую).

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32, _runtime: &mut Runtime,
) -> bool {
    let _ = _runtime;
    energyholding::check_cast(game, instance, player_id, game_tick_milliseconds)
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, _runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let _ = _runtime;
    selfcast_outcome(energyholding::run_ai(game, instance, game_tick_milliseconds))
}

pub(crate) fn execute_player_energy_holding<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != ENERGY_HOLDING_SKILL_ID {
        return QueuedSkillExecutionOutcome { state: QueuedSkillExecutionState::Rejected, first_contact: false };
    }
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::SelfCast,
        check_cast, |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_ai,
    )
}
