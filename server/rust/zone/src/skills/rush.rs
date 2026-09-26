//! Прямые рывки Rush/Rush2 и их оглушающие состояния (0x73/0x7C).
//! Источник: `gameserver.exe` (SHA-256 `4F5C98E0…`) + `GameServer.pdb`
//! (RSDS match), `appserver/skills/rush.cpp` и общий предметный контракт
//! `rush2.cpp`. Машинно установлено: Rush — MP до RP с частичным расходом,
//! weapon category 1 (GS0287), GetTargetPathWithLength, distance-гейты
//! `> max` Rush / `>= max` Rush2, snapshot типов `{400,600,601,602}`,
//! first-attack контроллер только у Rush, уровни movzx → scaled keep
//! (query 10002 / линейный scale с clamp), AddRushState общий (prev по
//! ID → End + освобождение слота → Begin → append → knockback); Rush2 —
//! безусловный virtual `OnBeenAttacked(false)` хвост после состояния и
//! отбрасывания; visual Rush/Rush2 — вторая remap-таблица байт
//! `{0→act1+dir, 1→act2, 2/7/8/13/14 персональные}`, GS-строки
//! 0278/0287/0288/0289/0302 байт-сверены; тела RushState/RushState2 —
//! разделяемый алиас `BlindState<ID>`, payload 8 байт.
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
//!
//! Отдельный ID состояния сохраняется в арене и codec; OnAction рывных
//! состояний пустой, поэтому Defense не снимает состояние, в отличие от
//! собственно Blind.
//!
//! Объявленные швы — переходные фасады `DashSkillGame`/`DashSkillContact`
//! (`skills/dash.rs`, реализация у прежнего владельца); расстояние до точки
//! удара — общий `regions::shape::real_distance_between_points` (прежний
//! `skills/baseattack.rs::real_distance` делегировал туда же, путь нормализован);
//! ForceMove применяет существующий фасад `CGame` (`gameserver/game/rush.rs`),
//! развёрнутый результат отбрасывается, как и раньше; Begin состояния —
//! прежний общий `begin_primary_blind_state` семейства Blind. Оставшиеся
//! UNKNOWN/PARTIAL разведки: второй lock RushState::Begin (fight-lock по
//! форме), потребители raw CAN `available` — честно сохранены;
//! `get_tile_x/y` геттеры — сквозная согласованность подтверждена.
//!
//! Сознательное отклонение от native-UB (сохранено со старого файла): AI
//! держит impact только в локальных переменных первого входа — повторный вход
//! с condition1 читал бы неинициализированную клетку; она не создаётся из
//! нулей, повторный вход завершается безопасным отказом.

use nebokrai_shared::runtime::get_line_direction;

use crate::app::game_message::CMessage;
use crate::combat::{AttackInformation, truncate_original};
use crate::content::CSkillBaseProperties;
use crate::effects::{BLIND_STATE_BYTES, BlindState};
use crate::regions::ShapeIdentity;
use crate::regions::serverregion::geometry::PLAYER_TYPE;
use crate::regions::shape::{CShape, ShapeAreaCoordinates, real_distance_between_points};

use super::baseattackruntime::SKILL_USAGE_REUSE_DELAY_TIME;
use super::dash::{
    DashSkillContact, DashSkillExecutionOutcome, DashSkillGame, DashSkillMoveShape, DashSkillPlayer,
};
use super::execution::{RegisteredSkillRecord, RushExecutionState};
use super::flash::master_info;
use super::pillar::PILLAR_SKILL_ID;
use super::skill_is_restored;
use super::skillfactory::SkillOwner;
use super::visualeffect::SkillVisualEffectKind;
use super::{PlayerSkillDispatch, SkillStage};

pub const RUSH_SKILL_ID: u32 = 0x73;
pub const RUSH_2_SKILL_ID: u32 = 0x7c;
pub const RUSH_STATE_ID: u32 = 0x73;
pub const RUSH_2_STATE_ID: u32 = 0x7c;
pub const RUSH_STATE_BYTES: usize = BLIND_STATE_BYTES;
pub const RUSH_2_STATE_BYTES: usize = BLIND_STATE_BYTES;
pub type RushState = BlindState<RUSH_STATE_ID>;
pub type Rush2State = BlindState<RUSH_2_STATE_ID>;

