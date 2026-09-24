//! Прямые рывки Rush/Rush2. Источник: gameserver.exe/GameServer.pdb,
//! appserver/skills/rush.cpp и общий предметный контракт rush2.cpp.
//!
//! Вход CPlayer проверяет меч, signed MP/RP и Pillar; AI повторно списывает
//! MP перед RP, сохраняет частичный расход и вызывает OnChangeStates до
//! повторной проверки оружия. Таблица свойств AI остаётся прежней через
//! callbacks. CAN независим от concrete фазы, задержки AI у рывков нет.
//! Путь заданной длины принадлежит захваченному экземпляру. Перенос в последнюю
//! свободную клетку предшествует visual0/condition/visual1; область удара —
//! первая преграда либо исходная целевая клетка. Снимок её фигур допускает
//! только native типы 400/600/601/602, затем проверяет смерть и IsAttackAble.
//!
//! Rush сообщает PK-контакт до живых уровней; Rush2 доставляет пустой удар
//! после состояния и отбрасывания. Новый state создаётся до прежнего End,
//! удаляется свежий остаток слота, затем Begin/append и ForceMove. Свойства
//! этого хвоста захвачены до state callbacks, число шагов читается повторно.
//! Общий зарегистрированный вход завершает навык; End сбрасывает фазу,
//! возвращает движение и лишь затем освобождает путь. Vec заменяет владение
//! исходного списка, не копируя его через callbacks.

use super::baseattack::{SKILL_USAGE_REUSE_DELAY_TIME, real_distance};
use super::basemagic::SKILL_USAGE_CAN_BE_BREAKED;
use super::playercast::execute_registered_player_cast;
use super::flash::cell_views;
use super::fightdefense::truncate_original;
use super::kernel::{skill_is_restored, SkillExecutionKernel, SkillStage};
use super::rushstate::{begin_primary_rush_state, RushState, RUSH_STATE_ID};
use super::skillbaseproperties::CSkillBaseProperties;
use super::skillfactory::SkillOwner;
use crate::gameserver::appserver::goods::cgoodsbaseproperties::GAP_WEAPON_CATEGORY;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::{CPlayer, PlayerSkillDispatch};
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_skill_sufferer, resolve_state_move_shape,
    resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState,
};
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::GameMessageDomainOps;
use crate::public::tools::get_line_direction;

pub(crate) const RUSH_SKILL_ID: u32 = 0x73;
const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const PLAYER_TYPE: i32 = 400;
const USER_MP_LOSE: u32 = 2;
const USER_RP_LOSE: u32 = 3;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_BACK_STEP: u32 = 1_001;
const TARGET_MOVE_SPEED: u32 = 2_001;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct RushExecutionState {
    kernel: SkillExecutionKernel<PlayerSkillDispatch>,
    path: Vec<(i32, i32, u8)>,
}

impl RushExecutionState {
    fn begin(dispatch: PlayerSkillDispatch, started: u32) -> Self {
        Self { kernel: SkillExecutionKernel::begin(dispatch, started), path: Vec::new() }
    }

    pub(crate) const fn kernel(&self) -> &SkillExecutionKernel<PlayerSkillDispatch> { &self.kernel }
    pub(crate) fn kernel_mut(&mut self) -> &mut SkillExecutionKernel<PlayerSkillDispatch> { &mut self.kernel }
    pub(crate) fn clear_end_paths(&mut self) { drop(std::mem::take(&mut self.path)); }
}

fn terminal(state: QueuedSkillExecutionState) -> QueuedSkillExecutionOutcome {
    QueuedSkillExecutionOutcome { state, first_contact: false }
}

fn visual_kind(skill_id: u32) -> SkillVisualEffectKind {
    if skill_id == RUSH_SKILL_ID { SkillVisualEffectKind::Rush } else { SkillVisualEffectKind::Rush2 }
}

pub(crate) fn publish_rush_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    if !matches!(skill.owner(), SkillOwner::CRush | SkillOwner::CRush2)
        || skill.visual_effect().is_none_or(|effect| effect.kind() != visual_kind(skill.id()) || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    let identity = source.identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    if matches!(mode, 2 | 7 | 8 | 13 | 14) {
        if identity.object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), identity.id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, _ => return };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    if action == 1 {
        message.add_long(source.get_direction());
    } else {
        let target = resolve_skill_sufferer(game, skill.lifecycle());
        message.add_long(target.map_or(0, |(_, target)| target.object_type));
        message.add_long(target.map_or(0, |(_, target)| target.id));
        let x = source.get_tile_x().unwrap_or(i32::MIN);
        let y = source.get_tile_y().unwrap_or(i32::MIN);
        message.add_long(x);
        message.add_long(y);
    }
    if let Some(region) = game.find_region(source.get_region_id()) {
        let _ = game.send_game_shape_around(region.base(), source, None, &message);
    }
}

