//! Прицельный удар молнии CLightning (0x133).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/lightning.cpp.
//! Player и Monster исполняют один зарегистрированный cast. Общая база
//! хранит U/S, точку, CAN, фазу и время; отдельно нужен только attacking.
//! Check использует исходного U: reuse→свежий путь→MAX→MP, без Move0.
//! Первый AI списывает MP и публикует состояния только игроку, затем
//! записывает CAN, поворачивает захваченного U и вызывает visual0.
//!
//! Выпуск и контакт имеют два независимых unsigned start+DELAY срока.
//! При ненулевых type/id S выпуск заново проверяет цель на смерть,
//! сохраняет X/Y и очищает её identity. Путь, visual1 и финальный контакт
//! разрешают уже новую базу: координатная цель может найти другую фигуру.
//! Visual1 предшествует независимой записи attacking; callback End не
//! прерывает оставшуюся часть этого AI искусственной проверкой фазы.
//! Контакт требует свежую живую S и её допуск к захваченному U.
//! Общий directelementattack сохраняет PK→свежий Calculate→raw receipt,
//! без оружейного множителя и RP. End сбрасывает фазу и attacking,
//! разрешает U без Move и передаёт фактический аргумент в Attack End.
//! Visual использует свежего U; выпуск передаёт свежую S либо 0/0 и точку.
//! Поле missile-time только обнулялось и не читалось этим владельцем.

use super::basemagic::{SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME};
use super::directelementattack::apply_direct_element_attack;
use super::kernel::{SkillStage, skill_is_restored};
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{
    CastPathBlock, check_cast_mana_without_movement, check_skill_path, spend_cast_mana, terminal,
};
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, end_state_skill, execute_owned_state_skill,
};
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, ServerRegionOwner,
};
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::GameMessageDomainOps;
use crate::public::tools::get_line_direction;
pub(crate) use nebokrai_zone::skills::execution::{LightningProgress, LightningExecutionState};

pub(crate) const LIGHTNING_SKILL_ID: u32 = 0x133;

fn check_lightning_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill,
    original_user: Option<(i32, ShapeIdentity)>, runtime: &mut Runtime,
) -> bool {
    let Some((region, identity)) = original_user else { return false; };
    let Some(source) = resolve_state_move_shape(game, region, identity) else { return false; };
    let source = (source.shape().get_region_id(), source.shape().identity());
    let player = (source.1.object_type == 400).then_some(source.1.id);
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        game.update_registered_skill_visual(instance, 13);
        if let Some(player) = player { game.send_skill_system_info(player, b"GS0278"); }
        return false;
    }
    let path = game.skill_target_path(skill.lifecycle());
    if !check_skill_path(game, instance, &properties, &path, player, CastPathBlock::Ignore) { return false; }
    check_cast_mana_without_movement(game, instance, source, &properties)
}

