//! Очищение CCure (0x131), gameserver.exe + GameServer.pdb,
//! appserver/skills/cure.cpp. Тела Check/AI, диагностика, обход состояний
//! и выбор снимаемых состояний перенесены буквально в Zone
//! `skills/{cure,curestate}.rs` (порция №6a «state-касты пятёрки +
//! heal-квартет»; основание и машинные статусы — Begin `0x1AD3A0`,
//! DoesTargetEffective `0x1AD590`, CastCure `0x1ADB10`, AI `0x1AE110` —
//! см. там и в записи аудита «Zone skills: машинная разведка
//! battlefairy-навыков (порция №6)»). Здесь — делегации с прежними
//! сигнатурами: зарегистрированный вход идёт общим stateskill, швы Zone
//! реализованы над `CGame` в `skills/statecast.rs`; разрешение
//! `begin_target`, полный End и потребители не меняются.

use super::kernel::SkillTermination;
use super::statecast::finish_state_cast;
use super::stateskill::{RegisteredStateSkill, StateSkillBeginTarget, StateSkillVisualTarget,
    execute_owned_state_skill, execute_player_state_skill, finish_player_state_skill};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, ServerRegionOwner,
};

pub(crate) use nebokrai_zone::effects::CURE_STATE_SKILL_ID as CURE_SKILL_ID;

pub(crate) fn publish_cure_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    nebokrai_zone::skills::cure::publish_cure_visual(game, skill, mode);
}

/// Прежний вход завершения конкретного curable состояния через общий End ветки.
pub(crate) fn finish_curable_state(
    game: &mut CGame, region_id: i32, target: ShapeIdentity, state_id: u32, now_ms: u32,
) -> bool {
    nebokrai_zone::skills::cure::finish_curable_state(game, region_id, target, state_id, now_ms)
}

struct Cure;

impl RegisteredStateSkill for Cure {
    const ID: u32 = CURE_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::Cure;
    const VISUAL_TARGET: StateSkillVisualTarget = StateSkillVisualTarget::SuffererOrUser;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, begin_target: StateSkillBeginTarget, runtime: &mut Runtime,
    ) -> bool {
        let Some(skill) = game.registered_skill(address) else { return false; };
        let target = begin_target.resolve(game, skill, true);
        nebokrai_zone::skills::cure::check_cast(game, address, target, &mut || runtime.now_milliseconds())
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = nebokrai_zone::skills::cure::run_cure_ai(game, address, &mut || runtime.now_milliseconds());
        finish_state_cast(game, address, outcome, runtime)
    }
}

/// Прежний предикат диспетчера расписания игрока (ID команд выбора CCure).
pub(crate) const fn is_cure_target(dispatch: PlayerSkillDispatch) -> bool {
    nebokrai_zone::skills::cure::is_cure_target(dispatch)
}

pub(crate) fn complete_player_cure<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> bool {
    finish_player_state_skill::<Cure, Runtime>(game, player_id, ai, 1, SkillTermination::Completed, runtime)
}

pub(crate) fn cancel_player_cure<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, ai: &mut CPlayerAI, nonzero_end: bool, runtime: &mut Runtime,
) -> bool {
    finish_player_state_skill::<Cure, Runtime>(game, player_id, ai, i32::from(nonzero_end), SkillTermination::Cancelled, runtime)
}

pub(crate) fn execute_player_cure<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch, ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_state_skill::<Cure, Runtime>(game, player_id, dispatch, ai, runtime)
}

pub(crate) fn execute_owned_monster_cure<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<Cure, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}