fn weapon_is_valid(game: &CGame, player: &CPlayer) -> bool {
    player.equipment().get_goods(2).is_some_and(|weapon| {
        weapon.addon_property_value(game.goods_factory(), GAP_WEAPON_CATEGORY, 1) == 1
    })
}

fn failure(game: &mut CGame, instance: RegisteredSkill, player_id: i32, code: u32) {
    game.update_registered_skill_visual(instance, code);
    let text: &[u8] = match code { 2 => b"GS0302", 13 => b"GS0278", 14 => b"GS0287", _ => return };
    game.send_skill_system_info(player_id, text);
}

fn resource_failure(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32,
    properties: &CSkillBaseProperties, usage: u32,
) {
    let (code, text): (u32, &[u8]) = if usage == USER_MP_LOSE { (7, b"GS0288") } else { (8, b"GS0289") };
    game.update_registered_skill_visual(instance, code);
    let amount = properties.query_property(usage);
    game.send_skill_system_info_with_unsigned(player_id, text, amount);
}

pub(super) fn scaled_state_time(source_level: u8, target_level: u8, base_time: u32) -> u32 {
    if u32::from(source_level) + 5 >= u32::from(target_level) { return base_time; }
    let difference = i32::from(target_level) - i32::from(source_level) - 5;
    let factor = (1.0_f32 - difference as f32 * 0.25).max(0.0);
    truncate_original(f64::from(base_time) * f64::from(factor)) as u32
}

pub(super) fn prepare_rush_control(
    game: &CGame, instance: RegisteredSkill, user: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
) -> Option<(CSkillBaseProperties, i32)> {
    resolve_state_move_shape(game, user.0, user.1)?;
    let target_shape = resolve_state_move_shape(game, target.0, target.1)?;
    if !game.live_skill_target_attackable(target_shape.shape().get_region_id(), user.1, target.1) {
        return None;
    }
    let skill = game.registered_skill(instance)?;
    let properties = game.skill_base_properties(skill.id(), skill.level())?.clone();
    let source = resolve_state_move_shape(game, user.0, user.1)?.shape();
    if !source.is_assigned_to_server_region() { return None; }
    game.find_region(source.get_region_id())?;
    Some((properties, source.get_region_id()))
}

pub(super) fn remove_previous_rush_state(
    game: &mut CGame, target: (i32, ShapeIdentity), skill_id: u32,
) {
    if let Some((index, _)) = resolve_state_move_shape(game, target.0, target.1)
        .and_then(|shape| shape.find_state_position(|state| state.state_id() == skill_id))
    {
        let _ = end_and_destroy_state_at(game, target.0, target.1, index);
    }
}

pub(super) fn rush_knockback(
    game: &mut CGame, properties: &CSkillBaseProperties, source_region: i32,
    user: (i32, ShapeIdentity), target: (i32, ShapeIdentity),
) {
    if properties.query_property(TARGET_BACK_STEP) == 0 { return; }
    let Some(target_shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let target_y = target_shape.shape().get_tile_y().unwrap_or(i32::MIN);
    let target_x = target_shape.shape().get_tile_x().unwrap_or(i32::MIN);
    let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return; };
    let source_y = source.shape().get_tile_y().unwrap_or(i32::MIN);
    let source_x = source.shape().get_tile_x().unwrap_or(i32::MIN);
    let direction = get_line_direction(source_x, source_y, target_x, target_y);
    let Some(target_shape) = resolve_state_move_shape(game, target.0, target.1) else { return; };
    let x = target_shape.shape().get_tile_x().unwrap_or(i32::MIN);
    let y = target_shape.shape().get_tile_y().unwrap_or(i32::MIN);
    let mut position = ShapeAreaCoordinates { x, y };
    let Some(region) = game.find_region(source_region).map(|region| region.base()) else { return; };
    let mut steps = properties.query_property(TARGET_BACK_STEP);
    let mut moved = 0u32;
    while moved < steps {
        let Ok(next) = CShape::get_direction_position(direction, position) else { break; };
        if next.x < 0 || next.y < 0 || next.x >= region.region.width || next.y >= region.region.height
            || region.skill_cell_block(next.x, next.y) & 7 != 0 { break; }
        position = next;
        moved = moved.wrapping_add(1);
        steps = properties.query_property(TARGET_BACK_STEP);
    }
    let duration = properties.query_property(TARGET_MOVE_SPEED).wrapping_mul(moved);
    // При ненулевом BACK_STEP ForceMove вызывается и после нуля свободных шагов.
    let _ = game.force_move_skill_target(target.0, target.1, position.x, position.y, duration);
}

