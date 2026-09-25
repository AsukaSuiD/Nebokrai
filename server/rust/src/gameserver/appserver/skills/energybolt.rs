//! Пошаговые снаряды CEnergyBolt (0x1A0), CSnakeBolt (0x1A5) и
//! CZombieClaw (0x1A2).
//! Источник: gameserver.exe/GameServer.pdb, исходные владельцы
//! appserver/skills/energybolt.cpp, snakebolt.cpp и zombieclaw.cpp.
//!
//! CPlayer и CMonster разделяют зарегистрированный жизненный цикл Attack: Begin
//! создаёт эффект до Check, первый AI повторно списывает MP и поворачивает U,
//! а выпуск ждёт беззнакового start+delay. Путь принудительно ограничен MAX,
//! сохраняется после выпуска и обрабатывается по одной клетке через
//! start+delay+flying*position. Область создаётся только после успешного Check
//! и сохраняется между End; уровни 1/2 дают 1×1, остальные (включая 0) — 3×3.
//! End очищает текущий путь и скаляры до Move1, но не сохранённую область.
//! У всех трёх позиция начинается с 0 в Begin; выпуск меняет флаги, но
//! не перезаписывает позицию, в том числе после вложенного вызова.
//! В Check нулевая MP-цена EnergyBolt не запрещает движение, а SnakeBolt и
//! ZombieClaw его запрещают; первый AI Player во всех трёх случаях выполняет
//! MP→OnChangeStates.
//!
//! Каждый контакт читает живой список CMoveShape области X→Y. Формула прямой
//! стихии, снимок/End SoulCollect, PK и непосредственный OnBeenAttacked
//! принадлежат directelementattack; здесь остаются порядок области, visual
//! target и жизненный цикл. Общий путь строится по сохранённой базе Begin и
//! живому S, поэтому point/object Begin не получают искусственного отказа.

use super::baseattack::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::directelementattack::apply_direct_element_attack;
use super::flash::cell_views;
use super::kernel::{SkillExecutionKernel, SkillStage, skill_is_restored};
use super::lightingarrowphalanx::ArrowTargetIdentity;
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{spend_cast_mana_without_text, terminal};
use super::skillbaseproperties::CSkillBaseProperties;
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
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
    ServerRegionOwner,
};
use crate::public::tools::get_line_direction;
pub(crate) use nebokrai_zone::skills::execution::{PathProjectileScope, PathProjectileProgress};

pub(crate) const ENERGY_BOLT_SKILL_ID: u32 = 0x1a0;

const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const USER_MP_LOSE: u32 = 2;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;
const BLOCK_UNFLY: u8 = 2;
const BLOCK_SHAPE: u8 = 3;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum PathProjectileProfile {
    EnergyBolt,
    SnakeBolt,
    ZombieClaw,
}

impl PathProjectileProfile {
    fn from_skill_id(skill_id: u32) -> Option<Self> {
        match skill_id {
            ENERGY_BOLT_SKILL_ID => Some(Self::EnergyBolt),
            super::snakebolt::SNAKE_BOLT_SKILL_ID => Some(Self::SnakeBolt),
            super::zombieclaw::ZOMBIE_CLAW_SKILL_ID => Some(Self::ZombieClaw),
            _ => None,
        }
    }

    const fn locks_free_mana(self) -> bool {
        !matches!(self, Self::EnergyBolt)
    }
}

fn resolved_user(game: &CGame, skill: &MoveShapeSkill) -> Option<(i32, ShapeIdentity)> {
    let (region, identity) = skill.lifecycle().user();
    let source = resolve_state_move_shape(game, region, identity)?.shape();
    Some((source.get_region_id(), source.identity()))
}

fn fail(game: &mut CGame, instance: RegisteredSkill, player: Option<i32>, mode: u32, text: &[u8]) {
    game.update_registered_skill_visual(instance, mode);
    if let Some(player) = player {
        game.send_skill_system_info(player, text);
    }
}

