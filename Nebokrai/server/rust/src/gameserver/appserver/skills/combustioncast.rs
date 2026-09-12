//! Зарегистрированный цикл горючей смеси Kerosene и воспламенения Ignition.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/kerosene.cpp
//! и ignition.cpp. Begin сохраняет исходные U/S до базовых callbacks;
//! Check использует их, но строит свежий базовый путь. NULL/self цели,
//! reuse, строгий MAX (включая ноль), BLOCK2, арбалет и MP проверяются
//! именно в этом порядке. MP0 допускает Move0 без чтения маны. Native
//! unchecked Player-доступ безопасно отклоняется только у границы экипировки.
//! Отказ Check публикует дополнительный visual2 перед End(0).
//!
//! Каждый AI держит одну таблицу и полные U/S через callbacks. Проверяется
//! смерть S, но не повторное self. Первый AI: MP→OnChangeStates→арбалет,
//! затем CAN→S.Y/X→U.Y/X→направление→visual0→condition. Непользовательский
//! U пропускает только расход и оружие. После unsigned start+delay следуют
//! Move1, свежий путь, строгий MAX и именованное препятствие, затем visual1.
//! Kerosene применяет состояние к захваченным U/S даже без visual-объекта.
//! Ignition без visual остаётся активным; с ним заново получает S после
//! visual1 и выполняет один контакт, затем End(1), даже при NULL свежем S.
//!
//! Общий playercast/kernel владеет единственным исполнением, фазой и часами;
//! специфические применение, wire-пакеты и End-политики остаются у owners.
//! Дополнительный payload, собственная очередь или копия ресурсов не нужны.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::ignition::{IGNITION_SKILL_ID, apply_ignition_attack};
use super::kerosene::{KEROSENE_SKILL_ID, apply_kerosene};
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{
    CastDistanceLimit, CastManaRule, CastPathBlock, RangedWeaponKind,
    check_ranged_weapon_and_mana, check_skill_path_with_limit,
    prepare_ranged_weapon_player, ranged_weapon_failure, terminal,
};
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::public::tools::get_line_direction;

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, original_user: Option<(i32, ShapeIdentity)>,
    original_target: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let Some(source) = original_user.and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
    else { return false; };
    let target = original_target.and_then(|(region, identity)| resolve_state_move_shape(game, region, identity));
    let invalid_target = target.is_none_or(|target| std::ptr::eq(source, target));
    let source = (source.shape().get_region_id(), source.shape().identity());
    let target = target.map(|target| (target.shape().get_region_id(), target.shape().identity()));
    let player = (source.1.object_type == 400).then_some(source.1.id);
    if invalid_target {
        ranged_weapon_failure(game, instance, player, 10, RangedWeaponKind::Crossbow);
        return false;
    }
    let Some(target) = target else { return false; };
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        ranged_weapon_failure(game, instance, player, 13, RangedWeaponKind::Crossbow);
        return false;
    }
    let path = game.skill_target_path(skill.lifecycle());
    if !check_skill_path_with_limit(game, instance, &properties, &path, player,
        CastDistanceLimit::IncludingZero, CastPathBlock::Named { target, message: b"GS0291" })
    { return false; }
    if player.is_none() { return false; }
    check_ranged_weapon_and_mana(game, instance, source, &properties,
        RangedWeaponKind::Crossbow, CastManaRule::AllowFree)
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (region, identity) = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, region, identity)
        .map(|source| (source.shape().get_region_id(), source.shape().identity()));
    let target = resolve_skill_sufferer(game, skill.lifecycle())
        .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
        .map(|target| (target.shape().get_region_id(), target.shape().identity()));
    let (Some(source), Some(target)) = (source, target) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let player = (source.1.object_type == 400).then_some(source.1.id);
    if game.move_shape_health(target.0, target.1) == Some(0) {
        game.update_registered_skill_visual(instance, 10);
        if let Some(player) = player { game.send_skill_system_info(player, b"GS0285"); }
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if stage == SkillStage::Begin {
        if !prepare_ranged_weapon_player(game, instance, player, &properties, RangedWeaponKind::Crossbow) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_available(can_break != 0); }
        let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let target_y = sufferer.shape().get_tile_y().unwrap_or(i32::MIN);
        let target_x = sufferer.shape().get_tile_x().unwrap_or(i32::MIN);
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) {
            user.shape_mut().set_direction(direction);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.set_moveable(true); }
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let path = game.skill_target_path(skill.lifecycle());
    if !check_skill_path_with_limit(game, instance, &properties, &path, player,
        CastDistanceLimit::IncludingZero, CastPathBlock::Named { target, message: b"GS0307" })
    { return terminal(QueuedSkillExecutionState::Rejected); }
    if skill_id == IGNITION_SKILL_ID
        && game.registered_skill(instance).is_none_or(|skill| skill.visual_effect().is_none())
    { return terminal(QueuedSkillExecutionState::Pending); }
    game.update_registered_skill_visual(instance, 1);
    if skill_id == KEROSENE_SKILL_ID { apply_kerosene(game, source, target, &properties, runtime); }
    else {
        let target = game.registered_skill(instance)
            .and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()));
        apply_ignition_attack(game, instance, source, target, runtime);
    }
    terminal(QueuedSkillExecutionState::Completed)
}

pub(super) fn execute_player_combustion<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let original_target = if game.registered_skill(instance).is_some_and(|skill| skill.player_dispatch().is_none()) {
        dispatch.object_target().and_then(|target|
            original_user.and_then(|source| game.player_skill_begin_object(source.0, target)))
    } else { None };
    let visual = if dispatch.skill_id() == KEROSENE_SKILL_ID { SkillVisualEffectKind::Kerosene }
        else { SkillVisualEffectKind::Ignition };
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, visual,
        |game, instance, _, runtime| {
            let target = if matches!(dispatch, PlayerSkillDispatch::Point { .. }) {
                game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()))
            } else { original_target };
            let accepted = check_cast(game, instance, original_user, target, runtime);
            if !accepted { game.update_registered_skill_visual(instance, 2); }
            accepted
        },
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(), run_ai,
    )
}