const EFFECT_MESSAGE: i32 = 0x000b_fe01;
const USER_MP_LOSE: u32 = 2;
const USER_RP_LOSE: u32 = 3;
const TARGET_MAX_DISTANCE: u32 = 5_003;
const STATE_PERSIST_TIME: u32 = 10_002;
const TARGET_BACK_STEP: u32 = 1_001;
const TARGET_MOVE_SPEED: u32 = 2_001;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;

/// Различение visual-ресурса двух рывков по ID: общий вход знает оба владельца.
pub fn visual_kind(skill_id: u32) -> SkillVisualEffectKind {
    if skill_id == RUSH_SKILL_ID { SkillVisualEffectKind::Rush } else { SkillVisualEffectKind::Rush2 }
}

pub fn publish_rush_visual<Game: DashSkillGame>(
    game: &Game,
    skill: &RegisteredSkillRecord<Game::MonsterExecution>,
    mode: u32,
) {
    if !matches!(skill.owner(), SkillOwner::CRush | SkillOwner::CRush2)
        || skill.visual_effect().is_none_or(|effect| effect.kind() != visual_kind(skill.id()) || effect.is_ended())
    { return; }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = game.resolve_state_move_shape(region, identity) else { return; };
    let source = user.shape();
    let identity = source.identity();
    let mut message = CMessage::new(EFFECT_MESSAGE);
    if matches!(mode, 2 | 7 | 8 | 13 | 14) {
        if identity.object_type == PLAYER_TYPE {
            message.add_byte(0);
            message.add_byte(mode as u8);
            game.send_dash_visual_to_player(identity.id, &message);
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
        let target = game.resolve_skill_sufferer(skill.lifecycle());
        message.add_long(target.map_or(0, |(_, target)| target.object_type));
        message.add_long(target.map_or(0, |(_, target)| target.id));
        let x = source.get_tile_x().unwrap_or(i32::MIN);
        let y = source.get_tile_y().unwrap_or(i32::MIN);
        message.add_long(x);
        message.add_long(y);
    }
    game.send_dash_visual_around(source.get_region_id(), source, &message);
}

/// Предикат меча рывков (category 1), отличный от меча Flash (category 2).
fn weapon_is_valid<Game: DashSkillGame>(game: &Game, player: &Game::Player) -> bool {
    game.player_weapon_addon_category(player).is_some_and(|category| category == 1)
}

fn failure<Game: DashSkillGame>(game: &mut Game, instance: Game::SkillAddress, player_id: i32, code: u32) {
    game.update_registered_skill_visual(instance, code);
    let text: &[u8] = match code { 2 => b"GS0302", 13 => b"GS0278", 14 => b"GS0287", _ => return };
    game.send_skill_system_info(player_id, text);
}

fn resource_failure<Game: DashSkillGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    player_id: i32,
    properties: &CSkillBaseProperties,
    usage: u32,
) {
    let (code, text): (u32, &[u8]) = if usage == USER_MP_LOSE { (7, b"GS0288") } else { (8, b"GS0289") };
    game.update_registered_skill_visual(instance, code);
    let amount = properties.query_property(usage);
    game.send_skill_system_info_with_unsigned(player_id, text, amount);
}

pub fn scaled_state_time(source_level: u8, target_level: u8, base_time: u32) -> u32 {
    if u32::from(source_level) + 5 >= u32::from(target_level) { return base_time; }
    let difference = i32::from(target_level) - i32::from(source_level) - 5;
    let factor = (1.0_f32 - difference as f32 * 0.25).max(0.0);
    truncate_original(f64::from(base_time) * f64::from(factor)) as u32
}

pub(super) fn prepare_rush_control<Game: DashSkillGame>(
    game: &Game,
    instance: Game::SkillAddress,
    user: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity),
) -> Option<(CSkillBaseProperties, i32)> {
    game.resolve_state_move_shape(user.0, user.1)?;
    let target_shape = game.resolve_state_move_shape(target.0, target.1)?;
    if !game.live_skill_target_attackable(target_shape.shape().get_region_id(), user.1, target.1) {
        return None;
    }
    let skill = game.registered_skill(instance)?;
    let properties = game.skill_base_properties(skill.id(), skill.level())?.clone();
    let source = game.resolve_state_move_shape(user.0, user.1)?.shape();
    if !source.is_assigned_to_server_region() { return None; }
    game.dash_region_size(source.get_region_id())?;
    Some((properties, source.get_region_id()))
}

