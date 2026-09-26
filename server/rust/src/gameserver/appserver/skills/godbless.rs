//! Божественное благословение CGodBless/CGodBless2.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/godbless{,2}.cpp.
//! Тела Check/AI и живые callbacks состояний перенесены буквально в Zone
//! `skills/{godbless,godblessstate}.rs` (порция №6a «state-касты пятёрки +
//! heal-квартет»; основание — Check `0x1B02A0` (не-Player → 1 до MP/Move0;
//! arg-null → тихий 0; cost0 → тихий 0; signed-diff → visual7/GS0288 или
//! Move0 → 1), AI `0x1B0480`; CGodBless2 AI `0x150990` (visual10/GS0305
//! при не-Monster); End(H) 3-fold `0x1502F0` — см. там и в записи аудита
//! «Zone skills: машинная разведка battlefairy-навыков (порция №6)»).
//! Здесь — делегации с прежними сигнатурами: общий цикл и visual остаются
//! у stateskill/playercast, первая установка состояния — у
//! `CGame::install_god_bless_state` (`game/godbless.rs`), часы и установка
//! приходят связкой через Zone-трейт `GodBlessCastRuntime`; потребители не
//! меняются. Замена состояния, в отличие от GodBless, не вызывает у
//! GodBless2 DelExStateByType — это различие общего установщика.

use super::godbless2::GOD_BLESS_2_SKILL_ID;
use super::kernel::SkillExecutionKernel;
use super::playercast::execute_registered_player_cast;
use super::statecast::state_cast_outcome;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, StateSkillVisualTarget, end_state_skill,
    execute_owned_state_skill,
};
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner,
};
use nebokrai_zone::skills::godbless::GodBlessCastRuntime;

pub(crate) use nebokrai_zone::skills::godbless::GOD_BLESS_SKILL_ID;

pub(crate) const fn is_god_bless_skill(skill_id: u32) -> bool {
    nebokrai_zone::skills::godbless::is_god_bless_skill(skill_id)
}

pub(crate) fn check_god_bless_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: Option<(i32, ShapeIdentity)>,
    runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::skills::godbless::check_cast(
        game, instance, original_user, &mut || runtime.now_milliseconds(),
    )
}

/// Владелец часов и первичной установки состояния благословения: runtime
/// прежнего главного цикла (в нём же Begin часов состояния).
struct GodBlessRuntime<'a, Runtime>(&'a mut Runtime);

impl<Runtime: GameMainLoopRuntime> GodBlessCastRuntime<CGame> for GodBlessRuntime<'_, Runtime> {
    fn now_milliseconds(&mut self) -> u32 {
        self.0.now_milliseconds()
    }

    fn install_god_bless_state(
        &mut self,
        game: &mut CGame,
        user: (i32, ShapeIdentity),
        sufferer: (i32, ShapeIdentity),
        skill_id: u32,
        create: &mut dyn FnMut() -> nebokrai_zone::effects::GodBlessState,
    ) -> bool {
        game.install_god_bless_state(user, sufferer, skill_id, create, self.0)
    }
}

pub(crate) fn run_god_bless_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    state_cast_outcome(nebokrai_zone::skills::godbless::run_god_bless_ai(
        game, instance, &mut GodBlessRuntime(runtime),
    ))
}

pub(crate) fn execute_player_god_bless<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::GodBless,
        |game, instance, _, runtime| check_god_bless_cast(game, instance, original_user, runtime),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_god_bless_ai,
    )
}

struct GodBlessSkill<const ID: u32>;
impl<const ID: u32> RegisteredStateSkill for GodBlessSkill<ID> {
    const ID: u32 = ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::GodBless;
    const VISUAL_FAILURES: &'static [u32] = if ID == GOD_BLESS_2_SKILL_ID { &[2, 7, 10, 13] } else { &[2, 7, 13] };
    const VISUAL_TARGET: StateSkillVisualTarget = StateSkillVisualTarget::SuffererOrUser;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, _begin_target: StateSkillBeginTarget,
        runtime: &mut Runtime,
    ) -> bool {
        let user = game.registered_skill(instance)
            .and_then(|skill| nebokrai_zone::skills::statecast::state_cast_participant(game, skill.lifecycle().user()));
        check_god_bless_cast(game, instance, user, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = run_god_bless_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => end_state_skill(game, instance, 0, runtime),
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse =>
                end_state_skill(game, instance, 1, runtime),
            _ => outcome,
        }
    }
}

pub(crate) fn execute_owned_monster_god_bless<const ID: u32, Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<GodBlessSkill<ID>, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}

pub(crate) fn publish_god_bless_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    nebokrai_zone::skills::godbless::publish_god_bless_visual(game, skill, mode);
}