fn add_rush_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, user: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), keep: u32, runtime: &mut Runtime,
) {
    let Some((properties, source_region)) = prepare_rush_control(game, instance, user, target) else { return; };
    let state = RushState::new(keep);
    remove_previous_rush_state(game, target, RUSH_STATE_ID);
    let _ = begin_primary_rush_state(
        game, target.0, target.1, Some(user), Some(target), state, &mut || runtime.now_milliseconds(),
    );
    rush_knockback(game, &properties, source_region, user, target);
}

fn check_cast<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, player_id: i32, runtime: &mut Runtime,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, runtime.now_milliseconds()) {
        failure(game, instance, player_id, 13);
        return false;
    }
    if !weapon_is_valid(game, player) {
        failure(game, instance, player_id, 14);
        return false;
    }
    if properties.query_property(USER_MP_LOSE) != 0 {
        let mana = player.mana();
        let loss = properties.query_property(USER_MP_LOSE);
        if (mana.wrapping_sub(loss) as i32) < 0 {
            resource_failure(game, instance, player_id, &properties, USER_MP_LOSE);
            return false;
        }
    }
    if properties.query_property(USER_RP_LOSE) != 0 {
        let rp = u32::from(player.rp());
        let loss = properties.query_property(USER_RP_LOSE);
        if (rp.wrapping_sub(loss) as i32) < 0 {
            resource_failure(game, instance, player_id, &properties, USER_RP_LOSE);
            return false;
        }
    }
    if player.has_state_by_skill_id(0x74) {
        failure(game, instance, player_id, 2);
        return false;
    }
    let Some(player) = game.find_player_mut(player_id) else { return false; };
    player.set_skill_moveable(false);
    true
}

