//! Усиление CPromotion (0x142), gameserver.exe/GameServer.pdb,
//! appserver/skills/promotion.cpp; состояние принадлежит promotionstate.cpp.
//! Тела Check/AI перенесены буквально в Zone `skills/{promotion,
//! promotionstate}.rs` (порция №6a «state-касты пятёрки + heal-квартет»;
//! основание — Begin `0x1686A0`, AI `0x169110`, Restart-fold состояния
//! `0x1FD450` — см. там и в записи аудита «Zone skills: машинная разведка
//! battlefairy-навыков (порция №6)»). Здесь — делегации с прежними
//! сигнатурами: зарегистрированный вход идёт общим stateskill, швы Zone
//! реализованы над `CGame` в `skills/statecast.rs`; потребители не
//! меняются.

use super::kernel::SkillTermination;
use super::statecast::finish_state_cast;
use super::stateskill::{RegisteredStateSkill, StateSkillVisualTarget,
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

pub(crate) use nebokrai_zone::skills::promotion::PROMOTION_SKILL_ID;

pub(crate) fn publish_promotion_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    nebokrai_zone::skills::promotion::publish_promotion_visual(game, skill, mode);
}

struct Promotion;

impl RegisteredStateSkill for Promotion {
    const ID: u32 = PROMOTION_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::Promotion;
    const VISUAL_TARGET: StateSkillVisualTarget = StateSkillVisualTarget::SuffererOrUser;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, begin_target: super::stateskill::StateSkillBeginTarget, runtime: &mut Runtime,
    ) -> bool {
        let Some(skill) = game.registered_skill(address) else { return false; };
        let target = begin_target.resolve(game, skill, false);
        nebokrai_zone::skills::promotion::check_cast(game, address, target, &mut || runtime.now_milliseconds())
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, address: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = nebokrai_zone::skills::promotion::run_promotion_ai(game, address, &mut || runtime.now_milliseconds());
        finish_state_cast(game, address, outcome, runtime)
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
