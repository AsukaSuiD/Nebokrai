//! Трёхлучевой залп CRainArrow (0xCE).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/rainarrow.cpp.
//! Общий Begin проверяет исходного U, reuse и длину свежего пути без проверки
//! block2. Источник не типа Player допускается без Move0; игроку нужны лук
//! категории 3 и ненулевая цена MP с неотрицательной signed DWORD-разностью.
//! AI не проверяет смерть U/S. Первый AI списывает MP, вызывает OnChangeStates
//! и повторяет проверку лука, затем CAN и свежую S. Самонацеливание заменяется
//! соседней клеткой по направлению U; поздний отказ не возвращает MP.
//!
//! Угол зависит от RealDistance, signed нижней границы 1 и unsigned разности
//! углов. MAX0 не заменяется единицей: деление на ноль остаётся частью расчёта.
//! Промежуточные записи float, отдельно суженный cos и точная константа pi
//! сохранены; стандартные f64 sin/cos заменяют x87 без самописной тригонометрии.
//! Visual0 предшествует свежему разрешению S и построению путей. Центральный
//! путь задаёт конечную точку; боковые независимо добавляют первую клетку
//! и max-1 шагов, затем читают блоки по сохранённому региону базы Begin.
//!
//! После unsigned start+delay выпуск возвращает движение, фиксирует время
//! полёта и число клеток до первого block2 для center/left/right. Пути не
//! обрезаются и остаются опубликованными через visual1, prepared и Summon.
//! Summon отдельно читает свойства и MasterInfo с country0; региональный
//! owner размещает форму, снимает ThunderBlow в клетке и публикует вход.
//! Даже успешный залп вызывает End0: без износа оружия и обновления reuse.
//! End сбрасывает фазу и скаляры, освобождает right/center/left, возвращает
//! движение свежему U и передаёт настоящий аргумент общему Attack End.
//! Vec и поколенческий ключ заменяют native контейнеры и указатели.

use super::baseattack::SKILL_USAGE_USER_HIT_MODIFIER;
use super::basemagic::{
    SKILL_USAGE_CAN_BE_BREAKED, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_ELEMENT_MODIFIER,
    SKILL_USAGE_SUMMONED_LIFETIME, SKILL_USAGE_SUMMONED_SPEED, SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use super::fightdefense::truncate_original;
use super::kernel::{SkillExecutionKernel, SkillStage};
use super::rangedweaponcast::{
    ArrowCastPathRule, RangedWeaponKind, check_ranged_weapon_cast, prepare_ranged_weapon_player, terminal,
};
use super::playercast::execute_registered_player_cast;
use super::rainarrowphalanx::{CRainArrowPhalanx, RainArrowCell, RainArrowPath};
use super::skillbaseproperties::CSkillBaseProperties;
use super::skillpath::straight_skill_path;
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

const MINIMUM_ANGLE: u32 = 2_002;
const MAXIMUM_ANGLE: u32 = 2_003;
const MISSILE_FLYING_TIME: u32 = 10_008;
const DAMAGE_FACTOR: u32 = 20_003;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RainArrowExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    angle_bits: u32,
    missile_flying_time: u32,
    right: RainArrowPath,
    center: RainArrowPath,
    left: RainArrowPath,
}

impl RainArrowExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self {
            kernel: SkillExecutionKernel::begin(dispatch, started), angle_bits: 0,
            missile_flying_time: 0,
            right: RainArrowPath { cells: Vec::new(), active_cells: 0 },
            center: RainArrowPath { cells: Vec::new(), active_cells: 0 },
            left: RainArrowPath { cells: Vec::new(), active_cells: 0 },
        }
    }
    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub(crate) const fn missile_flying_time(&self) -> u32 { self.missile_flying_time }
    pub(crate) fn right_impact(&self) -> Option<(i32, i32)> { impact(&self.right) }
    pub(crate) fn left_impact(&self) -> Option<(i32, i32)> { impact(&self.left) }

    pub(crate) fn clear_end_paths(&mut self) {
        self.missile_flying_time = 0;
        self.right.active_cells = 0;
        self.center.active_cells = 0;
        self.left.active_cells = 0;
        self.angle_bits = 0;
        self.right.cells.clear();
        self.center.cells.clear();
        self.left.cells.clear();
    }
}

fn impact(path: &RainArrowPath) -> Option<(i32, i32)> {
    path.active_cells.checked_sub(1).and_then(|index| path.cells.get(index as usize)).map(|cell| (cell.0, cell.1))
}

fn round_rotated(value: f32) -> i32 {
    let truncated = truncate_original(f64::from(value));
    if f64::from(value) - f64::from(truncated) > 0.5 { truncated.wrapping_add(1) } else { truncated }
}

