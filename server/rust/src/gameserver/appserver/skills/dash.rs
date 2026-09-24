//! Общая геометрия, визуальный формат и контактная атака Flash/LittleFlash.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/flash.cpp,
//! littleflash.cpp и littleflash2.cpp.
//!
//! Путь и список поражённых целей принадлежат concrete владельцам и не
//! копируются через callback атаки. Общая обработка пути сохраняет блоки
//! GetTargetPath, проверяет первую фигуру клетки и выбирает выход сначала
//! среди восьми соседей, затем через существующий CRegion random-поиск.
//! LittleFlash2 не требует занятую клетку и очищает одиночный закрытый выход;
//! поклеточный удар и live IsAttackAble остаются у вызывающего AI.
//! Единый формат visual передаёт последнюю клетку подготовленного пути;
//! безусловный базовый callback остаётся у зарегистрированного ресурса.
//!
//! Контакт использует общий оружейный расчёт с коэффициентом TARGET_DAMAGE_FACTOR.

use super::weaponattack::apply_player_weapon_attack;
use crate::gameserver::appserver::citygate::CITY_GATE_OBJECT_TYPE;
use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::appserver::states::visualeffect::SkillVisualEffectKind;
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, RegionShapeResolver,
};
use crate::public::tools::get_line_direction;
use crate::nets::netserver::message::CMessage;
use crate::nets::netserver::message::GameMessageDomainOps;

const TARGET_DAMAGE_FACTOR: u32 = 20_003;

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

pub(super) fn apply_dash_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    apply_player_weapon_attack(game, instance, source, target, TARGET_DAMAGE_FACTOR, runtime);
}
