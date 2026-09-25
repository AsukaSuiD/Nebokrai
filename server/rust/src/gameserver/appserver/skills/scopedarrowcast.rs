//! Общий поклеточный выстрел BloodRose и ExplosiveArrow/2/3 (D0/D6/E1/E2).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/bloodrose.cpp
//! и explosivearrow{,2,3}.cpp. Конкретные ID задают оружие, обход боевых душ
//! и условие отдачи; единственное исполнение не хранит копию этого профиля.
//!
//! Begin сохраняет исходные U/S для объектного Check; координатный Check
//! разрешает S после visual. Самонацеливание отклоняется до свойств/reuse,
//! затем идут дальность без BLOCK_UNFLY и signed MP. Цена MP0 даёт тихий отказ,
//! источник не типа Player проходит без Move0. D0/D6 требуют арбалет категории
//! 4; E1/E2 — лук категории 3, но отсутствие оружия сохраняет строку GS0293.
//! Отказ Begin не добавляет отдельный visual2 перед End(0).
//!
//! Первый AI списывает MP до OnChangeStates и повторной проверки оружия,
//! задаёт CAN, свежие S.X/Y, U.Y/X, направление и visual0. Выпуск после
//! абсолютного unsigned срока возвращает движение, строит путь длиной MAX
//! (включая MAX0), сохраняет endpoint/время полёта и очищает S до visual1.
//! Attacking/prepared пишутся после visual; индекс остаётся нулевым.
//!
//! Один AI обрабатывает не более клетки; step/delay берутся из таблицы этого
//! AI. Текущий регион U проверяется после часов, даже для завершённого пути.
//! Все уровни и варианты используют полную маску 3×3. Общий attack-owner
//! сохраняет X→Y, visual-цель до дедупликации и живой список попаданий.
//! Только E1 повторяет обход боевых душ перед каждой клеткой: обычный список
//! подавляет такой урон, но души не пополняют его и не останавливают полёт.
//!
//! После visual3 текущая позиция получает свежую длину пути и увеличивается.
//! Следующий подходящий AI выполняет отдачу стрелка, затем End(1), без нового
//! visual3. D0 читает скорость и вызывает ForceMove только при смещении;
//! ExplosiveArrow делает оба действия даже при нуле шагов. End сбрасывает
//! фазу и счётчики, освобождает путь перед списком до свежего U Move1 и общего
//! Attack End. Vec и kernel заменяют native контейнеры; одинаковые маски не
//! требуют отдельного объекта. Весь AI опубликован при синхронных callbacks.

use super::baseattack::SKILL_USAGE_DELAY_TIME;
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::crossbowattack::run_scoped_arrow_attack;
use super::kernel::{SkillStage};
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
pub(crate) use nebokrai_zone::skills::execution::{ScopedArrowExecutionState};

pub(crate) const BLOOD_ROSE_SKILL_ID: u32 = 0xD0;
pub(crate) const EXPLOSIVE_ARROW_SKILL_ID: u32 = 0xD6;
pub(crate) const EXPLOSIVE_ARROW_2_SKILL_ID: u32 = 0xE1;
pub(crate) const EXPLOSIVE_ARROW_3_SKILL_ID: u32 = 0xE2;
const PLAYER_TYPE: i32 = 400;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const MISSILE_FLYING_TIME: u32 = 10_008;
const TARGET_BACK_STEP: u32 = 30_002;
const TARGET_MOVE_SPEED: u32 = 30_003;

fn weapon_kind(skill_id: u32) -> Option<RangedWeaponKind> {
    match skill_id {
        BLOOD_ROSE_SKILL_ID | EXPLOSIVE_ARROW_SKILL_ID => Some(RangedWeaponKind::Crossbow),
        EXPLOSIVE_ARROW_2_SKILL_ID | EXPLOSIVE_ARROW_3_SKILL_ID => Some(RangedWeaponKind::ExplosiveBow),
        _ => None,
    }
}