pub(super) fn remove_previous_rush_state<Game: DashSkillGame>(
    game: &mut Game,
    target: (i32, ShapeIdentity),
    skill_id: u32,
) {
    if let Some(index) = game.move_shape_state_position(target.0, target.1, skill_id) {
        game.end_move_shape_state_at(target.0, target.1, index);
    }
}

pub(super) fn rush_knockback<Game: DashSkillGame>(
    game: &mut Game,
    properties: &CSkillBaseProperties,
    source_region: i32,
    user: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity),
) {
    if properties.query_property(TARGET_BACK_STEP) == 0 { return; }
    let Some(target_shape) = game.resolve_state_move_shape(target.0, target.1) else { return; };
    let target_y = target_shape.shape().get_tile_y().unwrap_or(i32::MIN);
    let target_x = target_shape.shape().get_tile_x().unwrap_or(i32::MIN);
    let Some(source) = game.resolve_state_move_shape(user.0, user.1) else { return; };
    let source_y = source.shape().get_tile_y().unwrap_or(i32::MIN);
    let source_x = source.shape().get_tile_x().unwrap_or(i32::MIN);
    let direction = get_line_direction(source_x, source_y, target_x, target_y);
    let Some(target_shape) = game.resolve_state_move_shape(target.0, target.1) else { return; };
    let x = target_shape.shape().get_tile_x().unwrap_or(i32::MIN);
    let y = target_shape.shape().get_tile_y().unwrap_or(i32::MIN);
    let mut position = ShapeAreaCoordinates { x, y };
    let Some((region_width, region_height)) = game.dash_region_size(source_region) else { return; };
    let mut steps = properties.query_property(TARGET_BACK_STEP);
    let mut moved = 0u32;
    while moved < steps {
        let Ok(next) = CShape::get_direction_position(direction, position) else { break; };
        if next.x < 0 || next.y < 0 || next.x >= region_width || next.y >= region_height
            || game.dash_skill_cell_block(source_region, next.x, next.y).unwrap_or(1) & 7 != 0 { break; }
        position = next;
        moved = moved.wrapping_add(1);
        steps = properties.query_property(TARGET_BACK_STEP);
    }
    let duration = properties.query_property(TARGET_MOVE_SPEED).wrapping_mul(moved);
    // При ненулевом BACK_STEP ForceMove вызывается и после нуля свободных шагов.
    game.force_move_skill_target(target.0, target.1, position.x, position.y, duration);
}

fn add_rush_state<Game: DashSkillGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    user: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity),
    keep: u32,
    now_milliseconds: fn() -> u32,
) {
    let Some((properties, source_region)) = prepare_rush_control(game, instance, user, target) else { return; };
    let state = RushState::new(keep);
    remove_previous_rush_state(game, target, RUSH_STATE_ID);
    let _ = game.begin_rush_state(target, Some(user), state, &mut || now_milliseconds());
    rush_knockback(game, &properties, source_region, user, target);
}

/// Хвост второго рывка Rush2 (0x7C): после состояния и отбрасывания создаётся
/// отдельная атака без компонент урона. Она сохраняет конструкторский skill-id,
/// получает свежие сведения об источнике и проходит обычный OnBeenAttacked,
/// даже если Begin или ForceMove не выполнились.
pub(super) fn add_rush_2_state<Game, Runtime>(
    game: &mut Game,
    address: Game::SkillAddress,
    user: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity),
    keep: u32,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) where
    Game: DashSkillGame + DashSkillContact<Runtime>,
{
    let Some((properties, source_region)) = prepare_rush_control(game, address, user, target) else { return; };
    let state = Rush2State::new(keep);
    remove_previous_rush_state(game, target, RUSH_2_STATE_ID);
    let _ = game.begin_rush_2_state(target, Some(user), state, &mut || now_milliseconds());
    rush_knockback(game, &properties, source_region, user, target);
    let Some(source) = game.find_player(user.1.id).filter(|_| user.1.object_type == PLAYER_TYPE) else { return; };
    let master = master_info(source);
    let contact = AttackInformation::for_master(master);
    game.apply_owned_skill_contact(master, target.1, target.0, contact, runtime);
}

