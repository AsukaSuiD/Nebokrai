//! Базовая стрельба игрока и монстра CArchery.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/archery.cpp.
//! Один зарегистрированный Summon-навык владеет базой исполнения и временем
//! полёта, сохраняемым между применениями. Begin создаёт visual до Check;
//! Check не списывает MP и не блокирует движение. Отказ вызывает End0.
//!
//! Первый AI: смерть S→указательная самоцель→CAN→повторная смерть,
//! затем поворот→visual0→Move0→condition. Самоцель даёт два visual10.
//! Выпуск использует unsigned start+delay; Move1 предшествует двум свежим
//! GetS и проверке смерти. Смерть на выпуске даёт visual10→текст→visual10.
//! Расстояние и Summon используют захваченные в начале AI полные U/S;
//! visual разрешает участников заново. Нет повторного допуска или RP.
//!
//! Summon строит свежий путь длиной RealDistance с footprint цели. Непустой
//! путь очищает S до BLOCK2; отказ не откатывает очистку. MasterInfo,
//! таблица, часы конструктора и Add имеют собственный порядок. MIN/MAX/EM
//! конструктора не участвуют в дальнейшем расчёте снаряда. End сбрасывает
//! фазу до Move1 и SummonEnd, но не обнуляет время полёта. Очередь остаётся
//! у CPlayerAI/CMonsterAI; SlotMap и Vec заменяют указатели и временный STL.
//! Снаряд живёт независимо от cast, а его формула принадлежит phalanx.

use super::archerycast::{archery_attack_path, check_archery_cast};
use super::archeryphalanx::CArcheryPhalanx;
use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_ELEMENT_MODIFIER,
    SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK, SKILL_USAGE_SUMMONED_LIFETIME,
    SKILL_USAGE_SUMMONED_SPEED,
};
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::terminal;
use super::stateskill::{
    RegisteredStateSkill, StateSkillBeginTarget, end_state_skill, execute_owned_state_skill,
};
use super::weaponattack::source_master;
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
use crate::public::tools::get_line_direction;

pub(crate) const ARCHERY_SKILL_ID: u32 = 2;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct ArcheryProgress {
    attack_time_ms: i32,
}
impl ArcheryProgress {
    pub(crate) const fn attack_time_ms(&self) -> i32 { self.attack_time_ms }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ArcheryExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
}
impl ArcheryExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started) }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
}

fn distance(game: &CGame, source: (i32, ShapeIdentity), target: (i32, ShapeIdentity)) -> Option<i32> {
    let source = game.skill_shape_view(source)?;
    let target = game.skill_shape_view(target)?;
    Some(source.real_distance(Some(target)))
}

fn summon<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    if resolve_state_move_shape(game, source.0, source.1).is_none()
        || resolve_state_move_shape(game, target.0, target.1).is_none()
    { return; }
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let Some(length) = distance(game, source, target) else { return; };
    let path = archery_attack_path(game, instance, length as u32);
    if path.is_empty() { return; }
    let Some(skill) = game.registered_skill_mut(instance) else { return; };
    let destination = skill.lifecycle().destination();
    skill.lifecycle_mut().set_point_target(destination);
    if path.iter().any(|cell| cell.2 == 2) { return; }
    let Some(mut master) = source_master(game, source) else { return; };
    master.master_country_id = 0;
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(flight) = skill.archery_progress().map(ArcheryProgress::attack_time_ms) else { return; };
    let Some(target_shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let target = target_shape.shape().identity();
    let _ = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    let _ = properties.query_property(SKILL_USAGE_MAX_ATTACK);
    let _ = properties.query_property(SKILL_USAGE_MIN_ATTACK);
    let level = skill.level();
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let phalanx = CArcheryPhalanx::new(id, master, started, lifetime, level, flight as u32, target);
    let (x, y, _) = path[0];
    let _ = game.spawn_archery_phalanx(source, phalanx, x, y, started, runtime);
}

fn target_failure(game: &mut CGame, instance: RegisteredSkill, player: Option<i32>, text: &[u8]) {
    game.update_registered_skill_visual(instance, 10);
    if let Some(player) = player { game.send_skill_system_info(player, text); }
}

