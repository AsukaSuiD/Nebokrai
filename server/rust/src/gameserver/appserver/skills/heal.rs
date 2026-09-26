//! Периодическое лечение CHeal/CHeal2/CSuperHeal/CSuperHeal2.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/heal{,2}.cpp
//! и superheal{,2}.cpp. Тела Check/AI перенесены буквально в Zone
//! `skills/{heal,healstate}.rs` (порция №6a «state-касты пятёрки +
//! heal-квартет»; машинные якоря — reuse(0x2715)+tick visual13/GS0278 →
//! GetTargetPath безусловно → distance-пара → MP; AI: MP→OnChangeStates→
//! CAN→направление→visual0→delay→visual1→формула `coeff*weapon*0.01+const`
//! (FISTP) → QueryProperty(6001) затем 10002 → ctor state(J,J) →
//! Begin(U,S) — см. там). **Fix той же порцией:** ключ частоты тика
//! `TARGET_AFFECT_FREQUENCY` — машинно `6001` (во всех четырёх AI, якорь
//! CHeal `0x581861`), документирующая поправка прежнего `5002` живёт теперь
//! в Zone `skills/heal.rs` с машинным якорем.
//! Здесь — делегации с прежними сигнатурами: общий зарегистрированный цикл
//! и visual остаются у stateskill/playercast, Check-скелет
//! `rangedweaponcast` объявлен швом среди Zone-фасадов; состояние
//! устанавливает Zone `skills/healstate.rs`. Потребители не меняются.

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
use crate::gameserver::appserver::states::state::resolve_skill_sufferer;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner,
};

pub(crate) const HEAL_SKILL_ID: u32 = nebokrai_zone::skills::heal::HEAL_SKILL_ID;

/// Прежний предикат семейства по ID навыка (петлевые потребители
/// расписания и монстровой диспетчеризации).
pub(crate) use nebokrai_zone::skills::heal::is_heal_skill;

pub(crate) fn check_heal_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: Option<(i32, ShapeIdentity)>,
    original_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    nebokrai_zone::skills::heal::check_heal_cast(
        game, instance, original_user, original_target, &mut || runtime.now_milliseconds(),
    )
}

pub(crate) fn run_heal_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    state_cast_outcome(nebokrai_zone::skills::heal::run_heal_ai(
        game, instance, &mut || runtime.now_milliseconds(),
    ))
}

pub(crate) fn execute_player_heal<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let original_target = if game.registered_skill(instance).is_some_and(|skill| skill.player_dispatch().is_none()) {
        dispatch.object_target().and_then(|target|
            original_user.and_then(|source| game.player_skill_begin_object(source.0, target)))
    } else { None };
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::Heal,
        |game, instance, _, runtime| {
            let target = if matches!(dispatch, PlayerSkillDispatch::Point { .. }) {
                game.registered_skill(instance).and_then(|skill|
                    resolve_skill_sufferer(game, skill.lifecycle()).or_else(|| {
                        nebokrai_zone::skills::statecast::state_cast_participant(game, skill.lifecycle().user())
                    }))
            } else { original_target };
            let accepted = check_heal_cast(game, instance, original_user, target, runtime);
            if !accepted { game.update_registered_skill_visual(instance, 2); }
            accepted
        },
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_heal_ai,
    )
}

struct HealSkill<const ID: u32>;
impl<const ID: u32> RegisteredStateSkill for HealSkill<ID> {
    const ID: u32 = ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::Heal;
    const VISUAL_TARGET: StateSkillVisualTarget = StateSkillVisualTarget::SuffererOrUser;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, begin_target: StateSkillBeginTarget,
        runtime: &mut Runtime,
    ) -> bool {
        let Some(skill) = game.registered_skill(instance) else { return false; };
        let user = nebokrai_zone::skills::statecast::state_cast_participant(game, skill.lifecycle().user());
        let target = begin_target.resolve(game, skill, true);
        check_heal_cast(game, instance, user, target, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = run_heal_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => end_state_skill(game, instance, 0, runtime),
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse =>
                end_state_skill(game, instance, 1, runtime),
            _ => outcome,
        }
    }
}

pub(crate) fn execute_owned_monster_heal<const ID: u32, Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<HealSkill<ID>, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}

pub(crate) fn publish_heal_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    nebokrai_zone::skills::heal::publish_heal_visual(game, skill, mode);
}
