//! Поклеточный арбалетный BloodRose (0xD0).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/bloodrose.cpp.
//! Общий зарегистрированный Begin сохраняет исходные U/S для объектного Check;
//! точечный Check заново разрешает S после создания visual. Самонацеливание
//! отклоняется до свойств/reuse, затем проверяются дальность без BLOCK_UNFLY,
//! арбалет категории 4 и ненулевая MP-цена. Первый AI списывает MP, публикует
//! OnChangeStates, повторяет проверку оружия и задаёт CAN/направление.
//!
//! Выпуск после Move1 строит свежий путь, сохраняет время до первой непролётной
//! клетки и переводит базовую цель в endpoint. Attacking/prepared записываются
//! после visual1; индекс остаётся нулевым. Все сроки абсолютные unsigned,
//! каждый AI читает свойства задержки заново и обрабатывает не более клетки.
//! Отсутствие текущего региона U оставляет полёт ожидающим, включая его конец.
//!
//! Все три исходные маски и fallback — полная область 3×3; общий арбалетный
//! owner обходит её X→Y, выбирает visual-цель до дедупликации и хранит список
//! попаданий в этом же живом экземпляре. Повторная допустимая цель подавляет
//! лишь урон, но всё равно завершает клетку. Удары не проверяют IsDied и не
//! начисляют RP. После visual3 индекс получает свежую длину пути и увеличивается;
//! отбрасывание стрелка выполняется только на следующем подходящем AI.
//!
//! Recoil читает число шагов после каждого свободного блока, а скорость —
//! только при фактическом смещении; общий ForceMove сохраняет packet/AI-tail.
//! End сбрасывает фазу, счётчики и visual-цель, освобождает путь перед списком
//! ударов, затем свежий U Move1 и общий Attack End. Vec и единый kernel заменяют
//! native контейнеры; постоянная одинаковая маска не требует отдельного объекта.

use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::crossbowattack::run_blood_rose_scope;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::lightingarrowphalanx::ArrowTargetIdentity;
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{
    ArrowCastPathRule, RangedWeaponKind, check_ranged_weapon_cast,
    prepare_ranged_weapon_player, ranged_weapon_failure, terminal,
};
use super::skillbaseproperties::CSkillBaseProperties;
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

pub(crate) const BLOOD_ROSE_SKILL_ID: u32 = 0xD0;
const PLAYER_TYPE: i32 = 400;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const MISSILE_FLYING_TIME: u32 = 10_008;
const TARGET_BACK_STEP: u32 = 30_002;
const TARGET_MOVE_SPEED: u32 = 30_003;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BloodRoseExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    attacking_started: bool,
    missile_flying_time: u32,
    current_position: u32,
    end_tile: (i32, i32),
    visual_target: (i32, i32),
    path: Vec<(i32, i32, u8)>,
    attacked: Vec<ArrowTargetIdentity>,
}

impl BloodRoseExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started),
            attacking_started: false,
            missile_flying_time: 0,
            current_position: 0,
            end_tile: (0, 0),
            visual_target: (0, 0),
            path: Vec::new(),
            attacked: Vec::new(),
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub(crate) const fn missile_flying_time(&self) -> u32 { self.missile_flying_time }
    pub(crate) const fn end_tile(&self) -> (i32, i32) { self.end_tile }
    pub(crate) const fn visual_target(&self) -> (i32, i32) { self.visual_target }

    pub(super) fn mark_target_attacked(&mut self, target: (i32, ShapeIdentity)) -> bool {
        let key = ArrowTargetIdentity::new(target.0, target.1);
        if self.attacked.contains(&key) { return false; }
        self.attacked.push(key);
        true
    }

    pub(super) fn select_visual_target_if_empty(&mut self, identity: ShapeIdentity) {
        if self.visual_target == (0, 0) {
            self.visual_target = (identity.object_type, identity.id);
        }
    }

    pub(crate) fn clear_end_paths(&mut self) {
        self.attacking_started = false;
        self.missile_flying_time = 0;
        self.current_position = 0;
        self.end_tile = (0, 0);
        self.visual_target = (0, 0);
        drop(std::mem::take(&mut self.path));
        drop(std::mem::take(&mut self.attacked));
    }
}

