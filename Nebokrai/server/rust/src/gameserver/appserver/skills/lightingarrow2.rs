//! Клеточная световая стрела LightingArrow2 (0xE7).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/lightingarrow2.cpp.
//!
//! Общий зарегистрированный Begin сохраняет ранние часы и исходного U для
//! Check: reuse, локальный путь, дальность, лук категории 3 и ненулевая цена
//! MP. Первый AI разрешает U/S один раз, проверяет смерть S, списывает MP,
//! публикует OnChangeStates, повторяет проверку лука и задаёт CAN/направление.
//! Таблица свойств остаётся той же до конца AI; Calculate получает свежую.
//!
//! Выпуск сохраняет путь и время одного шага, исключает первую непролётную
//! клетку из атакуемого префикса и переводит цель в endpoint перед visual1.
//! Attacking/prepared включаются после visual. Все сроки абсолютные unsigned;
//! один AI обрабатывает не более одной клетки даже при большом опоздании.
//! Путь, позиция и список ударов принадлежат единственному зарегистрированному
//! экземпляру и остаются опубликованы при callbacks фонового полёта.
//!
//! Каждая клетка получает снимок полного GetShapes из текущего региона U,
//! затем заново разрешает Move-цели и их виртуальный допуск. Дедупликация
//! относится только к урону: яд вызывается и при повторной встрече цели.
//! Общий оружейный расчёт сохраняет MIN→MAX→сырой RNG→свежий MIN, три компонента,
//! x87-коэффициент и критическое усечение. NULL properties оставляют UNKNOWN/1
//! для сырого OnBeenAttacked; IncreaseRp здесь отсутствует.
//!
//! End сбрасывает фазу и счётчики, освобождает путь/список до свежего U Move1
//! и общего Attack End. Безопасные Vec и общий kernel заменяют native
//! контейнеры и указатели; регион входит в ключ дедупликации региональных фигур.

use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::flash::cell_views;
use super::heartlessarrow::apply_daub_poison;
use super::kernel::{SkillExecutionKernel, SkillStage, SkillTermination};
use super::lightingarrowphalanx::ArrowTargetIdentity;
use super::playercast::execute_registered_player_cast;
use super::rangedweaponcast::{
    ArrowCastPathRule, RangedWeaponKind, check_ranged_weapon_cast,
    prepare_ranged_weapon_player, ranged_weapon_failure, terminal,
};
use super::weaponattack::{PlayerWeaponRoll, calculate_player_weapon_attack};
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

pub(crate) const LIGHTING_ARROW_2_SKILL_ID: u32 = 0xE7;
const PLAYER_TYPE: i32 = 400;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const MISSILE_FLYING_TIME: u32 = 10_008;
const TARGET_DAMAGE_FACTOR: u32 = 20_003;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LightingArrow2ExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    attacking_started: bool,
    missile_flying_time: u32,
    path: Vec<(i32, i32, u8)>,
    attack_cell_count: u32,
    current_cell: u32,
    attacked_creatures: Vec<ArrowTargetIdentity>,
}

impl LightingArrow2ExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started),
            attacking_started: false,
            missile_flying_time: 0,
            path: Vec::new(),
            attack_cell_count: 0,
            current_cell: 0,
            attacked_creatures: Vec::new(),
        }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub(crate) const fn missile_flying_time(&self) -> u32 { self.missile_flying_time }

    fn mark_target_attacked(&mut self, target: (i32, ShapeIdentity)) -> bool {
        let key = ArrowTargetIdentity::new(target.0, target.1);
        if self.attacked_creatures.contains(&key) { return false; }
        self.attacked_creatures.push(key);
        true
    }

    pub(crate) fn clear_end_paths(&mut self) {
        self.attacking_started = false;
        self.missile_flying_time = 0;
        self.attack_cell_count = 0;
        self.current_cell = 0;
        drop(std::mem::take(&mut self.path));
        drop(std::mem::take(&mut self.attacked_creatures));
    }
}

fn attack_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    if source == target { return; }
    if resolve_state_move_shape(game, source.0, source.1).is_none()
        || resolve_state_move_shape(game, target.0, target.1).is_none()
    { return; }
    let Some(state) = game.registered_skill_mut(instance)
        .and_then(|skill| skill.player_state_mut::<LightingArrow2ExecutionState>())
    else { return; };
    if !state.mark_target_attacked(target) { return; }
    let Some((mut master, attack)) = calculate_player_weapon_attack(
        game, instance, source, target, TARGET_DAMAGE_FACTOR, PlayerWeaponRoll::RawRange,
    ) else { return; };
    master.master_country_id = 0;
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
}

