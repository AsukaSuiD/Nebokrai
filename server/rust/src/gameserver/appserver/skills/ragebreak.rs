//! Тонкий путь яростного прорыва `CRageBreak` (0x6E) в Zone.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/ragebreak.cpp.
//! Тела Check/AI, порядок состояний (End+dtor прежнего 0x6E → новый
//! `CRageBreakState` → свип девяти id → только-End первого 0x131 → новый
//! `CCureState` → UpdateProperty → End(1)) и wire-visual с DWORD-формой
//! mode 8 перенесены буквально в `nebokrai_zone::skills::{ragebreak,fury}`
//! (якоря `0x59FF10`/`0x5A00F0`, статусы и швы — там). Здесь — делегации с
//! прежними сигнатурами: зарегистрированный вход идёт общим stateskill,
//! состояния — Zone, потребители не меняются.

use super::kernel::SkillTermination;
use super::statecast::finish_state_cast;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, execute_owned_state_skill,
    execute_player_state_skill, finish_player_state_skill,
};
use crate::gameserver::appserver::ai::playerai::CPlayerAI;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, ServerRegionOwner,
};

pub(crate) use nebokrai_zone::skills::ragebreak::RAGE_BREAK_SKILL_ID;

pub(crate) const fn is_rage_break_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    nebokrai_zone::skills::ragebreak::is_rage_break_dispatch(dispatch)
}

struct RageBreak;

impl RegisteredStateSkill for RageBreak {
    const ID: u32 = RAGE_BREAK_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::RageBreak;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, _target: StateSkillBeginTarget,
        runtime: &mut Runtime,
    ) -> bool {
        nebokrai_zone::skills::ragebreak::check_rage_break_cast(
            game, address, &mut || runtime.now_milliseconds(),
        )
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = nebokrai_zone::skills::ragebreak::run_rage_break_ai(
            game, address, &mut || runtime.now_milliseconds(),
        );
        finish_state_cast(game, address, outcome, runtime)
    }
}

pub(crate) fn publish_rage_break_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    nebokrai_zone::skills::ragebreak::publish_rage_break_visual(game, skill, mode);
}

pub(crate) fn cancel_player_rage_break<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, player_ai: &mut CPlayerAI,
    nonzero_end: bool, runtime: &mut Runtime,
) -> bool {
    finish_player_state_skill::<RageBreak, Runtime>(
        game, player_id, player_ai, i32::from(nonzero_end), SkillTermination::Cancelled, runtime,
    )
}

pub(crate) fn execute_player_rage_break<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, dispatch: PlayerSkillDispatch,
    player_ai: &mut CPlayerAI, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_player_state_skill::<RageBreak, Runtime>(game, player_id, dispatch, player_ai, runtime)
}

pub(crate) fn execute_owned_monster_rage_break<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<RageBreak, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}
