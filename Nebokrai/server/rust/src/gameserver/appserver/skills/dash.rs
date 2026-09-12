//! Общий вход и контактная атака рывков Flash, LittleFlash и LittleFlash2.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/flash.cpp,
//! littleflash.cpp и littleflash2.cpp, базовый appserver/states/attackskill.cpp.
//!
//! Begin, материализация, visual и End работают с одним поколенческим ключом.
//! Проверка видит concrete payload с выключенной фазой; успешный Begin включает
//! его без второго отсчёта времени. Общий End выполняется до возврата результата
//! расписанию, которое только освобождает payload и соответствующую команду.
//! Путь и список поражённых целей принадлежат concrete владельцам и не
//! копируются через callback атаки. Общая обработка пути сохраняет блоки
//! GetTargetPath, проверяет первую фигуру клетки и выбирает выход сначала
//! среди восьми соседей, затем через существующий CRegion random-поиск.
//! LittleFlash2 не требует занятую клетку и очищает одиночный закрытый выход;
//! поклеточный удар и live IsAttackAble остаются у вызывающего AI.
//! Единый формат visual передаёт последнюю клетку подготовленного пути;
//! безусловный базовый callback остаётся у зарегистрированного ресурса.
//!
//! Все три достигнутых входа принадлежат CPlayer. Attack сохраняет его PK-флаги
//! и принадлежность до Calculate, доставляет сырой OnBeenAttacked без повторного
//! допуска, затем вызывает IncreaseRp независимо от результата получателя.
//! NULL таблица Calculate оставляет исходный UNKNOWN/1, но не отменяет удар.
//! Формула сохраняет порядок живых getter-ов и обоих RNG, unsigned коэффициент
//! в x87-цепочке до записи float и усечение критического множителя к нулю.
//! Vec заменяет только техническое владение записями повреждений.

use super::fightdefense::truncate_original;
use super::flash::master_info;
use super::kernel::{PlayerSkillExecution, SkillStage, SkillTermination};
use super::stateskill::state_skill_outcome;
use crate::gameserver::appserver::citygate::CITY_GATE_OBJECT_TYPE;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::player::PlayerSkillDispatch;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::appserver::states::visualeffect::{SkillVisualEffect, SkillVisualEffectKind};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, QueuedSkillExecutionOutcome, QueuedSkillExecutionState, RegionShapeResolver,
};
use crate::public::tools::get_line_direction;
use crate::nets::netserver::message::CMessage;

const TARGET_DAMAGE_FACTOR: u32 = 20_003;
const USER_HIT_MODIFIER: u32 = 20_001;

pub(super) fn publish_dash_visual(
    game: &CGame, skill: &MoveShapeSkill, mode: u32, kind: SkillVisualEffectKind,
    destination: Option<(i32, i32)>,
) {
    if skill.visual_effect().is_none_or(|effect| effect.kind() != kind || effect.is_ended()) {
        return;
    }
    let (region, identity) = skill.lifecycle().user();
    let Some(user) = resolve_state_move_shape(game, region, identity) else { return; };
    let source = user.shape();
    let mut message = CMessage::new(0x000b_fe01);
    if matches!(mode, 2 | 4 | 7 | 8 | 10 | 11 | 13 | 14 | 15) {
        if source.identity().object_type == 400 {
            message.add_byte(0);
            message.add_byte(mode as u8);
            let _ = message.send_to_player(game.net_server(), source.identity().id);
        }
        return;
    }
    let action = match mode { 0 => 1, 1 => 2, 3 => 3, _ => return };
    message.add_byte(action);
    message.add_long(skill.id() as i32);
    message.add_short(skill.level() as i16);
    message.add_long(source.identity().object_type);
    message.add_long(source.identity().id);
    if mode == 1 {
        let Some((x, y)) = destination else { return; };
        message.add_long(0);
        message.add_long(0);
        message.add_long(x);
        message.add_long(y);
    } else {
        message.add_long(source.get_direction());
    }
    if let Some(owner) = game.find_region(source.get_region_id()) {
        let _ = game.send_game_shape_around(owner.base(), source, None, &message);
    }
}