fn attack_cell<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    x: i32, y: i32, runtime: &mut Runtime,
) {
    if x == 0 && y == 0 { return; }
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region = user.shape().get_region_id();
    let user_identity = user.shape().identity();
    for view in cell_views(game, region, x, y) {
        if view.identity == user_identity { continue; }
        let Some(target) = resolve_state_move_shape(game, region, view.identity) else { continue; };
        let target = (target.shape().get_region_id(), target.shape().identity());
        if !game.live_skill_target_attackable(region, source.1, target.1) { continue; }
        attack_target(game, instance, source, target, runtime);
        // Native вызывает яд вне Attack и потому вне списка уже задетых целей.
        if source.1.object_type == PLAYER_TYPE {
            apply_daub_poison(game, source.1.id, target.0, target.1, &mut || runtime.now_milliseconds());
        }
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
    let user = resolve_state_move_shape(game, region, identity)
        .map(|source| (source.shape().get_region_id(), source.shape().identity()));
    let sufferer = resolve_skill_sufferer(game, skill.lifecycle());
    let destination = match sufferer {
        Some((region, identity)) => {
            if game.move_shape_health(region, identity) == Some(0) {
                ranged_weapon_failure(game, instance, user.filter(|source| source.1.object_type == PLAYER_TYPE).map(|source| source.1.id), 10, RangedWeaponKind::Bow);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
            let Some(target) = resolve_state_move_shape(game, region, identity) else {
                return terminal(QueuedSkillExecutionState::Rejected);
            };
            (target.shape().get_tile_x().unwrap_or(i32::MIN), target.shape().get_tile_y().unwrap_or(i32::MIN))
        }
        None => skill.lifecycle().destination(),
    };
    let Some(user) = user else { return terminal(QueuedSkillExecutionState::Rejected); };
    if stage == SkillStage::Begin {
        if !prepare_ranged_weapon_player(game, instance, (user.1.object_type == PLAYER_TYPE).then_some(user.1.id), &properties, RangedWeaponKind::Bow) {
            return terminal(QueuedSkillExecutionState::Rejected);
        }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        let Some(source) = resolve_state_move_shape(game, user.0, user.1) else {
            return terminal(QueuedSkillExecutionState::Rejected);
        };
        let y = source.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = source.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) {
            source.shape_mut().set_direction(direction);
        }
        game.update_registered_skill_visual(instance, 0);
        if let Some(skill) = game.registered_skill_mut(instance) {
            let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check);
        }
    }
    let Some(attacking) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<LightingArrow2ExecutionState>())
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
        if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) { source.set_moveable(true); }
        let maximum = properties.query_property(TARGET_MAX_DISTANCE);
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let path = game.skill_target_path_with_length(skill.lifecycle(), maximum);
        let empty = path.is_empty();
        let Some(state) = game.registered_skill_mut(instance)
            .and_then(|skill| skill.player_state_mut::<LightingArrow2ExecutionState>())
        else { return terminal(QueuedSkillExecutionState::Rejected); };
        state.path = path;
        if empty {
            game.update_registered_skill_visual(instance, 3);
            let _ = game.end_registered_instance(instance, 0, SkillTermination::Rejected, runtime);
            // Исходная ветка не возвращается после End(0): выполняются выпуск
            // с очищенным контекстом и второй End(1) с общим хвостом.
        }
        let Some(state) = game.registered_skill(instance)
            .and_then(|skill| skill.player_state::<LightingArrow2ExecutionState>())
        else { return terminal(QueuedSkillExecutionState::Rejected); };
        if properties.query_property(TARGET_MAX_DISTANCE) != 0 {
            let maximum = properties.query_property(TARGET_MAX_DISTANCE);
            if maximum.wrapping_add(1) < state.path.len() as u32 {
                ranged_weapon_failure(game, instance, (user.1.object_type == PLAYER_TYPE).then_some(user.1.id), 11, RangedWeaponKind::Bow);
                return terminal(QueuedSkillExecutionState::Rejected);
            }
        }
        let count = state.path.iter().position(|cell| cell.2 == 2).unwrap_or(state.path.len());
        let endpoint = state.path.get(count).or_else(|| state.path.last()).map(|cell| (cell.0, cell.1));
        let missile = properties.query_property(MISSILE_FLYING_TIME);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        if let Some(state) = skill.player_state_mut::<LightingArrow2ExecutionState>() {
            state.missile_flying_time = missile;
            state.attack_cell_count = count as u32;
        }
        let destination = endpoint.unwrap_or_else(|| skill.lifecycle().destination());
        skill.lifecycle_mut().set_point_target(destination);
        game.update_registered_skill_visual(instance, 1);
        if let Some(state) = game.registered_skill_mut(instance)
            .and_then(|skill| skill.player_state_mut::<LightingArrow2ExecutionState>())
        {
            state.attacking_started = true;
            state.kernel.lifecycle_mut().mark_prepared();
            let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate);
            let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        }
    }
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(state) = skill.player_state::<LightingArrow2ExecutionState>() else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
    if !state.attacking_started { return terminal(QueuedSkillExecutionState::Pending); }
    if state.current_cell >= state.attack_cell_count { return terminal(QueuedSkillExecutionState::Completed); }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let deadline = delay.wrapping_add(state.missile_flying_time.wrapping_mul(state.current_cell))
        .wrapping_add(skill.lifecycle().started_at_ms());
    if runtime.now_milliseconds() < deadline { return terminal(QueuedSkillExecutionState::Pending); }
    let cell = state.path.get(state.current_cell as usize).copied();
    if let Some((x, y, _)) = cell {
        attack_cell(game, instance, user, x, y, runtime);
        if let Some(state) = game.registered_skill_mut(instance)
            .and_then(|skill| skill.player_state_mut::<LightingArrow2ExecutionState>())
        {
            state.current_cell = state.current_cell.wrapping_add(1);
        }
    }
    terminal(QueuedSkillExecutionState::Pending)
}

pub(crate) fn execute_player_lighting_arrow_2<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::ArrowCast,
        |game, instance, _, runtime| original_user.is_some_and(|source|
            check_ranged_weapon_cast(game, instance, source, ArrowCastPathRule::DistanceOnly, RangedWeaponKind::Bow, runtime)),
        |dispatch, started| LightingArrow2ExecutionState::begin(dispatch, started).into(), run_ai,
    )
}