fn run_ai<Runtime: GameMainLoopRuntime>(
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
    let source = resolve_state_move_shape(game, region, identity);
    let target = resolve_skill_sufferer(game, skill.lifecycle())
        .and_then(|(region, identity)| resolve_state_move_shape(game, region, identity));
    let (Some(source), Some(target)) = (source, target) else {
        game.update_registered_skill_visual(instance, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let is_self = std::ptr::eq(source, target);
    let source = (source.shape().get_region_id(), source.shape().identity());
    let target = (target.shape().get_region_id(), target.shape().identity());
    let player = (source.1.object_type == 400).then_some(source.1.id);
    if stage == SkillStage::Begin {
        if game.move_shape_health(target.0, target.1) == Some(0) {
            target_failure(game, instance, player, b"GS0285");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        if is_self {
            game.update_registered_skill_visual(instance, 10);
            target_failure(game, instance, player, b"GS0286");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_available(can_break != 0); }
        if game.move_shape_health(target.0, target.1) == Some(0) {
            target_failure(game, instance, player, b"GS0285");
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let Some(sufferer) = resolve_state_move_shape(game, target.0, target.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let target_y = sufferer.shape().get_tile_y().unwrap_or(i32::MIN);
        let target_x = sufferer.shape().get_tile_x().unwrap_or(i32::MIN);
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.shape_mut().set_direction(direction); }
        game.update_registered_skill_visual(instance, 0);
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.set_moveable(false); }
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.set_moveable(true); }
    if game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle())).is_none() {
        game.update_registered_skill_visual(instance, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let fresh_target = game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()));
    let Some(fresh_target) = fresh_target else {
        game.update_registered_skill_visual(instance, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if game.move_shape_health(fresh_target.0, fresh_target.1) == Some(0) {
        target_failure(game, instance, player, b"GS0285");
        game.update_registered_skill_visual(instance, 10);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let Some(length) = distance(game, source, target) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let speed = properties.query_property(SKILL_USAGE_SUMMONED_SPEED);
    if let Some(progress) = game.registered_skill_mut(instance).and_then(MoveShapeSkill::archery_progress_mut) {
        progress.attack_time_ms = length.wrapping_mul(speed as i32);
    }
    game.update_registered_skill_visual(instance, 1);
    summon(game, instance, source, target, runtime);
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_archery<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id).map(|player| (player.shape().get_region_id(), player.shape().identity()));
    let original_target = if game.registered_skill(instance).is_some_and(|skill| skill.player_dispatch().is_none()) {
        original_user.and_then(|(region, _)| dispatch.object_target()
            .and_then(|target| game.player_skill_begin_object(region, target)))
    } else { None };
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::Archery,
        |game, instance, _, runtime| {
            let target = if matches!(dispatch, PlayerSkillDispatch::Point { .. }) {
                game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()))
            } else { original_target };
            check_archery_cast(game, instance, original_user, target, runtime)
        },
        |dispatch, started| ArcheryExecutionState::begin(dispatch, started).into(), run_ai,
    )
}

struct ArcherySkill;
impl RegisteredStateSkill for ArcherySkill {
    const ID: u32 = ARCHERY_SKILL_ID;
    const VISUAL: SkillVisualEffectKind = SkillVisualEffectKind::Archery;
    const BEGIN_FAILURE_VISUAL: Option<u32> = None;
    fn check_cast<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, begin_target: StateSkillBeginTarget, runtime: &mut Runtime,
    ) -> bool {
        let Some(skill) = game.registered_skill(instance) else { return false; };
        let (region, identity) = skill.lifecycle().user();
        let user = resolve_state_move_shape(game, region, identity)
            .map(|shape| (shape.shape().get_region_id(), shape.shape().identity()));
        let target = begin_target.resolve(game, skill, false);
        check_archery_cast(game, instance, user, target, runtime)
    }
    fn run_ai<Runtime: GameMainLoopRuntime>(
        game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
    ) -> QueuedSkillExecutionOutcome {
        let outcome = run_ai(game, instance, runtime);
        match outcome.state {
            QueuedSkillExecutionState::Rejected => end_state_skill(game, instance, 0, runtime),
            QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse =>
                end_state_skill(game, instance, 1, runtime),
            _ => outcome,
        }
    }
}

pub(crate) fn execute_owned_monster_archery<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    execute_owned_state_skill::<ArcherySkill, Runtime>(game, owner, monster_id, target, skill_level, runtime)
}