fn finish_outcome<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, outcome: QueuedSkillExecutionOutcome,
    runtime: &mut Runtime,
) -> QueuedSkillExecutionOutcome {
    let end = match outcome.state {
        QueuedSkillExecutionState::Rejected => Some((0, SkillTermination::Rejected)),
        QueuedSkillExecutionState::Completed | QueuedSkillExecutionState::RejectedAfterUse => {
            Some((1, SkillTermination::Completed))
        }
        QueuedSkillExecutionState::Begun | QueuedSkillExecutionState::Pending => None,
    };
    if let Some((argument, termination)) = end {
        let _ = game.end_registered_instance(instance, argument, termination, runtime);
    }
    outcome
}

pub(super) fn execute_registered_dash<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, player_id: i32, instance: RegisteredSkill,
    dispatch: PlayerSkillDispatch, runtime: &mut Runtime, visual_kind: SkillVisualEffectKind,
    check: impl FnOnce(&mut CGame, RegisteredSkill, i32, &mut Runtime) -> bool,
    materialize: impl FnOnce(PlayerSkillDispatch, u32) -> PlayerSkillExecution,
    run_ai: impl FnOnce(&mut CGame, RegisteredSkill, &mut Runtime) -> QueuedSkillExecutionOutcome,
) -> QueuedSkillExecutionOutcome {
    let Some(skill) = game.registered_skill(instance) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    if skill.id() != dispatch.skill_id() {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    }
    if let Some(previous) = skill.player_dispatch() {
        if previous != dispatch {
            return state_skill_outcome(QueuedSkillExecutionState::Rejected);
        }
        let outcome = run_ai(game, instance, runtime);
        return finish_outcome(game, instance, outcome, runtime);
    }
    if !game.begin_registered_player_skill_with_combat(
        instance, player_id, dispatch, runtime.now_milliseconds(),
    ) {
        return finish_outcome(game, instance, state_skill_outcome(QueuedSkillExecutionState::Rejected), runtime);
    }
    let Some(skill) = game.registered_skill_mut(instance) else {
        return state_skill_outcome(QueuedSkillExecutionState::Rejected);
    };
    skill.replace_visual_effect(SkillVisualEffect::new(visual_kind, 1));
    let mut execution = materialize(dispatch, skill.lifecycle().started_at_ms());
    execution.kernel_mut().clear_phase_for_end();
    if !skill.install_player_execution(execution) {
        return finish_outcome(game, instance, state_skill_outcome(QueuedSkillExecutionState::Rejected), runtime);
    }
    if !check(game, instance, player_id, runtime) {
        return finish_outcome(game, instance, state_skill_outcome(QueuedSkillExecutionState::Rejected), runtime);
    }
    if let Some(skill) = game.registered_skill_mut(instance) {
        let _ = skill.advance_execution(SkillStage::Idle, SkillStage::Begin);
    }
    state_skill_outcome(QueuedSkillExecutionState::Begun)
}