fn knock_back_owner(
    game: &mut CGame, source: (i32, ShapeIdentity), region: i32,
    properties: &CSkillBaseProperties, force_even_if_stationary: bool,
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
    let should_move = force_even_if_stationary || resolve_state_move_shape(game, source.0, source.1)
        .is_some_and(|user| position.x != user.shape().get_tile_x().unwrap_or(i32::MIN)
            || position.y != user.shape().get_tile_y().unwrap_or(i32::MIN));
    if should_move {
        let duration = properties.query_property(TARGET_MOVE_SPEED).wrapping_mul(count);
        let _ = game.force_move_skill_target(region, source.1, position.x, position.y, duration);
    }
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let skill_id = skill.id();
    let Some(weapon) = weapon_kind(skill_id) else { return terminal(QueuedSkillExecutionState::Rejected); };
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
            &properties, weapon,
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
        .and_then(|skill| skill.player_state::<ScopedArrowExecutionState>())
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
            .and_then(|skill| skill.player_state_mut::<ScopedArrowExecutionState>())
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
        if let Some(state) = skill.player_state_mut::<ScopedArrowExecutionState>() {
            state.missile_flying_time = missile;
        }
        if blocked_endpoint.is_none() && let Some(endpoint) = last_endpoint {
            skill.lifecycle_mut().set_destination(endpoint);
        }
        let destination = skill.lifecycle().destination();
        skill.lifecycle_mut().set_point_target(destination);
        game.update_registered_skill_visual(instance, 1);
        if let Some(state) = game.registered_skill_mut(instance)
            .and_then(|skill| skill.player_state_mut::<ScopedArrowExecutionState>())
        {
            state.attacking_started = true;
            state.kernel.lifecycle_mut().mark_prepared();
            let _ = state.kernel.advance(SkillStage::Check, SkillStage::Calculate);
            let _ = state.kernel.advance(SkillStage::Calculate, SkillStage::Attack);
        }
    }
    let Some(attacking) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<ScopedArrowExecutionState>())
        .map(|state| state.attacking_started)
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !attacking { return terminal(QueuedSkillExecutionState::Pending); }
    let missile = properties.query_property(MISSILE_FLYING_TIME);
    let Some(position) = game.registered_skill(instance)
        .and_then(|skill| skill.player_state::<ScopedArrowExecutionState>())
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
        .and_then(|skill| skill.player_state::<ScopedArrowExecutionState>())
    else { return terminal(QueuedSkillExecutionState::Rejected); };
    if state.path.len() as u32 <= state.current_position {
        knock_back_owner(game, source, region, &properties, skill_id != BLOOD_ROSE_SKILL_ID);
        return terminal(QueuedSkillExecutionState::Completed);
    }
    let Some((x, y, _)) = state.path.get(state.current_position as usize).copied() else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    if let Some(state) = game.registered_skill_mut(instance)
        .and_then(|skill| skill.player_state_mut::<ScopedArrowExecutionState>())
    {
        state.end_tile = (x, y);
    }
    let block = game.find_region(region).map_or(2, |owner| owner.base().skill_cell_block(x, y));
    let contact = block == 3 && run_scoped_arrow_attack(game, instance, source, (x, y), runtime);
    if contact || block == 2 {
        game.update_registered_skill_visual(instance, 3);
        if let Some(state) = game.registered_skill_mut(instance)
            .and_then(|skill| skill.player_state_mut::<ScopedArrowExecutionState>())
        {
            state.current_position = state.path.len() as u32;
        }
    }
    if let Some(state) = game.registered_skill_mut(instance)
        .and_then(|skill| skill.player_state_mut::<ScopedArrowExecutionState>())
    {
        state.current_position = state.current_position.wrapping_add(1);
    }
    terminal(QueuedSkillExecutionState::Pending)
}

pub(crate) fn execute_player_scoped_arrow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(weapon) = game.registered_skill(instance).and_then(|skill| weapon_kind(skill.id())) else {
        return terminal(QueuedSkillExecutionState::Rejected);
    };
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
                ranged_weapon_failure(game, instance, (source.1.object_type == PLAYER_TYPE).then_some(source.1.id), 10, weapon);
                return false;
            }
            check_ranged_weapon_cast(game, instance, source, ArrowCastPathRule::DistanceOnly, weapon, runtime)
        },
        |dispatch, started| ScopedArrowExecutionState::begin(dispatch, started).into(), run_ai,
    )
}