fn rotate(delta: (f32, f32), angle: f32) -> (f32, f32) {
    let radians = f64::from(angle) * f64::from(0.005_555_555_7_f32) * 3.141_592_653_5_f64;
    let cosine = radians.cos() as f32;
    let sine = radians.sin();
    (
        (f64::from(cosine) * f64::from(delta.0) - sine * f64::from(delta.1)) as f32,
        (sine * f64::from(delta.0) + f64::from(cosine) * f64::from(delta.1)) as f32,
    )
}

fn calculate_angle(
    game: &CGame, source: (i32, ShapeIdentity), destination: (i32, i32), properties: &CSkillBaseProperties,
) -> Option<f32> {
    let user = resolve_state_move_shape(game, source.0, source.1)?;
    let mut distance = user.shape().real_distance_to_point(destination.0, destination.1) as u32;
    if properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) < distance {
        distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    }
    if (distance as i32) < 1 { distance = 1; }
    let maximum_angle = properties.query_property(MAXIMUM_ANGLE) as f32;
    let upper = properties.query_property(MAXIMUM_ANGLE);
    let lower = properties.query_property(MINIMUM_ANGLE);
    let span = upper.wrapping_sub(lower) as f32;
    let distance = distance as i32 as f32;
    let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    Some((f64::from(maximum_angle) - f64::from(distance) / f64::from(maximum) * f64::from(span)) as f32)
}

fn side_path(
    game: &CGame, region: i32, first: (i32, i32), delta: (f32, f32), angle: f32, maximum: u32,
) -> Vec<RainArrowCell> {
    let (x, y) = rotate(delta, angle);
    let endpoint_y = first.1.wrapping_add(round_rotated(y));
    let endpoint_x = first.0.wrapping_add(round_rotated(x));
    let mut path = vec![(first.0, first.1, 2)];
    path.extend(straight_skill_path(
        None, first.0, first.1, endpoint_x, endpoint_y, Some(maximum.wrapping_sub(1)),
    ));
    // Native append-path перечитывает блоки всего контейнера, включая
    // заранее добавленную первую клетку, а не только созданного суффикса.
    if let Some(region) = game.find_region(region) {
        for cell in &mut path { cell.2 = region.base().skill_cell_block(cell.0, cell.1); }
    }
    path
}

fn calculate_final_position(game: &mut CGame, instance: RegisteredSkill, maximum: u32) {
    let Some(skill) = game.registered_skill(instance) else { return; };
    if resolve_skill_sufferer(game, skill.lifecycle()).is_some() {
        let Some((region, target)) = game.registered_skill(instance)
            .and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle())) else { return; };
        let Some(target) = resolve_state_move_shape(game, region, target) else { return; };
        let x = target.shape().get_tile_x().unwrap_or(i32::MIN);
        let Some(skill) = game.registered_skill_mut(instance) else { return; };
        let (_, y) = skill.lifecycle().destination();
        skill.lifecycle_mut().set_destination((x, y));
        let Some((region, target)) = game.registered_skill(instance)
            .and_then(|skill| resolve_skill_sufferer(game, skill.lifecycle())) else { return; };
        let Some(target) = resolve_state_move_shape(game, region, target) else { return; };
        let y = target.shape().get_tile_y().unwrap_or(i32::MIN);
        let Some(skill) = game.registered_skill_mut(instance) else { return; };
        skill.lifecycle_mut().set_point_target((x, y));
    }
    let Some(skill) = game.registered_skill(instance) else { return; };
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = (user.shape().get_region_id(), user.shape().identity());
    if skill.lifecycle().destination() == (0, 0) { return; }
    let center = game.skill_target_path_with_length(skill.lifecycle(), maximum);
    let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<RainArrowExecutionState>()) else { return; };
    state.center.cells = center;
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let mut first = (user.shape().get_tile_x().unwrap_or(i32::MIN), user.shape().get_tile_y().unwrap_or(i32::MIN));
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(state) = skill.player_state::<RainArrowExecutionState>() else { return; };
    let mut destination = skill.lifecycle().destination();
    if let (Some(begin), Some(end)) = (state.center.cells.first(), state.center.cells.last()) {
        destination = (end.0, end.1);
        first = (begin.0, begin.1);
    }
    let angle = f32::from_bits(state.angle_bits);
    let Some(skill) = game.registered_skill_mut(instance) else { return; };
    skill.lifecycle_mut().set_destination(destination);
    let delta = ((f64::from(destination.0) - f64::from(first.0)) as f32,
        (f64::from(destination.1) - f64::from(first.1)) as f32);
    let left = side_path(game, region, first, delta, angle, maximum);
    let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<RainArrowExecutionState>()) else { return; };
    state.left.cells = left;
    let right = side_path(game, region, first, delta, -angle, maximum);
    if let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<RainArrowExecutionState>()) { state.right.cells = right; }
}