fn check_path_projectile_mana(
    game: &mut CGame,
    instance: RegisteredSkill,
    source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
    profile: PathProjectileProfile,
) -> bool {
    if source.1.object_type != PLAYER_TYPE {
        return true;
    }
    let player = source.1.id;
    let cost = properties.query_property(USER_MP_LOSE);
    if cost != 0 {
        let Some(mana) = game.find_player(player).map(|player| player.mana()) else {
            return false;
        };
        if (mana.wrapping_sub(cost) as i32) < 0 {
            game.update_registered_skill_visual(instance, 7);
            game.send_skill_system_info_with_unsigned(player, b"GS0288", cost);
            return false;
        }
    }
    if cost != 0 || profile.locks_free_mana() {
        let Some(source) = resolve_state_move_shape_mut(game, source.0, source.1) else {
            return false;
        };
        source.set_moveable(false);
    }
    true
}

fn check_path_projectile_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    original_user: Option<(i32, ShapeIdentity)>,
    begin_target: StateSkillBeginTarget,
    runtime: &mut Runtime,
) -> bool {
    let Some(original_user) = original_user else {
        return false;
    };
    let Some(source) = resolve_state_move_shape(game, original_user.0, original_user.1) else {
        return false;
    };
    let source = (source.shape().get_region_id(), source.shape().identity());
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);
    let target = game.registered_skill(instance)
        .and_then(|skill| begin_target.resolve(game, skill, false));
    if let Some(target) = target {
        let self_target = match (
            resolve_state_move_shape(game, source.0, source.1),
            resolve_state_move_shape(game, target.0, target.1),
        ) {
            (Some(source), Some(target)) => std::ptr::eq(source, target),
            _ => false,
        };
        if self_target {
            fail(game, instance, player, 10, b"GS0286");
            return false;
        }
    }
    let Some(skill) = game.registered_skill(instance) else {
        return false;
    };
    let Some(profile) = PathProjectileProfile::from_skill_id(skill.id()) else {
        return false;
    };
    let level = skill.level();
    let Some(properties) = game.skill_base_properties(skill.id(), level).cloned() else {
        return false;
    };
    if !skill_is_restored(
        skill.last_used_ms(),
        properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
        runtime.now_milliseconds(),
    ) {
        fail(game, instance, player, 13, b"GS0278");
        return false;
    }

    let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let Some(path) = game.registered_skill(instance).map(|skill| game.skill_target_path(skill.lifecycle())) else {
        return false;
    };
    if maximum != 0 && path.len() > maximum as usize {
        fail(game, instance, player, 11, b"GS0290");
        return false;
    }
    if !check_path_projectile_mana(game, instance, source, &properties, profile) {
        return false;
    }
    let Some(progress) = game.registered_skill_mut(instance)
        .and_then(MoveShapeSkill::path_projectile_progress_mut)
    else {
        return false;
    };
    progress.clear_end_paths();
    progress.ensure_scope(level);
    true
}

fn attack_scope<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    source: (i32, ShapeIdentity),
    region: i32,
    center_x: i32,
    center_y: i32,
    runtime: &mut Runtime,
) -> bool {
    if resolve_state_move_shape(game, source.0, source.1).is_none()
        || (center_x == 0 && center_y == 0)
    {
        return false;
    }
    let Some(radius) = game.registered_skill(instance)
        .and_then(MoveShapeSkill::path_projectile_progress)
        .and_then(|progress| progress.scope())
        .map(PathProjectileScope::radius)
    else {
        return false;
    };
    let source_key = ArrowTargetIdentity::new(source.0, source.1);
    let mut attacked = Vec::<ArrowTargetIdentity>::new();
    let mut did_attack = false;

    for offset_x in -radius..=radius {
        for offset_y in -radius..=radius {
            let cell_x = center_x.wrapping_add(offset_x);
            let cell_y = center_y.wrapping_add(offset_y);
            for view in cell_views(game, region, cell_x, cell_y) {
                let Some(target) = resolve_state_move_shape(game, region, view.identity) else {
                    continue;
                };
                let target = (target.shape().get_region_id(), target.shape().identity());
                let target_key = ArrowTargetIdentity::new(target.0, target.1);
                if target_key == source_key
                    || game.move_shape_health(target.0, target.1).is_none_or(|health| health == 0)
                    || attacked.contains(&target_key)
                {
                    continue;
                }
                // Цель эффекта записывается до проверок типа и CanAttack.
                if cell_x == center_x && cell_y != 0 {
                    if let Some(progress) = game.registered_skill_mut(instance)
                        .and_then(MoveShapeSkill::path_projectile_progress_mut)
                    {
                        progress.select_visual_target_if_empty(target.1);
                    }
                }
                if !matches!(target.1.object_type, PLAYER_TYPE | MONSTER_TYPE)
                    || !game.live_skill_target_attackable_between(source, target)
                {
                    continue;
                }
                apply_direct_element_attack(game, instance, source, target, runtime);
                // Добавление идёт после непосредственного контакта: End или
                // повторный callback не отменяют запись временного списка.
                attacked.push(target_key);
                did_attack = true;
            }
        }
    }
    did_attack
}

