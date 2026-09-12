//! Монстровый вход базовой магии GameServer.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/basemagic.cpp.
//! Существующая региональная очередь владеет cast, отдельно отсчитывает reuse
//! и передаёт путь с footprint цели снаряду CBaseMagicPhalanx. Этот адаптер
//! сохраняет текущую magic-цепочку; базовая стрельба использует свой общий
//! зарегистрированный Begin/AI, без отдельной копии для монстра.

use crate::gameserver::appserver::states::state::resolve_owned_skill_begin_object;
use super::baseattack::time_reached;
use super::basemagicphalanx::CBaseMagicPhalanx;
use super::kernel::SkillStage;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use super::monsterattack::resolve_owned_monster_attack_target;
use super::basemagic::{
    BASE_MAGIC_SKILL_ID, BASE_MAGIC_EFFECT_MESSAGE, SKILL_USAGE_DELAY_TIME,
    SKILL_USAGE_ELEMENT_MODIFIER, SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK,
    SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_SUMMONED_LIFETIME, SKILL_USAGE_SUMMONED_SPEED,
    SKILL_USAGE_TARGET_MAX_DISTANCE,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;
const MONSTER_TYPE: i32 = 600;

fn send_monster_base_magic_visual(
    game: &CGame,
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    skill_level: u16,
    action: u8,
    target: Option<(ShapeIdentity, i32, i32, i32)>,
) {
    let mut message = CMessage::new(BASE_MAGIC_EFFECT_MESSAGE);
    message.add_byte(action);
    message.add_long(BASE_MAGIC_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    if action == 1 {
        message.add_long(source.get_direction());
    } else if let Some((target, x, y, attack_time)) = target {
        message.add_long(target.object_type);
        message.add_long(target.id);
        message.add_long(x);
        message.add_long(y);
        message.add_long(attack_time);
    } else {
        return;
    }
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет monster AI, skill и region owners")]
pub(crate) fn execute_owned_monster_base_magic<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region_owner: &mut crate::gameserver::gameserver::game::ServerRegionOwner,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    runtime: &mut Runtime,
) -> bool {
    let skill_id = BASE_MAGIC_SKILL_ID;
    let Some(properties) = game
        .skill_base_properties(skill_id, i32::from(skill_level))
        .cloned()
    else {
        return false;
    };
    let Some((source, property, tamed, cast, last_used_ms)) = region_owner.base()
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            Some((
                monster.move_shape().shape().clone(),
                game.find_monster_property_by_origin_name(monster.base_property_key()?)?
                    .clone(),
                monster.is_tamed(),
                monster.current_active_attack_cast(game.skill_factory()),
                monster.skill_last_used_ms(skill_id, game.skill_factory()),
            ))
        })
    else {
        return false;
    };
    if cast.is_some_and(|cast| {
        cast.dispatch().skill_id != skill_id
            || cast.dispatch().target != target_identity
    }) {
        return false;
    }
    let now_ms = runtime.now_milliseconds();
    let Some(target) = resolve_owned_monster_attack_target(game, region_owner, target_identity) else {
        if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
            if cast.is_none_or(|execution| execution.termination().is_some()) {
                monster.move_shape_mut().set_moveable(true);
            }
            if cast.is_some() {
                let _ = monster.finish_base_attack_cast_without_reuse(skill_id, game.skill_factory());
            }
            monster.clear_ai_target(game.skill_factory());
        }
        return true;
    };
    if target.dead
        || (cast.is_none()
            && (target.god
                || target.city_dead
                || !game.live_skill_target_attackable_in(region_owner, ShapeIdentity { object_type: MONSTER_TYPE, id: monster_id, ex_id: crate::public::guid::CGuid::GUID_INVALID }, target_identity)))
    {
        if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
            if cast.is_none_or(|execution| execution.termination().is_some()) {
                monster.move_shape_mut().set_moveable(true);
            }
            if cast.is_some() {
                let _ = monster.finish_base_attack_cast_without_reuse(skill_id, game.skill_factory());
            }
            monster.clear_ai_target(game.skill_factory());
        }
        return true;
    }
    let (Ok(source_x), Ok(source_y), Ok(target_x), Ok(target_y)) = (
        source.get_tile_x(),
        source.get_tile_y(),
        target.shape.get_tile_x(),
        target.shape.get_tile_y(),
    ) else {
        if cast.is_some()
            && let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id)
        {
            let _ = monster.finish_base_attack_cast_without_reuse(skill_id, game.skill_factory());
        }
        return true;
    };
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    if cast.is_none() {
        if !approach_attack_range(
            game,
            region_owner.base_mut(),
            monster_id,
            MonsterTraceTarget::Shape(target.view),
            maximum_distance,
            runtime,
        ) {
            return true;
        }
        let attack_interval = if tamed {
            region_owner.base_mut()
                .find_monster_by_id(monster_id)
                .map(|monster| monster.pet_attack_properties(&property).attack_interval)
                .unwrap_or(property.attack_speed)
        } else {
            property.attack_speed
        };
        if schedule_attack_interval(property.ai, attack_interval).is_some_and(|interval| {
            region_owner.base_mut()
                .find_monster_by_id_mut(monster_id)
                .is_none_or(|monster| !monster.begin_ai_attack_attempt(now_ms, interval))
        }) {
            return true;
        }
        let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        if !crate::gameserver::appserver::skills::kernel::skill_is_restored(
                last_used_ms,
                reuse_delay_ms,
                now_ms,
            )
        {
            return true;
        }
        let Some((path_x, path_y)) = game.base_magic_target_point_in(region_owner, source_x, source_y, target_identity)
        else { return true; };
        let path = region_owner.base().straight_skill_path(source_x, source_y, path_x, path_y, None);
        if maximum_distance != 0
            && path.len() > maximum_distance as usize
        {
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        let target_object = resolve_owned_skill_begin_object(game, region_owner.base_mut(), target_identity);
        if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(
                target_identity,
                skill_id,
                skill_level,
                now_ms,
                target_object,
                game.skill_factory(),
            );
        }
        let source = region_owner.base()
            .find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape())
            .unwrap_or(&source);
        send_monster_base_magic_visual(game, region_owner.base(), source, skill_level, 1, None);
        return true;
    }
    let cast = cast.expect("monster base projectile cast проверен выше");
    if !time_reached(
        now_ms,
        cast.started_at_ms(),
        properties.query_property(SKILL_USAGE_DELAY_TIME),
    ) {
        return true;
    }
    if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().set_moveable(true);
    }
    let Some(source_view) = game.shape_view_in_owner(region_owner, source.identity()) else { return true; };
    let attack_time = source_view.real_distance(Some(target.view))
        .wrapping_mul(properties.query_property(SKILL_USAGE_SUMMONED_SPEED) as i32);
    send_monster_base_magic_visual(
        game,
        region_owner.base_mut(),
        &source,
        skill_level,
        2,
        Some((target_identity, target_x, target_y, attack_time)),
    );
    let Some(source_view) = game.shape_view_in_owner(region_owner, source.identity()) else { return true; };
    let forced_distance = source_view.real_distance(Some(target.view)) as u32;
    let Some((path_x, path_y)) = game.base_magic_target_point_in(region_owner, source_x, source_y, target_identity)
    else { return true; };
    let path = region_owner.base().straight_skill_path(
        source_x,
        source_y,
        path_x,
        path_y,
        Some(forced_distance),
    );
    if !path.is_empty() && path.iter().all(|cell| cell.2 != 2) {
        let summon_id = game.allocate_summon_shape_id();
        let started_at_ms = runtime.now_milliseconds();
        let master = MasterInfo {
            master_type: MONSTER_TYPE,
            master_id: monster_id,
            ..MasterInfo::default()
        };
        let (tile_x, tile_y, _) = path[0];
        let (area_width, area_height) = game.area_dimensions();
        let mut phalanx = CBaseMagicPhalanx::new(
            summon_id,
            master,
            started_at_ms,
            properties.query_property(SKILL_USAGE_SUMMONED_LIFETIME),
            i32::from(skill_level),
            properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32,
            properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32,
            properties.query_property(SKILL_USAGE_ELEMENT_MODIFIER) as i32,
            attack_time as u32,
            target_identity,
        );
        phalanx.shape_mut().set_region_id(region_owner.base_mut().id);
        let _ = region_owner.base_mut().add_base_magic_phalanx(
            phalanx,
            tile_x,
            tile_y,
            area_width,
            area_height,
            started_at_ms,
            runtime,
        );
    }
    if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(skill_id, SkillStage::Check, SkillStage::Calculate, game.skill_factory());
        let _ = monster.advance_base_attack_cast(skill_id, SkillStage::Calculate, SkillStage::Attack, game.skill_factory());
        let _ = monster.advance_base_attack_cast(skill_id, SkillStage::Attack, SkillStage::Apply, game.skill_factory());
        monster.move_shape_mut().shape_mut().set_action(1);
        let _ = monster.finish_base_attack_cast_with_clock(skill_id, game.skill_factory(), || runtime.now_milliseconds());
    }
    true
}
