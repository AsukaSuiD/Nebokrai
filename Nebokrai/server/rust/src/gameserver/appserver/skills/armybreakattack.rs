//! Поклеточный удар ArmyBreak и ArmyBreak2.
//! Источник: gameserver.exe/GameServer.pdb, skills/armybreak.cpp и armybreak2.cpp.
//!
//! Обе исходные маски заполнены целиком: 5×5 вокруг клетки перед U. Обход идёт
//! по X, затем Y; GetShapes создаёт отдельный снимок каждой достигнутой клетки.
//! Допускаются все CMoveShape, кроме U, без предварительного фильтра смерти.
//! Основная цель определяется её живыми координатами перед IsAttackAble.
//! Локальный Vec отмечает цель после Attack, а также при отказе IsAttackAble;
//! callback End не очищает этот список. Сам контакт и RP выполняет общий
//! оружейный расчёт с разными usage для основной и побочной целей.

use super::flash::cell_views;
use super::weaponattack::apply_player_weapon_attack;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

const SCOPE_SIDE: i32 = 5;
const MINOR_TARGET_DAMAGE_FACTOR: u32 = 20_011;
const MAJOR_TARGET_DAMAGE_FACTOR: u32 = 20_012;

pub(super) fn run_army_break_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    direction: i32, runtime: &mut Runtime,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let shape = user.shape();
    if !shape.is_assigned_to_server_region() { return; }
    let region_id = shape.get_region_id();
    if game.find_region(region_id).is_none() { return; }
    let user_identity = shape.identity();
    let position = ShapeAreaCoordinates {
        x: shape.get_tile_x().unwrap_or(i32::MIN),
        y: shape.get_tile_y().unwrap_or(i32::MIN),
    };
    // Исходный GetDirPos индексирует таблицу без проверки. Некорректный индекс
    // безопасно прекращает обход, не подставляя выдуманную центральную клетку.
    let Ok(front) = CShape::get_direction_position(direction, position) else { return; };
    let origin_x = front.x.wrapping_sub(SCOPE_SIDE / 2);
    let origin_y = front.y.wrapping_sub(SCOPE_SIDE / 2);
    let mut seen = Vec::new();
    for x in 0..SCOPE_SIDE {
        for y in 0..SCOPE_SIDE {
            for view in cell_views(game, region_id, origin_x.wrapping_add(x), origin_y.wrapping_add(y)) {
                let Some(target) = resolve_state_move_shape(game, region_id, view.identity) else { continue; };
                let target_shape = target.shape();
                let identity = target_shape.identity();
                if identity == user_identity || seen.contains(&identity) { continue; }
                let target_region = target_shape.get_region_id();
                let major = target_shape.get_tile_x().unwrap_or(i32::MIN) == front.x
                    && target_shape.get_tile_y().unwrap_or(i32::MIN) == front.y;
                if game.live_skill_target_attackable(target_region, source.1, identity) {
                    let usage = if major { MAJOR_TARGET_DAMAGE_FACTOR } else { MINOR_TARGET_DAMAGE_FACTOR };
                    apply_player_weapon_attack(game, instance, source, (target_region, identity), usage, runtime);
                }
                seen.push(identity);
            }
        }
    }
}