fn run_path_projectile_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    if PathProjectileProfile::from_skill_id(skill.id()).is_none() {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let Some(source) = resolved_user(game, skill) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.registered_skill(instance)
        .and_then(MoveShapeSkill::path_projectile_progress)
        .and_then(PathProjectileProgress::scope)
        .is_none()
    {
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let player = (source.1.object_type == PLAYER_TYPE).then_some(source.1.id);

    if stage == SkillStage::Begin {
        if !spend_cast_mana_without_text(game, instance, player, &properties) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if let Some(skill) = game.registered_skill_mut(instance) {
            skill.lifecycle_mut().set_available(
                properties.query_property(SKILL_USAGE_CAN_BE_BREAKED) != 0,
            );
        }
        let destination = game.registered_skill(instance)
            .and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()))
            .and_then(|target| resolve_state_move_shape(game, target.0, target.1))
            .map(|target| (
                target.shape().get_tile_x().unwrap_or(i32::MIN),
                target.shape().get_tile_y().unwrap_or(i32::MIN),
            ))
            .or_else(|| game.registered_skill(instance).map(|skill| skill.lifecycle().destination()));
        let Some(destination) = destination else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let direction = get_line_direction(
            user.shape().get_tile_x().unwrap_or(i32::MIN),
            user.shape().get_tile_y().unwrap_or(i32::MIN),
            destination.0,
            destination.1,
        );
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) {
            user.shape_mut().set_direction(direction);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }

    let Some(release_started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let fired = game.registered_skill(instance)
        .and_then(MoveShapeSkill::path_projectile_progress)
        .map(PathProjectileProgress::fired);
    let Some(fired) = fired else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };

    if !fired {
        if runtime.now_milliseconds() < release_started.wrapping_add(delay) {
            return terminal(QueuedSkillExecutionState::Pending);
        }
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) {
            user.set_moveable(true);
        }
        let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
        let Some(path) = game.registered_skill(instance)
            .map(|skill| game.skill_target_path_with_length(skill.lifecycle(), maximum))
        else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if maximum != 0 && path.len() > maximum.wrapping_add(1) as usize {
            game.update_registered_skill_visual(instance, 11);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let flying_unit_ms = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME);
        let Some(progress) = game.registered_skill_mut(instance)
            .and_then(MoveShapeSkill::path_projectile_progress_mut)
        else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        progress.prepare_flight(path, flying_unit_ms);
        game.update_registered_skill_visual(instance, 1);
        let Some(skill) = game.registered_skill_mut(instance) else {
            return terminal(QueuedSkillExecutionState::Pending);
        };
        let Some(progress) = skill.path_projectile_progress_mut() else {
            return terminal(QueuedSkillExecutionState::Pending);
        };
        progress.start_flight();
        skill.lifecycle_mut().mark_prepared();
        let _ = skill.advance_execution(SkillStage::Check, SkillStage::Calculate);
    }

    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some((current_position, path_len, cell)) = game.registered_skill(instance)
        .and_then(MoveShapeSkill::path_projectile_progress)
        .map(|progress| (
            progress.current_position(),
            progress.path_len(),
            progress.current_cell(),
        ))
    else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let flying_unit_ms = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME);
    let due = started
        .wrapping_add(delay)
        .wrapping_add(flying_unit_ms.wrapping_mul(current_position as u32));
    if runtime.now_milliseconds() < due {
        return terminal(QueuedSkillExecutionState::Pending);
    }

    // После срока оригинал читает текущий server region; его исчезновение не
    // превращается в End и не подменяется прежним регионом Begin.
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let user = user.shape();
    if !user.is_assigned_to_server_region() {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let region = user.get_region_id();
    if game.find_region(region).is_none() {
        return terminal(QueuedSkillExecutionState::Pending);
    }

    if current_position >= path_len {
        game.update_registered_skill_visual(instance, 3);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Calculate, SkillStage::Attack);
            let _ = skill.advance_execution(SkillStage::Attack, SkillStage::Apply);
        }
        return terminal(QueuedSkillExecutionState::Completed);
    }
    let Some((cell_x, cell_y)) = cell else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(progress) = game.registered_skill_mut(instance)
        .and_then(MoveShapeSkill::path_projectile_progress_mut)
    else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    progress.set_end_position(cell_x, cell_y);

    let Some(block) = game.find_region(region)
        .map(|owner| owner.base().skill_cell_block(cell_x, cell_y))
    else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    match block {
        BLOCK_SHAPE => {
            if attack_scope(game, instance, source, region, cell_x, cell_y, runtime) {
                game.update_registered_skill_visual(instance, 3);
                let Some(progress) = game.registered_skill_mut(instance)
                    .and_then(MoveShapeSkill::path_projectile_progress_mut)
                else {
                    return terminal(QueuedSkillExecutionState::Pending);
                };
                progress.finish_after_collision();
                return terminal(QueuedSkillExecutionState::Pending);
            }
        }
        BLOCK_UNFLY => {
            game.update_registered_skill_visual(instance, 3);
            let Some(progress) = game.registered_skill_mut(instance)
                .and_then(MoveShapeSkill::path_projectile_progress_mut)
            else {
                return terminal(QueuedSkillExecutionState::Pending);
            };
            progress.finish_after_unfly();
        }
        _ => {}
    }
    let Some(skill) = game.registered_skill_mut(instance) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let Some(progress) = skill.path_projectile_progress_mut() else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    progress.advance();
    let _ = skill.advance_execution(SkillStage::Calculate, SkillStage::Attack);
    terminal(QueuedSkillExecutionState::Pending)
}

