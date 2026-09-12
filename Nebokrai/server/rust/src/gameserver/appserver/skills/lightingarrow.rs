//! Световая стрела CLightingArrow (0xCB).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/lightingarrow.cpp.
//! Attack Begin сохраняет исходного U и ранний отсчёт; Check проверяет reuse,
//! свежий путь и дальность. Чужой CMoveShape затем допускается без Move0;
//! игроку нужны лук категории 3 и ненулевая цена MP. MP0 — тихий отказ.
//! Неотрицательная signed DWORD-разность разрешает запрет движения исходному U.
//!
//! Каждый AI удерживает свежие свойства, U и координаты S через callbacks.
//! Отсутствующая S использует базовую точку, мёртвая S даёт End(0), смерть U
//! не проверяется. Первый AI игрока списывает MP и вызывает OnChangeStates
//! до повторной проверки лука; поздний отказ не возвращает MP. Затем CAN,
//! направление, visual0 и condition. Срок выпуска — absolute unsigned start+delay.
//!
//! Выпуск возвращает движение, заново строит путь заданной длины и очищает S
//! перед visual1/prepared. Пустой путь вызывает End(0), но исходный AI продолжает
//! Summon и End(1); этот необычный порядок сохранён. Путь остаётся опубликованным
//! до End и копируется в региональный снаряд. End сбрасывает фазу, очищает путь,
//! возвращает движение свежему U и завершает Attack с настоящим аргументом.
//!
//! Summon получает отдельную свежую таблицу, снимок MasterInfo и параметры
//! снаряда; его размещение, замена форм в клетке и wire принадлежат региональному
//! owner. Vec и поколенческий ключ заменяют указатели без второго хранилища.

use super::baseattack::SKILL_USAGE_USER_HIT_MODIFIER;
use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_ELEMENT_MODIFIER,
    SKILL_USAGE_SUMMONED_LIFETIME,
    SKILL_USAGE_SUMMONED_SPEED, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use super::lightingarrowphalanx::CLightingArrowPhalanx;
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{
    ArrowCastPathRule, RangedWeaponKind, check_ranged_weapon_cast,
    prepare_ranged_weapon_player, ranged_weapon_failure, terminal,
};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    resolve_skill_sufferer, resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::public::tools::get_line_direction;

pub(crate) const LIGHTING_ARROW_SKILL_ID: u32 = 0xcb;
const PLAYER_TYPE: i32 = 400;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LightingArrowExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    path: Vec<(i32, i32, u8)>,
}

impl LightingArrowExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), path: Vec::new() }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub(crate) fn clear_end_paths(&mut self) { self.path.clear(); }
}

fn summon<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    x: i32, y: i32, runtime: &mut Runtime,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region = user.shape().get_region_id();
    if game.find_region(region).is_none() { return; }
    let identity = user.shape().identity();
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let destination = skill.lifecycle().destination();
    let Some(skill) = game.registered_skill_mut(instance) else { return; };
    skill.lifecycle_mut().set_point_target(destination);
    let mut master = MasterInfo { master_type: identity.object_type, master_id: identity.id, ..Default::default() };
    if identity.object_type == PLAYER_TYPE {
        let Some(player) = game.find_player(identity.id) else { return; };
        master.master_team_id = player.team_id();
        master.master_guild_id = player.faction_id();
        master.master_union_id = player.union_id();
        let permissions = player.pk_permissions();
        master.permitted_to_kill_player = i32::from(permissions.player);
        master.permitted_to_kill_teammate = i32::from(permissions.teammate);
        master.permitted_to_kill_guild_member = i32::from(permissions.guild_member);
        master.permitted_to_kill_criminal = i32::from(permissions.criminal);
    }
    let _ = properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER);
    let speed = properties.query_property(SKILL_USAGE_SUMMONED_SPEED);
    let factor = properties.query_property(TARGET_DAMAGE_FACTOR);
    let hit = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let Some(skill) = game.registered_skill(instance) else { return; };
    let level = skill.level();
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let Some(state) = skill.player_state::<LightingArrowExecutionState>() else { return; };
    let path = state.path.clone();
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let phalanx = CLightingArrowPhalanx::new(id, master, started, lifetime, level, hit, factor, path, speed);
    let _ = game.spawn_lighting_arrow_phalanx(region, phalanx, x, y, started, runtime);
}

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
        .map(|source| (source.shape().get_region_id(), source.shape().identity()));
    let player = source.filter(|source| source.1.object_type == PLAYER_TYPE).map(|source| source.1.id);
    let destination = match resolve_skill_sufferer(game, skill.lifecycle()) {
        Some((region, identity)) => {
            if game.move_shape_health(region, identity) == Some(0) {
                ranged_weapon_failure(game, instance, player, 10, RangedWeaponKind::Bow);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            let Some(target) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
            let x = target.shape().get_tile_x().unwrap_or(i32::MIN);
            let y = target.shape().get_tile_y().unwrap_or(i32::MIN);
            (x, y)
        }
        None => skill.lifecycle().destination(),
    };
    let Some(source) = source else { return terminal(QueuedSkillExecutionState::Rejected); };
    if stage == SkillStage::Begin {
        if !prepare_ranged_weapon_player(game, instance, player, &properties, RangedWeaponKind::Bow) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.shape_mut().set_direction(direction); }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.set_moveable(true); }
    let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let path = game.skill_target_path_with_length(skill.lifecycle(), maximum);
    let empty = path.is_empty();
    let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<LightingArrowExecutionState>()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    state.path = path;
    if empty {
        game.update_registered_skill_visual(instance, 2);
        let _ = game.end_registered_instance(instance, 0, SkillTermination::Rejected, runtime);
    }
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) != 0 {
        let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
        let Some(state) = game.registered_skill(instance).and_then(|skill| skill.player_state::<LightingArrowExecutionState>()) else { return terminal(QueuedSkillExecutionState::Rejected); };
        if maximum.wrapping_add(1) < state.path.len() as u32 {
            ranged_weapon_failure(game, instance, player, 11, RangedWeaponKind::Bow);
            return terminal(QueuedSkillExecutionState::Rejected);
        }
    }
    let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let destination = skill.lifecycle().destination();
    skill.lifecycle_mut().set_point_target(destination);
    game.update_registered_skill_visual(instance, 1);
    if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().mark_prepared(); }
    if let Some(user) = resolve_state_move_shape(game, source.0, source.1) {
        let shape = user.shape();
        let x = shape.get_tile_x().unwrap_or(i32::MIN);
        let y = shape.get_tile_y().unwrap_or(i32::MIN);
        if let Ok(face) = CShape::get_direction_position(shape.get_direction(), ShapeAreaCoordinates { x, y }) {
            summon(game, instance, source, face.x, face.y, runtime);
        }
    }
    terminal(QueuedSkillExecutionState::Completed)
}

pub(crate) fn execute_player_lighting_arrow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::ArrowCast,
        |game, instance, _player_id, runtime| original_user
            .is_some_and(|source| check_ranged_weapon_cast(game, instance, source, ArrowCastPathRule::DistanceOnly, RangedWeaponKind::Bow, runtime)),
        |dispatch, started| LightingArrowExecutionState::begin(dispatch, started).into(),
        run_ai,
    )
}