pub(super) fn check_dash_path<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, source: (i32, ShapeIdentity), mut path: Vec<(i32, i32, u8)>,
    maximum: u32, require_shape_block: bool, clear_single_blocked: bool, runtime: &mut Runtime,
) -> Vec<(i32, i32, u8)> {
    if path.is_empty() { return path; }
    let Some(shape) = resolve_state_move_shape(game, source.0, source.1).map(|shape| shape.shape()) else {
        return Vec::new();
    };
    if !shape.is_assigned_to_server_region() { return Vec::new(); }
    let Some(owner) = game.find_region(shape.get_region_id()) else { return Vec::new(); };
    let region = owner.base();
    let (source_x, source_y) = (shape.get_tile_x().unwrap_or(i32::MIN), shape.get_tile_y().unwrap_or(i32::MIN));
    if path.first().is_some_and(|cell| cell.0 == source_x && cell.1 == source_y) { path.remove(0); }
    path.truncate(maximum as usize);
    // Native обращается к back() даже после подрезки до нуля. Пустой путь
    // остаётся безопасным отказом без выдуманной клетки назначения.
    let Some(last) = path.last().copied() else { return path; };
    if let Ok(next) = CShape::get_direction_position(
        get_line_direction(source_x, source_y, last.0, last.1),
        ShapeAreaCoordinates { x: last.0, y: last.1 },
    ) {
        path.push((next.x, next.y, region.skill_cell_block(next.x, next.y)));
    }
    let (area_width, area_height) = game.area_dimensions();
    let resolver = RegionShapeResolver { game, owner };
    let mut saw_shape_block = false;
    let mut trim_index = path.len();
    for (index, cell) in path.iter().enumerate() {
        let city_gate = region.get_shape(cell.0, cell.1, area_width, area_height, &resolver)
            .ok().flatten().is_some_and(|shape| shape.identity.object_type == CITY_GATE_OBJECT_TYPE as i32);
        if city_gate || matches!(cell.2, 1 | 2) {
            trim_index = index.saturating_sub(1);
            break;
        }
        if !saw_shape_block { saw_shape_block = cell.2 == 3; }
        else if cell.2 != 3 { trim_index = index; break; }
    }
    if require_shape_block && !saw_shape_block { return Vec::new(); }
    if trim_index == path.len() { trim_index = path.len().saturating_sub(1); }
    path.truncate(trim_index.saturating_add(1));
    let Some(anchor) = path.last().copied() else { return path; };
    if anchor.2 != 0 {
        if clear_single_blocked && path.len() == 1 {
            path.clear();
            return path;
        }
        for direction in 0..8 {
            let Ok(candidate) = CShape::get_direction_position(
                direction, ShapeAreaCoordinates { x: anchor.0, y: anchor.1 },
            ) else { continue; };
            if candidate.x >= 0 && candidate.y >= 0 && candidate.x < region.region.width
                && candidate.y < region.region.height
                && region.skill_cell_block(candidate.x, candidate.y) & 7 == 0
            {
                path.push((candidate.x, candidate.y, 0));
                return path;
            }
        }
        if let Ok(candidate) = region.region.get_random_pos_in_range(
            anchor.0.wrapping_sub(2), anchor.1.wrapping_sub(2), 5, 5, runtime,
        ) {
            path.push((candidate.x, candidate.y, 0));
        }
    }
    path
}

fn calculate_dash_attack(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), attack: &mut AttackInformation,
) {
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()) else { return; };
    attack.skill_id = skill.id();
    attack.skill_level = skill.level() as u8;
    attack.damage_modifier = 0;
    let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
    let Some(player) = game.find_player(source.1.id) else { return; };
    let (divisor, minimum_factor) = game.globe_setup().weapon_damage_factors();
    let weapon_factor = player.weapon_modifier(
        game.goods_factory(), i32::from(target_level), divisor, minimum_factor,
    );
    let damage_factor = properties.query_property(TARGET_DAMAGE_FACTOR);
    attack.damage_factor =
        (f64::from(damage_factor) * f64::from(weapon_factor) * f64::from(0.01_f32)) as f32;
    attack.hit_modifier = properties.query_property(USER_HIT_MODIFIER) as i32;
    let maximum = player.combat_properties().maximum_attack;
    let minimum = player.combat_properties().minimum_attack;
    let width = (maximum as i32).wrapping_sub(minimum as i32).wrapping_abs().wrapping_add(1);
    let random = game.skill_random_below(width);
    let Some(player) = game.find_player(source.1.id) else { return; };
    let physical = (player.combat_properties().minimum_attack as i32).wrapping_add(random).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 });
    let element = (player.combat_properties().add_element_attack as i32).max(0);
    attack.damages.push(AttackPower { kind: AttackPowerType::Element, hp_damage: element, mp_damage: 0 });
    let soul = i32::from(player.combat_properties().add_soul_attack);
    attack.damages.push(AttackPower { kind: AttackPowerType::Soul, hp_damage: soul, mp_damage: 0 });
    let critical_chance = player.combat_properties().cch;
    if game.skill_random_below(100) < i32::from(critical_chance) {
        attack.critical = true;
        let rate = game.globe_setup().critical_rate();
        for power in &mut attack.damages {
            power.hp_damage = truncate_original(f64::from(power.hp_damage) * f64::from(rate));
        }
    }
}

pub(super) fn apply_dash_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    let Some(master) = game.find_player(source.1.id).map(master_info) else { return; };
    let mut attack = AttackInformation::for_master(master);
    calculate_dash_attack(game, instance, source, target, &mut attack);
    game.apply_owned_skill_contact(master, target.1, target.0, attack, runtime);
    game.increase_owned_player_rp(source.1.id, true, 0);
}
