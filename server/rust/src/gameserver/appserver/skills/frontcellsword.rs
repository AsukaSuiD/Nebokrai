//! Выбор целей и контакт фронтальных ударов мечом.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/jucut.cpp,
//! lightningsword[2/3/4].cpp и inversechopped.cpp.
//!
//! JuCut и четыре LightningSword выбирают первый CMoveShape клетки и
//! останавливаются независимо от допуска, смерти и результата Attack.
//! InverseChopped перебирает все CMoveShape, пропуская себя до допуска;
//! дедупликации нет. Проверка смерти внутри Attack предшествует PK-seed,
//! свежему Calculate и сырому OnBeenAttacked. Общая формула и различие RP
//! принадлежат weaponattack. InverseChopped после сканирования завершает
//! первое оставшееся EnergyHolding даже при NULL регионе или пустой клетке.
//! Vec заменяет временный native vector одной клетки; execution-state,
//! visual и End остаются у зарегистрированного caller-а. GetFacePos и
//! GetDirPos имеют одинаковые восемь смещений. Для направления вне таблицы
//! безопасная модель не воспроизводит native чтение за её пределами.

use super::energyholdingstate::clear_energy_holding;
use super::flash::cell_views;
use super::weaponattack::apply_front_cell_weapon_attack;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

fn front_cell(
    game: &CGame, source: (i32, ShapeIdentity), inverse: bool,
) -> Option<(i32, ShapeAreaCoordinates)> {
    let shape = resolve_state_move_shape(game, source.0, source.1)?.shape();
    if inverse && (!shape.is_assigned_to_server_region()
        || game.find_region(shape.get_region_id()).is_none()) { return None; }
    let position = ShapeAreaCoordinates {
        x: shape.get_tile_x().unwrap_or(i32::MIN),
        y: shape.get_tile_y().unwrap_or(i32::MIN),
    };
    let face = CShape::get_direction_position(shape.get_direction(), position).ok()?;
    if !shape.is_assigned_to_server_region() { return None; }
    let region_id = shape.get_region_id();
    game.find_region(region_id)?;
    Some((region_id, face))
}

fn attack_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), inverse: bool, runtime: &mut Runtime,
) {
    if source.1 == target.1
        || !game.move_shape_health(target.0, target.1).is_some_and(|health| health != 0)
    { return; }
    apply_front_cell_weapon_attack(game, instance, source, target, inverse, runtime);
}

pub(super) fn run_front_cell_sword_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    inverse: bool, runtime: &mut Runtime,
) {
    if let Some((region_id, face)) = front_cell(game, source, inverse) {
        for view in cell_views(game, region_id, face.x, face.y) {
            let Some(sufferer) = resolve_state_move_shape(game, region_id, view.identity) else { continue; };
            let target = (sufferer.shape().get_region_id(), sufferer.shape().identity());
            if inverse && target.1 == source.1 { continue; }
            if game.live_skill_target_attackable(target.0, source.1, target.1) {
                attack_target(game, instance, source, target, inverse, runtime);
            }
            if !inverse { break; }
        }
    }
    if inverse { clear_energy_holding(game, source); }
}