pub(crate) fn execute_player_path_projectile<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let begin_target = match dispatch {
        PlayerSkillDispatch::Point { .. } => StateSkillBeginTarget::Resolved,
        _ => StateSkillBeginTarget::Object(original_user.and_then(|(region, _)| {
            dispatch.object_target().and_then(|target| game.player_skill_begin_object(region, target))
        })),
    };
    execute_registered_player_cast(
        game,
        player_id,
        instance,
        dispatch,
        runtime,
        SkillVisualEffectKind::PathProjectile,
        |game, instance, _, runtime| {
            check_path_projectile_cast(game, instance, original_user, begin_target, runtime)
        },
        |dispatch, started| SkillExecutionKernel::begin(dispatch, started).into(),
        run_path_projectile_ai,
    )
}

struct PathProjectileSkill<const ID: u32>;

impl<const ID: u32> RegisteredStateSkill for PathProjectileSkill<ID> {
    const ID: u32 = ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::PathProjectile;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;

    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame,
        instance: RegisteredSkill,
        target: StateSkillBeginTarget,
        runtime: &mut Runtime,
    ) -> bool {
        let source = game.registered_skill(instance)
            .and_then(|skill| resolved_user(game, skill));
        check_path_projectile_cast(game, instance, source, target, runtime)
    }

    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame,
        instance: RegisteredSkill,
        runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = run_path_projectile_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => end_state_skill(game, instance, 0, runtime),
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse => {
                end_state_skill(game, instance, 1, runtime)
            }
            _ => outcome,
        }
    }
}

pub(crate) fn execute_owned_path_projectile<const ID: u32, Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target: ShapeIdentity,
    skill_level: u16,
    runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<PathProjectileSkill<ID>, Runtime>(
        game, owner, monster_id, target, skill_level, runtime,
    )
}

pub(crate) fn execute_owned_monster_energy_bolt<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target: ShapeIdentity,
    skill_level: u16,
    runtime: &mut Runtime,
) -> bool {
    execute_owned_path_projectile::<ENERGY_BOLT_SKILL_ID, Runtime>(
        game, owner, monster_id, target, skill_level, runtime,
    )
}
