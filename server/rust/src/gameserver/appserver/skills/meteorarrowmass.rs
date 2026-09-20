//! Накопление метеорных стрел CMeteorArrowMass (0xCC).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/meteorarrowmass.cpp.
//! Общий Begin сохраняет исходного U и ранний отсчёт. Check требует reuse,
//! игроку — лук категории 3, ненулевую цену MP и неотрицательную signed
//! DWORD-разность перед Move0. Источник не типа Player допускается без Move0;
//! путь, S, смерть и регион в Check не проверяются. Ошибка Begin даёт End0.
//!
//! AI удерживает свежие свойства и U либо запасную S. Смерть источника
//! не проверяется. Первый AI игрока списывает MP, вызывает OnChangeStates
//! и заново проверяет лук; поздний отказ не возвращает MP. Затем CAN,
//! visual0 и condition. Выпуск наступает по unsigned start+delay, публикует
//! visual1 и пополняет первое состояние IDCC либо создаёт новое.
//!
//! Владелец состояния сохраняет выбор первого слота, поздние LIMIT/AMOUNT
//! и Add до append. Здесь нет отдельного UpdateProperty или OnChangeStates
//! после пополнения. Выпуск всегда завершает End1, даже при отказе Begin
//! состояния. Общий End сбрасывает фазу, возвращает движение свежему U
//! либо S и вызывает State End с настоящим аргументом. Отдельный payload
//! не нужен: стадия общего kernel заменяет concrete active/condition.

use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME};
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::rangedweaponcast::{
    ArrowCastPathRule, RangedWeaponKind, check_ranged_weapon_cast, prepare_ranged_weapon_player, terminal,
};
use super::meteorarrowstate::add_meteor_arrows;
pub(crate) use super::meteorarrowstate::METEOR_ARROW_MASS_SKILL_ID;
use super::playercast::execute_registered_player_cast;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{resolve_skill_sufferer, resolve_state_move_shape};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return terminal(QueuedSkillExecutionState::Rejected); };
    let (region, identity) = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, region, identity)
        .map(|source| (source.shape().get_region_id(), source.shape().identity()))
        .or_else(|| resolve_skill_sufferer(game, skill.lifecycle()));
    let Some(source) = source else { return terminal(QueuedSkillExecutionState::Rejected); };
    let player = (source.1.object_type == 400).then_some(source.1.id);
    if stage == SkillStage::Begin {
        if !prepare_ranged_weapon_player(game, instance, player, &properties, RangedWeaponKind::Bow) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    game.update_registered_skill_visual(instance, 1);
    let _ = add_meteor_arrows(game, source, &properties, &mut || runtime.now_milliseconds());
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_meteor_arrow_mass<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::SelfCast,
        |game, instance, _player_id, runtime| original_user
            .is_some_and(|source| check_ranged_weapon_cast(game, instance, source, ArrowCastPathRule::None, RangedWeaponKind::Bow, runtime)),
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_ai,
    )
}
