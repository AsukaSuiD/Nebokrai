//! Зеркало душ `CSoulMirror` (`0x13C`).
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/soulmirror.cpp`.
//!
//! Общий ZonalCast хранит зарегистрированный Attack, его U/S, Check с
//! Player-only MP/Move0, unsigned срок start+delay и общий End. После visual1
//! этот owner захватывает текущие регион и центр U. Начало области остаётся
//! от этого момента, но перед каждой клеткой заново читаются level и
//! direction: GetScope задаёт фронтальную линию ширины `2 * level - 1` в
//! таблицах 3×3/5×5/7×7. Клетки идут X→Y и не собираются заранее, поэтому
//! синхронный контакт меняет следующий снимок. Любой разрешённый CMoveShape
//! делает клетку занятой; допуск, дедупликация и raw Attack относятся только
//! к подходящим целям. Пустая проходимая клетка создаёт CSummonedCreature с
//! fresh Master(country0), lifetime, направлением и picture. Формулу и raw
//! контакт сохраняет directelementattack: weapon factor, Player-only EM и
//! единственный RNG без damage modifier, RP, CCH и второго RNG.

use super::directelementattack::apply_direct_element_attack;
use super::flash::cell_views;
use super::lightingarrowphalanx::ArrowTargetIdentity;
use super::skillbaseproperties::CSkillBaseProperties;
use super::weaponattack::source_master;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) const SOUL_MIRROR_SKILL_ID: u32 = 0x13c;

const SUMMONED_LIFETIME: u32 = 30_001;
const SUMMONED_CREATURE_ID: u32 = 30_003;

const SCOPE_DIRECTIONS: [((i32, i32), (i32, i32)); 8] = [
    ((0, -1), (1, 0)), ((1, -1), (1, 1)), ((1, 0), (0, 1)), ((1, 1), (-1, 1)),
    ((0, 1), (1, 0)), ((-1, 1), (1, 1)), ((-1, 0), (0, 1)), ((-1, -1), (-1, 1)),
];

fn scope_size(level: i32) -> Option<i32> {
    match level {
        1 => Some(3),
        2 => Some(5),
        3 => Some(7),
        _ => None,
    }
}

fn current_scope_size(game: &CGame, instance: RegisteredSkill) -> Option<i32> {
    game.registered_skill(instance).and_then(|skill| scope_size(skill.level()))
}

fn scope_cell(level: i32, direction: i32, x: i32, y: i32) -> bool {
    let Some(size) = scope_size(level) else { return false; };
    if !(0..size).contains(&x) || !(0..size).contains(&y) { return false; }
    let Some(&(forward, tangent)) = SCOPE_DIRECTIONS.get(direction as usize) else {
        return false;
    };
    let center = level;
    let radius = center.wrapping_sub(1);
    (-radius..=radius).any(|offset| {
        x == center.wrapping_add(forward.0).wrapping_add(tangent.0.wrapping_mul(offset))
            && y == center.wrapping_add(forward.1).wrapping_add(tangent.1.wrapping_mul(offset))
    })
}

fn summon_empty_cell(
    game: &mut CGame,
    source: (i32, ShapeIdentity),
    region_id: i32,
    x: i32,
    y: i32,
    properties: &CSkillBaseProperties,
) {
    let free = game.find_region(region_id).is_some_and(|owner| {
        let region = owner.base();
        x >= 0 && x < region.region.width && y >= 0 && y < region.region.height
            && region.skill_cell_block(x, y) & 7 == 0
    });
    if !free { return; }

    let Some(mut master) = source_master(game, source) else { return; };
    master.master_country_id = 0;
    let lifetime = properties.query_property(SUMMONED_LIFETIME);
    let Some(direction) = resolve_state_move_shape(game, source.0, source.1)
        .map(|source| source.shape().get_direction())
    else {
        return;
    };
    let picture = properties.query_property(SUMMONED_CREATURE_ID);
    let Some(property) = game.find_monster_property_by_picture_id(picture).cloned() else {
        return;
    };
    let Some(mut owner) = game.take_region_owner(region_id) else { return; };
    let _ = game.add_summoned_creature_owned(
        owner.base_mut(), &property, master, x, y, direction, lifetime,
    );
    game.restore_region_owner(owner);
}

pub(super) fn apply_soul_mirror_area<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    instance: RegisteredSkill,
    source: (i32, ShapeIdentity),
    properties: &CSkillBaseProperties,
    runtime: &mut Runtime,
) {
    // visual1 уже мог вызвать произвольный код: здесь берутся live region и
    // центр U, но source для Attack остаётся результатом GetUser этого AI.
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else {
        return;
    };
    let user = user.shape();
    if !user.is_assigned_to_server_region() { return; }
    let region_id = user.get_region_id();
    let Some(initial_size) = current_scope_size(game, instance) else { return; };
    let center_x = user.get_tile_x().unwrap_or(i32::MIN);
    let center_y = user.get_tile_y().unwrap_or(i32::MIN);
    let start_x = center_x.wrapping_sub(initial_size >> 1);
    let start_y = center_y.wrapping_sub(initial_size >> 1);
    let mut attacked = Vec::<ArrowTargetIdentity>::new();
    let mut column = 0_i32;

    loop {
        // Условие внешнего цикла повторно читает this->level.
        let Some(width) = current_scope_size(game, instance) else { return; };
        if column >= width { break; }
        let mut row = 0_i32;
        loop {
            // Высота и GetScope не используют кэшированный level: callback
            // предыдущей клетки может завершить регистрацию или изменить его.
            let Some(height) = current_scope_size(game, instance) else { return; };
            if row >= height { break; }
            let Some(level) = game.registered_skill(instance).map(|skill| skill.level()) else {
                return;
            };
            let Some(direction) = resolve_state_move_shape(game, source.0, source.1)
                .map(|source| source.shape().get_direction())
            else {
                return;
            };
            let cell_x = start_x.wrapping_add(column);
            let cell_y = start_y.wrapping_add(row);
            if scope_cell(level, direction, column, row) {
                // Один resolver-снимок на клетку; следующий создаётся только
                // после всех callbacks текущей клетки.
                let mut occupied = false;
                for view in cell_views(game, region_id, cell_x, cell_y) {
                    let Some(target) = resolve_state_move_shape(game, region_id, view.identity) else {
                        continue;
                    };
                    occupied = true;
                    let target = (target.shape().get_region_id(), target.shape().identity());
                    if !game.live_skill_target_attackable_between(source, target) { continue; }
                    let target_key = ArrowTargetIdentity::new(target.0, target.1);
                    if attacked.contains(&target_key) { continue; }
                    apply_direct_element_attack(game, instance, source, target, runtime);
                    // Raw Attack может сам пропустить U==S; список всё равно
                    // получает достигнутую цель только после этого вызова.
                    attacked.push(target_key);
                }
                if !occupied {
                    summon_empty_cell(game, source, region_id, cell_x, cell_y, properties);
                }
            }
            row = row.wrapping_add(1);
        }
        column = column.wrapping_add(1);
    }
}