fn knock_back_owner(
    game: &mut CGame, source: (i32, ShapeIdentity), region: i32,
    properties: &CSkillBaseProperties,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let direction = user.shape().get_direction();
    let reverse = direction.wrapping_add(4);
    let reverse = if reverse > 7 { direction.wrapping_sub(4) } else { reverse };
    let mut position = ShapeAreaCoordinates {
        x: user.shape().get_tile_x().unwrap_or(i32::MIN),
        y: user.shape().get_tile_y().unwrap_or(i32::MIN),
    };
    let mut count = 0_u32;
    let mut maximum = properties.query_property(TARGET_BACK_STEP);
    while count < maximum {
        let Ok(next) = CShape::get_direction_position(reverse, position) else { break; };
        if game.find_region(region).is_none_or(|owner| owner.base().skill_cell_block(next.x, next.y) != 0) {
            break;
        }
        count = count.wrapping_add(1);
        maximum = properties.query_property(TARGET_BACK_STEP);
        position = next;
    }
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if position.x != user.shape().get_tile_x().unwrap_or(i32::MIN)
        || position.y != user.shape().get_tile_y().unwrap_or(i32::MIN)
    {
        let duration = properties.query_property(TARGET_MOVE_SPEED).wrapping_mul(count);
        let _ = game.force_move_skill_target(region, source.1, position.x, position.y, duration);
    }
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
    let Some(source) = resolve_state_move_shape(game, region, identity) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    let source = (source.shape().get_region_id(), source.shape().identity());
    if stage == SkillStage::Begin {
        if !prepare_ranged_weapon_player(
            game, instance, (source.1.object_type == PLAYER_TYPE).then_some(source.1.id),
            &properties, RangedWeaponKind::Crossbow,
        ) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let destination = match resolve_skill_sufferer(game, skill.lifecycle()) {
            Some((region, identity)) => {
                let Some(target) = resolve_state_move_shape(game, region, identity) else {
                    return terminal(QueuedSkillExecutionState::Rejected);
                };
                (target.shape().get_tile_x().unwrap_or(i32::MIN), target.shape().get_tile_y().unwrap_or(i32::MIN))
            }
            None => skill.lifecycle().destination(),
        };
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) {
            user.shape_mut().set_direction(direction);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(attacking) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<BloodRoseExecutionState>())
        .map(|state| state.attacking_started)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !attacking {
        let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        if runtime.now_milliseconds() < started.wrapping_add(delay) {
            return terminal(QueuedSkillExecutionState::Pending);
        }
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.set_moveable(true); }
        let maximum = properties.query_property(TARGET_MAX_DISTANCE);
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let path = game.skill_target_path_with_length(skill.lifecycle(), maximum);
        let Some(state) = game.registered_skill_mut(instance)
            .and_then(|skill| skill.player_state_mut::<BloodRoseExecutionState>())
        else { return terminal(QueuedSkillExecutionState::Rejected); };
        state.path = path;
        let stop = state.path.iter().position(|cell| cell.2 == 2);
        let index = stop.unwrap_or(state.path.len()) as u32;
        let blocked_endpoint = stop.and_then(|index| state.path.get(index)).map(|cell| (cell.0, cell.1));
        let last_endpoint = state.path.last().map(|cell| (cell.0, cell.1));
        if let Some(endpoint) = blocked_endpoint
            && let Some(skill) = game.registered_skill_mut(instance)
        {
            skill.lifecycle_mut().set_destination(endpoint);
        }
        let missile = index.wrapping_mul(properties.query_property(MISSILE_FLYING_TIME));
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        if let Some(state) = skill.player_state_mut::<BloodRoseExecutionState>() {
            state.missile_flying_time = missile;
        }
        if blocked_endpoint.is_none() && let Some(endpoint) = last_endpoint {
            skill.lifecycle_mut().set_destination(endpoint);
        }
        let destination = skill.lifecycle().destination();
        skill.lifecycle_mut().set_point_target(destination);
        game.update_registered_skill_visual(instance, 1);
        if let Some(state) = game.registered_skill_mut(instance)
            .and_then(|skill| skill.player_state_mut::<BloodRoseExecutionState>())
        {
            state.attacking_started = true;
            state.kernel.lifecycle_mut().mark_prepared();
            let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate);
            let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        }
    }
    let Some(attacking) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<BloodRoseExecutionState>())
        .map(|state| state.attacking_started)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !attacking { return terminal(QueuedSkillExecutionState::Pending); }
    let missile = properties.query_property(MISSILE_FLYING_TIME);
    let Some(position) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<BloodRoseExecutionState>())
        .map(|state| state.current_position)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if runtime.now_milliseconds() < missile.wrapping_mul(position).wrapping_add(delay).wrapping_add(started) {
        return terminal(QueuedSkillExecutionState::Pending);
    }
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    if !user.shape().is_assigned_to_server_region() { return terminal(QueuedSkillExecutionState::Pending); }
    let region = user.shape().get_region_id();
    if game.find_region(region).is_none() { return terminal(QueuedSkillExecutionState::Pending); }
    let Some(state) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<BloodRoseExecutionState>())
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if state.path.len() as u32 <= state.current_position {
        knock_back_owner(game, source, region, &properties);
        return terminal(QueuedSkillExecutionState::Completed);
    }
    let Some((x, y, _)) = state.path.get(state.current_position as usize).copied() else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    if let Some(state) = game.registered_skill_mut(instance)
        .and_then(|skill| skill.player_state_mut::<BloodRoseExecutionState>())
    {
        state.end_tile = (x, y);
    }
    let block = game.find_region(region).map_or(2, |owner| owner.base().skill_cell_block(x, y));
    let contact = block == 3 && run_blood_rose_scope(game, instance, source, (x, y), runtime);
    if contact || block == 2 {
        game.update_registered_skill_visual(instance, 3);
        if let Some(state) = game.registered_skill_mut(instance)
            .and_then(|skill| skill.player_state_mut::<BloodRoseExecutionState>())
        {
            state.current_position = state.path.len() as u32;
        }
    }
    if let Some(state) = game.registered_skill_mut(instance)
        .and_then(|skill| skill.player_state_mut::<BloodRoseExecutionState>())
    {
        state.current_position = state.current_position.wrapping_add(1);
    }
    terminal(QueuedSkillExecutionState::Pending)
}

pub(crate) fn execute_player_blood_rose<Runtime: GameMainLoopRuntime>(
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
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::CrossbowCast,
        |game, instance, _, runtime| {
            let Some(source) = original_user else { return false; };
            let target = if matches!(dispatch, PlayerSkillDispatch::Point { .. }) {
                game.registered_skill(instance).and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle()))
            } else { original_target };
            let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return false; };
            let targets_self = target.and_then(|(region, identity)| resolve_state_move_shape(game, region, identity))
                .is_some_and(|target| std::ptr::eq(user, target));
            if targets_self {
                ranged_weapon_failure(game, instance, Some(source.1.id), 10, RangedWeaponKind::Crossbow);
                return false;
            }
            check_ranged_weapon_cast(game, instance, source, ArrowCastPathRule::DistanceOnly, RangedWeaponKind::Crossbow, runtime)
        },
        |dispatch, started| BloodRoseExecutionState::begin(dispatch, started).into(), run_ai,
    )
}