pub fn check_cast<Game: DashSkillGame>(
    game: &mut Game,
    instance: Game::SkillAddress,
    player_id: i32,
    now_milliseconds: fn() -> u32,
) -> bool {
    let Some(player) = game.find_player(player_id) else { return false; };
    let Some(skill) = game.registered_skill(instance) else { return false; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return false; };
    let reuse = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if !skill_is_restored(skill.last_used_ms(), reuse, now_milliseconds()) {
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
    if player.has_state_by_skill_id(PILLAR_SKILL_ID) {
        failure(game, instance, player_id, 2);
        return false;
    }
    let Some(player) = game.find_player_mut(player_id) else { return false; };
    player.set_skill_moveable(false);
    true
}

pub fn run_ai<Game, Runtime>(
    game: &mut Game,
    instance: Game::SkillAddress,
    runtime: &mut Runtime,
    now_milliseconds: fn() -> u32,
) -> DashSkillExecutionOutcome
where
    Game: DashSkillGame + DashSkillContact<Runtime>,
{
    let Some(skill) = game.registered_skill(instance) else { return DashSkillExecutionOutcome::Rejected; };
    let Some(stage) = skill.execution_stage().filter(|stage| *stage != SkillStage::Idle) else {
        return DashSkillExecutionOutcome::Pending;
    };
    let skill_id = skill.id();
    let Some(properties) = game.skill_base_properties(skill_id, skill.level()).cloned() else { return DashSkillExecutionOutcome::Rejected; };
    let (region_id, identity) = skill.lifecycle().user();
    let Some(source) = game.resolve_state_move_shape(region_id, identity) else { return DashSkillExecutionOutcome::Rejected; };
    if !source.shape().is_assigned_to_server_region() { return DashSkillExecutionOutcome::Rejected; }
    let user = (source.shape().get_region_id(), source.shape().identity());
    // Native хранит impact только в локальных переменных первого AI: повторный
    // вход с condition1 читает неинициализированную клетку. Не создаём её из нулей.
    if stage != SkillStage::Begin { return DashSkillExecutionOutcome::Rejected; }
    let mut impact = match game.resolve_skill_sufferer(skill.lifecycle()) {
        Some((region, identity)) => {
            let Some(target) = game.resolve_state_move_shape(region, identity) else { return DashSkillExecutionOutcome::Rejected; };
            (target.shape().get_tile_x().unwrap_or(i32::MIN), target.shape().get_tile_y().unwrap_or(i32::MIN))
        }
        None => skill.lifecycle().destination(),
    };
    let Some(player) = game.find_player(user.1.id).filter(|_| user.1.object_type == PLAYER_TYPE) else { return DashSkillExecutionOutcome::Rejected; };
    let mana = player.mana();
    let remaining = mana.wrapping_sub(properties.query_property(USER_MP_LOSE));
    if (remaining as i32) < 0 {
        resource_failure(game, instance, user.1.id, &properties, USER_MP_LOSE);
        return DashSkillExecutionOutcome::Rejected;
    }
    if let Some(player) = game.find_player_mut(user.1.id) { player.set_mana(remaining); }
    let Some(player) = game.find_player(user.1.id) else { return DashSkillExecutionOutcome::Rejected; };
    let rp = u32::from(player.rp());
    let remaining = rp.wrapping_sub(properties.query_property(USER_RP_LOSE));
    if (remaining as i32) < 0 {
        resource_failure(game, instance, user.1.id, &properties, USER_RP_LOSE);
        return DashSkillExecutionOutcome::Rejected;
    }
    if let Some(player) = game.find_player_mut(user.1.id) { player.set_rp(remaining as u16); }
    game.publish_player_states(user.1.id);
    if game.find_player(user.1.id).is_none_or(|player| !weapon_is_valid(game, player)) {
        failure(game, instance, user.1.id, 14);
        return DashSkillExecutionOutcome::Rejected;
    }
    let can_break = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if let Some(skill) = game.registered_skill_mut(instance) { skill.lifecycle_mut().set_available(can_break != 0); }
    let Some(source) = game.resolve_state_move_shape(user.0, user.1) else { return DashSkillExecutionOutcome::Rejected; };
    let source_y = source.shape().get_tile_y().unwrap_or(i32::MIN);
    let source_x = source.shape().get_tile_x().unwrap_or(i32::MIN);
    if let Some(source) = game.resolve_state_move_shape_mut(user.0, user.1) {
        source.shape_mut().set_direction(get_line_direction(source_x, source_y, impact.0, impact.1));
    }
    let Some(source) = game.resolve_state_move_shape(user.0, user.1) else { return DashSkillExecutionOutcome::Rejected; };
    let mut destination = (source.shape().get_tile_x().unwrap_or(i32::MIN), source.shape().get_tile_y().unwrap_or(i32::MIN));
    let maximum = properties.query_property(TARGET_MAX_DISTANCE);
    let Some(skill) = game.registered_skill(instance) else { return DashSkillExecutionOutcome::Rejected; };
    let path = game.skill_target_path_with_length(skill.lifecycle(), maximum);
    let Some(state) = game.registered_skill_mut(instance).and_then(|skill| skill.player_state_mut::<RushExecutionState>()) else { return DashSkillExecutionOutcome::Rejected; };
    state.path = path;
    if state.path.is_empty() { return DashSkillExecutionOutcome::Rejected; }
    for &(x, y, block) in &state.path {
        if block != 0 { impact = (x, y); break; }
        destination = (x, y);
    }
    game.set_player_tile_position(user.1.id, destination.0, destination.1);
    game.update_registered_skill_visual(instance, 0);
    if let Some(skill) = game.registered_skill_mut(instance) { let _ = skill.advance_execution(SkillStage::Begin, SkillStage::Check); }
    game.update_registered_skill_visual(instance, 1);
    let Some(source) = game.resolve_state_move_shape(user.0, user.1) else { return DashSkillExecutionOutcome::Rejected; };
    let source_shape = source.shape();
    let region_id = source_shape.get_region_id();
    if !source_shape.is_assigned_to_server_region() || game.dash_region_size(region_id).is_none() {
        game.update_registered_skill_visual(instance, 2);
        return DashSkillExecutionOutcome::Rejected;
    }
    let x = source_shape.get_tile_x().unwrap_or(i32::MIN);
    let y = source_shape.get_tile_y().unwrap_or(i32::MIN);
    let distance = real_distance_between_points(x, y, impact.0, impact.1) as u32;
    let maximum = properties.query_property(TARGET_MAX_DISTANCE);
    if if skill_id == RUSH_SKILL_ID { distance > maximum } else { distance >= maximum } {
        game.update_registered_skill_visual(instance, 2);
        return DashSkillExecutionOutcome::Rejected;
    }
    for view in game.dash_cell_views(region_id, impact.0, impact.1) {
        let identity = view.identity;
        if !matches!(identity.object_type, 400 | 600 | 601 | 602) { continue; }
        let Some(target_shape) = game.resolve_state_move_shape(region_id, identity) else { continue; };
        let Some(source) = game.resolve_state_move_shape(user.0, user.1) else { continue; };
        if std::ptr::eq(source, target_shape) { continue; }
        let target = (target_shape.shape().get_region_id(), identity);
        if game.base_magic_target_dead(target.0, target.1)
            || !game.live_skill_target_attackable(target.0, user.1, target.1) { continue; }
        if skill_id == RUSH_SKILL_ID
            && let Some(controller) = game.skill_target_controller(target.0, target.1)
            && controller != user.1.id
        {
            let Some(source) = game.resolve_state_move_shape(user.0, user.1) else { continue; };
            let y = source.shape().get_tile_y().unwrap_or(i32::MIN);
            let x = source.shape().get_tile_x().unwrap_or(i32::MIN);
            game.rush_first_attack_at_position(user.1.id, controller, Some(region_id), (x, y), runtime);
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
            add_rush_state(game, instance, user, target, keep, now_milliseconds);
        } else {
            add_rush_2_state(game, instance, user, target, keep, runtime, now_milliseconds);
        }
    }
    DashSkillExecutionOutcome::Completed
}

pub const fn is_rush_2_dispatch(dispatch: PlayerSkillDispatch) -> bool {
    dispatch.skill_id() == RUSH_2_SKILL_ID
}
