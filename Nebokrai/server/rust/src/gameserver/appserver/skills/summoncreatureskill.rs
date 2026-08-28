//! Общий достигнутый путь исполнения четырёх исходных владельцев `CSummonSkill`.
//!
//! `CSummonCorpseCandle`, `CSummonSkeleton` и `CSummonSpore` различаются
//! только идентификатором навыка. `CBossFiendSummon` дополнительно выбирает
//! одну из трёх разновидностей ровно одним исходным броском на всё применение.
//! Модуль сохраняет общий объектный путь,
//! задержку повторного применения, задержку исполнения, пакеты `0xBFE01` и
//! последовательность вызовов создания.
//! Поиск владельцев и around-доставка остаются у `CGame`; создаваемая сущность
//! сразу публикуется через `CServerRegion::add_summoned_creature`.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, time_reached};
use super::bossfiendsummon::{BOSS_FIEND_SUMMON_SKILL_ID, summoned_creature_usage};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
const SKILL_USAGE_CONST: u32 = 20_010;
const SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME: u32 = 30_001;
const SKILL_USAGE_SUMMONED_CREATURE_ID: u32 = 30_003;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SummonCreatureProgress {
    destination_x: i32,
    destination_y: i32,
}

fn target_coordinates(
    game: &CGame,
    region: &CServerRegion,
    target: ShapeIdentity,
) -> Option<(i32, i32)> {
    let shape = match target.object_type {
        400 => game.find_player(target.id).and_then(|player| {
            (player.server_region_id() == Some(region.id)).then(|| player.shape())
        }),
        MONSTER_TYPE => region
            .find_monster_by_id(target.id)
            .map(|monster| monster.move_shape().shape()),
        _ => None,
    }?;
    Some((shape.get_tile_x().ok()?, shape.get_tile_y().ok()?))
}

fn send_start(
    game: &CGame,
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_id: u32,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "пакет буквально сохраняет поля исходного сетевого эффекта")]
fn send_fire(
    game: &CGame,
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_id: u32,
    skill_level: u16,
    target_x: i32,
    target_y: i32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(target_x);
    message.add_long(target_y);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет идентификатор, цель и текущий такт исходного навыка")]
pub(crate) fn execute_owned_summon_creature<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target: ShapeIdentity,
    skill_id: u32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    let Some((source, cast, last_used_ms)) = region
        .find_monster_by_id(monster_id)
        .map(|monster| (
            monster.move_shape().shape().clone(),
            monster.base_attack_cast(),
            monster.last_base_attack_ms(),
        ))
    else {
        return false;
    };

    if let Some(cast) = cast {
        if cast.dispatch().skill_id != skill_id {
            return false;
        }
        if !time_reached(now_ms, cast.started_at_ms(), properties.query_property(SKILL_USAGE_DELAY_TIME)) {
            return true;
        }
        let target = target_coordinates(game, region, cast.dispatch().target).or_else(|| {
            region
                .find_monster_by_id(monster_id)
                .and_then(|monster| monster.summon_creature_progress())
                .map(|progress| (progress.destination_x, progress.destination_y))
        });
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        }
        if let Some((target_x, target_y)) = target {
            send_fire(game, region, &source, monster_id, skill_id, skill_level, target_x, target_y);
        }

        let amount = properties.query_property(SKILL_USAGE_CONST);
        let summoned_creature_usage = if skill_id == BOSS_FIEND_SUMMON_SKILL_ID {
            summoned_creature_usage(game.skill_random_below(3))
        } else {
            SKILL_USAGE_SUMMONED_CREATURE_ID
        };
        let source_x = source.get_tile_x().unwrap_or_default();
        let source_y = source.get_tile_y().unwrap_or_default();
        let master = MasterInfo {
            master_type: MONSTER_TYPE,
            master_id: monster_id,
            ..MasterInfo::default()
        };
        let (area_width, area_height) = game.area_dimensions();
        for _ in 0..amount {
            let mut tile_x = 0;
            let mut tile_y = 0;
            if let Ok(position) = region.region.get_random_pos_in_range(
                source_x.wrapping_sub(4), source_y.wrapping_sub(4), 8, 8, runtime,
            ) && position.found {
                tile_x = position.x;
                tile_y = position.y;
            }
            let lifetime_ms = properties.query_property(SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME);
            let picture_id = properties.query_property(summoned_creature_usage);
            let property = game.find_monster_property_by_picture_id(picture_id).cloned();
            if let Some(property) = property {
                let _ = region.add_summoned_creature(
                    &property, master, tile_x, tile_y, -1, lifetime_ms,
                    area_width, area_height, runtime, |runtime| runtime.now_milliseconds(),
                );
            }
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
            let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
            let _ = monster.finish_base_attack_cast(now_ms);
        }
        return true;
    }

    if last_used_ms != 0
        && !time_reached(now_ms, last_used_ms, properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME))
    {
        return true;
    }
    let Some((destination_x, destination_y)) = target_coordinates(game, region, target) else {
        return true;
    };
    let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().set_moveable(false);
        monster.begin_base_attack_cast(target, skill_id, skill_level, now_ms);
        monster.set_summon_creature_progress(SummonCreatureProgress {
            destination_x,
            destination_y,
        });
    }
    send_start(game, region, &source, monster_id, skill_id, skill_level);
    true
}
