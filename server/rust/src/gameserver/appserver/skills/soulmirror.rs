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
//! fresh Master(country0) и параметрами Zone. Формулу и raw
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
use nebokrai_zone::skills::{SoulMirrorArea, SoulMirrorSummonParameters};

pub(crate) use nebokrai_zone::skills::SOUL_MIRROR_SKILL_ID;

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
    let Some(parameters) = SoulMirrorSummonParameters::read(
        |property| properties.query_property(property),
        || resolve_state_move_shape(game, source.0, source.1)
            .map(|source| source.shape().get_direction()),
    ) else { return; };
    let Some(property) = game.find_monster_property_by_picture_id(parameters.creature_picture_id).cloned() else {
        return;
    };
    let Some(mut owner) = game.take_region_owner(region_id) else { return; };
    let _ = game.add_summoned_creature_owned(
        owner.base_mut(), &property, master, x, y,
        parameters.direction, parameters.lifetime_ms,
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
    let Some(initial_level) = game.registered_skill(instance).map(|skill| skill.level()) else { return; };
    let center_x = user.get_tile_x().unwrap_or(i32::MIN);
    let center_y = user.get_tile_y().unwrap_or(i32::MIN);
    let Some(mut area) = SoulMirrorArea::new((center_x, center_y), initial_level) else { return; };
    let mut attacked = Vec::<ArrowTargetIdentity>::new();

    while let Some((cell_x, cell_y)) = area.next_cell(
        || game.registered_skill(instance).map(|skill| skill.level()),
        || resolve_state_move_shape(game, source.0, source.1)
            .map(|source| source.shape().get_direction()),
    ) {
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
}
