//! Обход и наложение состояния рыцарского удара.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/knightcut.cpp.
//!
//! Полная маска 5×5 центрируется на сохранённом направлении перед U. Каждый
//! GetShapes снимает отдельную клетку; дедупликации нет. Первый допуск следует
//! за смертью/Cure. Для пары игроков SAFE проверяется у живого U и именно
//! сканируемой клетки; OnFirstSkill вызывается до проверки уровней в Add.
//!
//! Длительность получает свойства данного AI, но Add после строгого сравнения
//! уровней и повторного допуска отдельно разрешает свежую таблицу. ReAnk
//! вычитается у источника-игрока, а не у цели. Удаление старого состояния,
//! Begin нового и сохранение выбранного слота принадлежат общему blind-owner-у.
//! IncreaseRp и ForceMove выполняются и при отказе Begin. Отбрасывание
//! разрешает границу координаты 0 и проверяет только три младших бита блока.

use super::blindstate::replace_primary_blind_state;
use super::cure::CURE_SKILL_ID;
use super::fightdefense::truncate_original;
use super::flash::cell_views;
use super::impactattack::knock_back_impact_target_with_block_mask;
use super::knightcutstate::KnightCutState;
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::region::RegionSecurity;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

const STATE_PERSIST_TIME: u32 = 10_002;
const TIME_PERCENT: u32 = 15_005;

fn add_knight_cut_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity), duration: u32, runtime: &mut Runtime,
) {
    let Some(source_level) = game.move_shape_level(source.0, source.1) else { return; };
    let Some(target_level) = game.move_shape_level(target.0, target.1) else { return; };
    if target_level >= source_level || !game.live_skill_target_attackable(target.0, source.1, target.1) { return; }
    let Some(skill) = game.registered_skill(instance) else { return; };
    let Some(properties) = game.skill_base_properties(skill.id(), skill.level()).cloned() else { return; };
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    if !user.shape().is_assigned_to_server_region() { return; }
    let region_id = user.shape().get_region_id();
    if game.find_region(region_id).is_none() { return; }
    let duration = if source.1.object_type == 400 {
        let Some(player) = game.find_player(source.1.id) else { return; };
        let reduced = duration.wrapping_sub(u32::from(player.combat_properties().reank));
        if (reduced as i32) < 0 { 0 } else { reduced }
    } else { duration };
    let state = KnightCutState::new(0, duration);
    let _ = replace_primary_blind_state(game, source, target, state, &mut || runtime.now_milliseconds());
    if source.1.object_type == 400 { game.increase_owned_player_rp(source.1.id, true, 0); }
    knock_back_impact_target_with_block_mask(game, source, target, region_id, &properties, 7);
}

pub(super) fn run_knight_cut_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    direction: i32, ai_properties: &CSkillBaseProperties, runtime: &mut Runtime,
) {
    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { return; };
    let shape = user.shape();
    if !shape.is_assigned_to_server_region() { return; }
    let region_id = shape.get_region_id();
    if game.find_region(region_id).is_none() { return; }
    let position = ShapeAreaCoordinates {
        x: shape.get_tile_x().unwrap_or(i32::MIN),
        y: shape.get_tile_y().unwrap_or(i32::MIN),
    };
    let Ok(front) = CShape::get_direction_position(direction, position) else { return; };
    let origin_x = front.x.wrapping_sub(2);
    let origin_y = front.y.wrapping_sub(2);
    for x in 0..5 {
        for y in 0..5 {
            let cell_x = origin_x.wrapping_add(x);
            let cell_y = origin_y.wrapping_add(y);
            for view in cell_views(game, region_id, cell_x, cell_y) {
                let Some(sufferer) = resolve_state_move_shape(game, region_id, view.identity) else { continue; };
                let target = (sufferer.shape().get_region_id(), sufferer.shape().identity());
                if target.1 == source.1
                    || game.move_shape_health(target.0, target.1).is_none_or(|health| health == 0)
                    || sufferer.has_state_by_skill_id(CURE_SKILL_ID)
                    || !game.live_skill_target_attackable(target.0, source.1, target.1)
                { continue; }
                if source.1.object_type == 400 && target.1.object_type == 400 {
                    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { continue; };
                    let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
                    let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
                    let Some(owner) = game.find_region(region_id) else { continue; };
                    if owner.get_security(source_x, source_y).ok() == Some(RegionSecurity::SAFE)
                        || owner.get_security(cell_x, cell_y).ok() == Some(RegionSecurity::SAFE)
                    { continue; }
                    let Some(user) = resolve_state_move_shape(game, source.0, source.1) else { continue; };
                    let source_y = user.shape().get_tile_y().unwrap_or(i32::MIN);
                    let source_x = user.shape().get_tile_x().unwrap_or(i32::MIN);
                    let _ = game.player_on_first_skill_at_position(
                        source.1.id, target.1.id, region_id, source_x, source_y, runtime,
                    );
                }
                let duration = if target.1.object_type == 400 {
                    ai_properties.query_property(STATE_PERSIST_TIME)
                } else {
                    let percent = f64::from(ai_properties.query_property(TIME_PERCENT)) as f32;
                    let persist = ai_properties.query_property(STATE_PERSIST_TIME);
                    truncate_original(f64::from(persist) * f64::from(percent) * f64::from(0.01_f32)) as u32
                };
                add_knight_cut_state(game, instance, source, target, duration, runtime);
            }
        }
    }
}