fn run_lightning_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let (region, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region, identity) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let user = (source.shape().get_region_id(), source.shape().identity());
    let player = (user.1.object_type == 400).then_some(user.1.id);
    if stage == SkillStage::Begin {
        if !spend_cast_mana(game, instance, player, &properties) { return terminal(QueuedSkillExecutionState::Rejected); }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_available(can_break != 0); }
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let destination = resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
            .map(|target| (
                target.shape().get_tile_x().unwrap_or(i32::MIN),
                target.shape().get_tile_y().unwrap_or(i32::MIN),
            ))
            .unwrap_or_else(|| skill.lifecycle().destination());
        let Some(source) = resolve_state_move_shape(game, user.0, user.1) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let source_y = source.shape().get_tile_y().unwrap_or(i32::MIN);
        let source_x = source.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(source_x, source_y, destination.0, destination.1);
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) {
            source.shape_mut().set_direction(direction);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(attacking) = game.registered_skill(instance)
        .and_then(MoveShapeSkill::lightning_progress).map(|progress| progress.attacking)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !attacking {
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if runtime.now_milliseconds() >= started.wrapping_add(delay) {
            let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let stored_target = skill.lifecycle().sufferer().1;
            if stored_target.object_type != 0 && stored_target.id != 0 {
                let target = resolve_skill_sufferer(game, skill.lifecycle());
                let Some((region, identity)) = target else {
                    game.update_registered_skill_visual(instance, 10);
                    return terminal(QueuedSkillExecutionState::Rejected);
                };
                if game.move_shape_health(region, identity).is_none_or(|health| health == 0) {
                    game.update_registered_skill_visual(instance, 10);
                    return terminal(QueuedSkillExecutionState::Rejected);
                }
                let Some(target) = resolve_state_move_shape(game, region, identity) else {
                    return terminal(QueuedSkillExecutionState::Rejected);
                };
                let destination = (
                    target.shape().get_tile_x().unwrap_or(i32::MIN),
                    target.shape().get_tile_y().unwrap_or(i32::MIN),
                );
                if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_point_target(destination); }
            }
            let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let path = game.skill_target_path(skill.lifecycle());
            if !check_skill_path(game, instance, &properties, &path, player, CastPathBlock::Ignore) {
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            game.update_registered_skill_visual(instance, 1);
            if let Some(progress) = game.registered_skill_mut(instance).and_then(MoveShapeSkill::lightning_progress_mut) {
                progress.attacking = true;
            }
        }
        if game.registered_skill(instance).and_then(MoveShapeSkill::lightning_progress)
            .is_none_or(|progress| !progress.attacking)
        { return terminal(QueuedSkillExecutionState::Pending); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    let target = game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()));
    let Some((region, identity)) = target else {
        game.update_registered_skill_visual(instance, 3);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let target = resolve_state_move_shape(game, region, identity)
        .map(|target| (target.shape().get_region_id(), target.shape().identity()));
    let Some(target) = target else {
        game.update_registered_skill_visual(instance, 3);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.move_shape_health(target.0, target.1).is_none_or(|health| health == 0)
        || !game.live_skill_target_attackable_between(user, target)
    {
        game.update_registered_skill_visual(instance, 3);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    apply_direct_element_attack(game, instance, user, target, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_lightning<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::Lightning,
        |game, instance, _, runtime| check_lightning_cast(game, instance, original_user, runtime),
        |dispatch, started| LightningExecutionState::begin(dispatch, started).into(), run_lightning_ai,
    )
}

pub(crate) fn publish_lightning_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if skill.id() != LIGHTNING_SKILL_ID || skill.visual_effect().is_none_or(|effect|
        effect.kind() != SkillVisualEffectKind::Lightning || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    if matches!(mode, 2 | 7 | 10 | 11 | 13 | 15) {
        if source.identity().object_type == 400 {
            let mut message = CMessage::new(0x000b_fe01);
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }
    if !matches!(mode, 0 | 1 | 3) { return; }
    let target = if mode == 1 {
        resolve_skill_sufferer(game, skill.lifecycle())
            .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
            .map(|target| target.shape())
    } else { None };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(match mode { 0 => 1, 1 => 2, _ => 3 });
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if mode == 1 {
        let (identity, destination) = target.map_or(
            (None, skill.lifecycle().destination()),
            |target| (
                Some(target.identity()),
                (target.get_tile_x().unwrap_or(i32::MIN), target.get_tile_y().unwrap_or(i32::MIN)),
            ),
        );
        message.add_long(identity.map_or(0, |identity| identity.object_type));
        message.add_long(identity.map_or(0, |identity| identity.id));
        message.add_long(destination.0);
        message.add_long(destination.1);
    } else { message.add_long(source.get_direction()); }
    if source.is_assigned_to_server_region()
        && let Some(region) = game.find_region(source.get_region_id())
    {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}

struct LightningSkill;

impl RegisteredStateSkill for LightningSkill {
    const ID: u32 = LIGHTNING_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::Lightning;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;

    fn prepare_monster(skill: &mut MoveShapeSkill) {
        skill.set_monster_progress(LightningProgress::default());
    }

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, _target: StateSkillBeginTarget, runtime: &mut Runtime,
    ) -> bool {
        // Базовый Monster Begin и создание visual не вызывают мутаций U до Check.
        let original_user = game.registered_skill(instance)
            .map(|skill| skill.lifecycle().user())
            .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
            .map(|source| (source.shape().get_region_id(), source.shape().identity()));
        check_lightning_cast(game, instance, original_user, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = run_lightning_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => end_state_skill(game, instance, 0, runtime),
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse =>
                end_state_skill(game, instance, 1, runtime),
            _ => outcome,
        }
    }
}

pub(crate) fn execute_owned_monster_lightning<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<LightningSkill, Runtime>(
        game, owner, monster_id, target, skill_level, runtime,
    )
}