fn rush_ai<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return terminal(QueuedSkillExecutionState::Pending);
    };
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else { return terminal(QueuedSkillExecutionState::Rejected); };
    let (region_id, identity) = skill.lifecycle().user();
    let Some(source) = resolve_state_move_shape(game, region_id, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
    if !source.shape().is_assigned_to_server_region() { return terminal(QueuedSkillExecutionState::Rejected); }
    let user = (source.shape().get_region_id(), source.shape().identity());
    // Native хранит impact только в локальных переменных первого AI: повторный
    // вход с condition1 читает неинициализированную клетку. Не создаём её из нулей.
    if stage != SkillStage::Begin { return terminal(QueuedSkillExecutionState::Rejected); }
    let mut impact = match resolve_skill_sufferer(game, skill.lifecycle()) {
        Some((region, identity)) => {
            let Some(target) = resolve_state_move_shape(game, region, identity) else { return terminal(QueuedSkillExecutionState::Rejected); };
            (target.shape().get_tile_x().unwrap_or(i32::MIN), target.shape().get_tile_y().unwrap_or(i32::MIN))
        }
        None => skill.lifecycle().destination(),
    };
    let Some(player) = game.find_player(user.1.id).filter(|_| user.1.object_type == PLAYER_TYPE) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let mana = player.mana();
    let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
    if (remaining as i32) < 0 {
        resource_failure(game, instance, user.1.id, &properties, USER_MP_LOSE);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if let Some(player) = game.find_player_mut(user.1.id) { player.set_mana(remaining); }
    let Some(player) = game.find_player(user.1.id) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let rp = u32::from(player.rp());
    let remaining = rp.wrapping_sub(properties.query_property(USER_RP_LOSE));
    if (remaining as i32) < 0 {
        resource_failure(game, instance, user.1.id, &properties, USER_RP_LOSE);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    if let Some(player) = game.find_player_mut(user.1.id) { player.set_rp(remaining as u16); }
    game.publish_player_states(user.1.id);
    if game.find_player(user.1.id).is_none_or(|player| !weapon_is_valid(game, player)) {
        failure(game, instance, user.1.id, 14);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_available(can_break != 0); }
    let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let source_y = source.shape().get_tile_y().unwrap_or(i32::MIN);
    let source_x = source.shape().get_tile_x().unwrap_or(i32::MIN);
    if let Some(source) = resolve_state_move_shape_mut(game, user.0, user.1) {
        source.shape_mut().set_direction(get_line_direction(source_x, source_y, impact.0, impact.1));
    }
    let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let mut destination = (source.shape().get_tile_x().unwrap_or(i32::MIN), source.shape().get_tile_y().unwrap_or(i32::MIN));
    let maximum = properties.query_property(TARGET_MAX_DISTANCE);
    let Some(skill) = game.registered_skill(instance) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let path = game.skill_target_path_with_length(skill.lifecycle(), maximum);
    let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<RushExecutionState>()) else { return terminal(QueuedSkillExecutionState::Rejected); };
    state.path = path;
    if state.path.is_empty() { return terminal(QueuedSkillExecutionState::Rejected); }
    for &(x, y, block) in &state.path {
        if block != 0 { impact = (x, y); break; }
        destination = (x, y);
    }
    let _ = game.set_player_tile_position(user.1.id, destination.0, destination.1);
    game.update_registered_skill_visual(instance, 0);
    if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    game.update_registered_skill_visual(instance, 1);
    let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { return terminal(QueuedSkillExecutionState::Rejected); };
    let source_shape = source.shape();
    let region_id = source_shape.get_region_id();
    if !source_shape.is_assigned_to_server_region() || game.find_region(region_id).is_none() {
        game.update_registered_skill_visual(instance, 2);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    let x = source_shape.get_tile_x().unwrap_or(i32::MIN);
    let y = source_shape.get_tile_y().unwrap_or(i32::MIN);
    let distance = real_distance(x, y, impact.0, impact.1) as u32;
    let maximum = properties.query_property(TARGET_MAX_DISTANCE);
    if if skill_id == RUSH_SKILL_ID { distance > maximum } else { distance >= maximum } {
        game.update_registered_skill_visual(instance, 2);
        return terminal(QueuedSkillExecutionState::Rejected);
    }
    for view in cell_views(game, region_id, impact.0, impact.1) {
        let identity = view.identity;
        if !matches!(identity.object_type, 400 | 600 | 601 | 602) { continue; }
        let Some(target_shape) = resolve_state_move_shape(game, region_id, identity) else { continue; };
        let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { continue; };
        if std::ptr::eq(source, target_shape) { continue; }
        let target = (target_shape.shape().get_region_id(), identity);
        if game.base_magic_target_dead(target.0, target.1)
            || !game.live_skill_target_attackable(target.0, user.1, target.1) { continue; }
        if skill_id == RUSH_SKILL_ID
            && let Some(controller) = game.skill_target_controller(target.0, target.1)
            && controller != user.1.id
        {
            let Some(source) = resolve_state_move_shape(game, user.0, user.1) else { continue; };
            let y = source.shape().get_tile_y().unwrap_or(i32::MIN);
            let x = source.shape().get_tile_x().unwrap_or(i32::MIN);
            let _ = game.player_on_first_attack_at_position(user.1.id, controller, Some(region_id), (x, y), runtime);
        }
        let Some(mut source_level) = game.move_shape_level(user.0, user.1) else { continue; };
        let Some(mut target_level) = game.move_shape_level(target.0, target.1) else { continue; };
        if u32::from(source_level) + 5 < u32::from(target_level) {
            let Some(level) = game.move_shape_level(user.0, user.1) else { continue; };
            source_level = level;
            let Some(level) = game.move_shape_level(target.0, target.1) else { continue; };
            target_level = level;
        }
        let keep = scaled_state_time(source_level, target_level, properties.query_property(STATE_PERSIST_TIME));
        if keep == 0 { continue; }
        if skill_id == RUSH_SKILL_ID {
            add_rush_state(game, instance, user, target, keep, runtime);
        } else {
            super::rush2::add_rush_2_state(game, instance, user, target, keep, runtime);
        }
    }
    terminal(QueuedSkillExecutionState::Completed)
}

pub(super) fn execute_rush<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    execute_registered_player_cast(
        game, player_id, instance, dispatch, runtime, visual_kind(dispatch.skill_id()),
        check_cast, |dispatch, started| RushExecutionState::begin(dispatch, started).into(), rush_ai,
    )
}

pub(crate) fn execute_player_rush<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    if dispatch.skill_id() != RUSH_SKILL_ID { return terminal(QueuedSkillExecutionState::Rejected); }
    execute_rush(game, player_id, instance, dispatch, runtime)
}
