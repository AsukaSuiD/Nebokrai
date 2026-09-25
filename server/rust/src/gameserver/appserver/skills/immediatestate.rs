//! Зарегистрированный цикл TaiJi, Origin, трёх Enlarge, Swordship и WuXing.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/{taiji,origin,
//! enlargefullmiss,enlargemaxhp,enlargemaxmp,swordship*,wuxing*}.cpp.
//! ID-карта семьи, подстановка S, ветка установки, End-политика и табличная
//! подготовка payload — zone rules `skills/immediate.rs` и `skills/wuxing.rs`;
//! здесь живой обход Game по тем же контрактам.
//!
//! Begin записывает общую базу, проверяет только исходный U и свежие свойства,
//! затем включает фазу. Здесь нет reuse-допуска, MP, visual или Move.
//! Игрок, активный и фоновый монстр используют один зарегистрированный экземпляр;
//! pending AutoStart хранит только запрос Begin, а не второй Begun/Ended.
//!
//! AI читает свойства до GetU. Swordship требует именно U; остальные семейства
//! при NULL U используют GetS. WuXing собирает все параметры ещё до этого
//! выбора, затем допускает только игрока. Нет свойств или участника — End0.
//! Новые состояния получают собственный Begin(U,U) до публикации в списке;
//! порядок удаления и установки сохраняет immediatestateinstallation.
//!
//! Swordship заканчивает AI с End0, включая успешную установку. Остальные
//! успешные ветви вызывают End1 с оружейным AfterUse и свежими часами.
//! Внешние End1/4/0 не подменяются этой политикой AI. Общий End сбрасывает фазу,
//! не меняя движение и CAN. UpdateProperty не заменяется OnChangeStates.
//! SlotMap сохраняет идентичность навыка через callbacks без копии исполнения.

use super::immediatestateinstallation::apply_immediate_state;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::playercast::execute_registered_player_cast_without_visual;
use super::rangedweaponcast::terminal;
use super::stateskill::end_state_skill;
use super::wuxing::apply_wuxing_state;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner,
};
pub(crate) use nebokrai_zone::skills::{
    immediate_completion_end_argument, is_immediate_state_skill,
};
use nebokrai_zone::skills::{immediate_ai_sufferer_fallback, prepare_wuxing_parameters};

pub(crate) fn check_immediate_state_cast(
    game: &CGame, instance: RegisteredSkill, original_user: Option<(i32, ShapeIdentity)>,
) -> bool {
    original_user.is_some() && game.registered_skill(instance)
        .is_some_and(|skill| game.skill_base_properties(skill.id(), skill.level()).is_some())
}

pub(crate) fn run_immediate_state_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if skill.execution_stage().is_none_or(|stage| stage == SkillStage::Idle) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let wuxing_parameters = prepare_wuxing_parameters(skill_id, |usage| properties.query_property(usage));
    let (region, identity) = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, region, identity).or_else(|| {
        if !immediate_ai_sufferer_fallback(skill_id) { return None; }
        let (region, identity) = resolve_skill_sufferer(game, skill.lifecycle())?;
        resolve_state_move_shape(game, region, identity)
    });
    let Some(source) = source.map(|shape| (shape.shape().get_region_id(), shape.shape().identity())) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if let Some(parameters) = wuxing_parameters {
        if source.1.object_type != 400 { return terminal(QueuedSkillExecutionState::Rejected); }
        let _ = apply_wuxing_state(game, source, skill_id, parameters, runtime);
    } else {
        let _ = apply_immediate_state(game, source, skill_id, &properties, runtime);
    }
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_immediate_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if !is_immediate_state_skill(dispatch.skill_id()) { return terminal(QueuedSkillExecutionState::Rejected); }
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast_without_visual(
        game, player_id, instance, dispatch, runtime,
        |game, instance, _, _| check_immediate_state_cast(game, instance, original_user),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_immediate_state_ai,
    )
}

pub(crate) fn execute_monster_immediate_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    skill_id: u32, _skill_level: i32, runtime: &mut Runtime,
) -> bool {
    let Some(monster) = owner.as_ref().and_then(|region| region.base().find_monster_by_id(monster_id)) else { return false; };
    let source = (monster.move_shape().shape().get_region_id(), monster.move_shape().shape().identity());
    game.with_published_region(owner, |game| {
        let Some(instance) = game.registered_move_shape_skill(source.0, source.1, skill_id) else { return false; };
        let outcome = run_immediate_state_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => { end_state_skill(game, instance, 0, runtime); }
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse => {
                end_state_skill(game, instance, immediate_completion_end_argument(skill_id), runtime);
            }
            _ => {}
        }
        true
    }).unwrap_or(false)
}
