//! Призыв движущейся сферы CChaosSphere.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/chaossphere.cpp.
//! Begin/Check/AI/visual/End общие в zonalcast: Check не читает путь и цель,
//! AI после проверки смерти сохраняет S как точку. Summon заново читает таблицу:
//! SPEED, при ненулевом значении ещё SPEED→LIFETIME/div→GetAttackPath(length).
//! Исходный пустой путь прекращает Summon. Иначе identity S очищается, BLOCK2
//! обрезает остаток; получившийся пустой путь всё ещё допускает форму.
//! Затем Master(country0)/Player EM→параметры Zone→ctor(clock→ID).
//! SetTile использует path[0] либо свежие captured U Y/X, не аргументы AI.
//! После свежего actual region U выполняются Add→encode/BF502 даже при отказе.
//! Путь принадлежит Vec формы; отдельного payload или condition рядом с kernel нет.

use super::chaosspherephalanx::CChaosSpherePhalanx;
use super::weaponattack::{SourceProperty, source_master, source_property};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use nebokrai_zone::skills::{ChaosSphereSummonParameters, ElementSummonLiveField,
    chaos_sphere_path_length};

pub(crate) use nebokrai_zone::skills::CHAOS_SPHERE_SKILL_ID;

pub(super) fn summon_chaos_sphere<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity), runtime: &mut Runtime,
) {
    if resolve_state_move_shape(game, source.0, source.1).is_none() { return; }
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let length = chaos_sphere_path_length(|property| properties.query_property(property));
    let mut path = game.skill_target_path_with_length(skill.lifecycle(), length);
    if path.is_empty() { return; }
    if let Some(skill) = game.registered_skill_mut(instance) {
        let destination = skill.lifecycle().destination();
        skill.lifecycle_mut().set_point_target(destination);
    }
    if let Some(index) = path.iter().position(|cell| cell.2 == 2) { path.truncate(index); }
    let Some(mut master) = source_master(game, source) else { return; };
    master.master_country_id = 0;
    let element = if source.1.object_type == 400 {
        let Some(player) = game.find_player(source.1.id) else { return; };
        player.combat_properties().element_modify
    } else { 0 };
    let Some(parameters) = ChaosSphereSummonParameters::read(
        |property| properties.query_property(property),
        |field| match field {
            ElementSummonLiveField::CriticalChance => source_property(game, source, SourceProperty::CriticalChance).map(|value| value as i32),
            ElementSummonLiveField::AddElementAttack => source_property(game, source, SourceProperty::Element).map(|value| value as i32),
        },
        || game.registered_skill(instance).map(|skill| i32::from(skill.level())),
        element,
    ) else { return; };
    let started = runtime.now_milliseconds();
    let id = game.allocate_summon_shape_id();
    let mut phalanx = CChaosSpherePhalanx::new(
        id, master, started, parameters,
        path.iter().map(|&(x, y, _)| (x, y)).collect(),
    );
    let (x, y) = if let Some(&(x, y, _)) = path.first() { (x, y) } else {
        let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
        let y = user.shape().get_tile_y().unwrap_or(i32::MIN);
        let x = user.shape().get_tile_x().unwrap_or(i32::MIN);
        (x, y)
    };
    phalanx.shape_mut().set_pos_xy_base(
        (f64::from(x) + 0.5) as f32, (f64::from(y) + 0.5) as f32,
    );
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region = user.shape().get_region_id();
    let _ = game.add_chaos_sphere_phalanx(region, phalanx, started, runtime);
}