fn usable(path: &[RainArrowCell]) -> u32 {
    path.iter().position(|cell| cell.2 == 2).unwrap_or(path.len()) as u32
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
    if identity.object_type == 400 {
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
    let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let speed = properties.query_property(SKILL_USAGE_SUMMONED_SPEED);
    let Some(state) = game.registered_skill(instance).and_then(|skill| skill.player_state::<RainArrowExecutionState>()) else { return; };
    let right_count = state.right.active_cells;
    let center_count = state.center.active_cells;
    let left_count = state.left.active_cells;
    let factor = properties.query_property(DAMAGE_FACTOR) as i32;
    let hit = properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let lifetime = properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME);
    let Some(state) = game.registered_skill(instance).and_then(|skill| skill.player_state::<RainArrowExecutionState>()) else { return; };
    let left = RainArrowPath { cells: state.left.cells.clone(), active_cells: left_count };
    let center = RainArrowPath { cells: state.center.cells.clone(), active_cells: center_count };
    let right = RainArrowPath { cells: state.right.cells.clone(), active_cells: right_count };
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let phalanx = CRainArrowPhalanx::new(
        id, master, started, lifetime, level, hit, factor,
        left, center, right, speed as i32, maximum as i32,
    );
    let _ = game.spawn_rain_arrow_phalanx(region, phalanx, x, y, started, runtime);
}

fn run_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else { return terminal(QueuedSkillExecutionState::Pending); };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return terminal(QueuedSkillExecutionState::Rejected); };
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let source = (user.shape().get_region_id(), user.shape().identity());
    if stage == SkillStage::Begin {
        let player = (source.1.object_type == 400).then_some(source.1.id);
        if !prepare_ranged_weapon_player(game, instance, player, &properties, RangedWeaponKind::Bow) { return terminal(QueuedSkillExecutionState::Rejected); }
        let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        skill.lifecycle_mut().set_available(can_break != 0);
        let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let mut destination = skill.lifecycle().destination();
        if let Some((region, identity)) = resolve_skill_sufferer(game, skill.lifecycle()) {
            let Some(target) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
            destination = (target.shape().get_tile_x().unwrap_or(i32::MIN), target.shape().get_tile_y().unwrap_or(i32::MIN));
            if (target.shape().get_region_id(), target.shape().identity()) == source {
                let direction = target.shape().get_direction();
                if let Ok(point) = CShape::get_direction_position(direction, ShapeAreaCoordinates { x: destination.0, y: destination.1 }) {
                    destination = (point.x, point.y);
                }
                let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
                skill.lifecycle_mut().set_point_target(destination);
            }
        }
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        let direction = get_line_direction(x, y, destination.0, destination.1);
        if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.shape_mut().set_direction(direction); }
        let Some(angle) = calculate_angle(game, source, destination, &properties) else { return terminal(QueuedSkillExecutionState::Rejected); };
        let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<RainArrowExecutionState>()) else { return terminal(QueuedSkillExecutionState::Rejected); };
        state.angle_bits = angle.to_bits();
        game.update_registered_skill_visual(instance, 0);
        let maximum = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
        calculate_final_position(game, instance, maximum);
        if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    }
    let delay = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let Some(started) = game.registered_skill(instance).map(|skill| skill.lifecycle().started_at_ms()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if runtime.now_milliseconds() < started.wrapping_add(delay) { return terminal(QueuedSkillExecutionState::Pending); }
    if let Some(user) = resolve_state_move_shape_mut(game, source.0, source.1) { user.set_moveable(true); }
    let flight = properties.query_property(MISSILE_FLYING_TIME);
    let Some(skill) = game.registered_skill_mut(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(state) = skill.player_state_mut::<RainArrowExecutionState>() else { return terminal(QueuedSkillExecutionState::Rejected); };
    state.missile_flying_time = flight;
    let center_count = usable(&state.center.cells);
    let blocked_destination = state.center.cells.get(center_count as usize).map(|cell| (cell.0, cell.1));
    state.center.active_cells = center_count;
    state.left.active_cells = usable(&state.left.cells);
    state.right.active_cells = usable(&state.right.cells);
    if let Some(destination) = blocked_destination { skill.lifecycle_mut().set_destination(destination); }
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

pub(crate) fn execute_player_rain_arrow<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let original_user = game.find_player(player_id)
        .map(|player| (player.shape().get_region_id(), player.shape().identity()));
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, SkillVisualEffectKind::ArrowCast,
        |game, instance, _player_id, runtime| original_user
            .is_some_and(|source| check_ranged_weapon_cast(game, instance, source, ArrowCastPathRule::DistanceOnly, RangedWeaponKind::Bow, runtime)),
        |dispatch, started| RainArrowExecutionState::begin(dispatch, started).into(), run_ai,
    )
}
