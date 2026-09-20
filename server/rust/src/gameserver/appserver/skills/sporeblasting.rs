//! Взрыв споры `CSporeBlasting` (`0x195`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/sporeblasting.cpp`. После задержки навык обходит восемь
//! клеток подтверждённой маски 3×3 в порядке X→Y, немедленно заменяет
//! `KnockOutState` каждой допустимой цели и помечает монстра-источник на
//! удаление. Обход допускает RTTI CMoveShape через живой IsAttackAble, без
//! собственного фильтра смерти или ограничения типа 400/600. Новый payload
//! создаётся до End прежнего состояния и destructor свежего остатка слота;
//! общий Blind Begin выполняет visual и блокировки до публикации в прежнем
//! слоте, либо в конце арены, если прежнего состояния не было.
//! End (0x00582810, общий с CorpseCandleBlasting) сбрасывает флаги, снимает
//! один запрет движения и вызывает CAttackSkill::End. AI вызывает его после
//! сообщения смерти (0x00582509); общая очистка CMonster также обслуживает
//! отмену/Stiffen, не взрывая источник и не снимая KnockOutState на целях.

use crate::gameserver::appserver::states::state::{
    end_and_destroy_state_at, resolve_owned_skill_begin_object, resolve_state_move_shape,
};
use super::baseattack::{SKILL_USAGE_DELAY_TIME, time_reached};
use super::blindstate::begin_primary_blind_state_at;
use super::knockoutstate::{KnockOutState, KNOCK_OUT_STATE_ID};
use super::monsterattack::{
    monster_attack_cell_candidates,
    resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, ServerRegionOwner};
use crate::nets::netserver::message::CMessage;

pub(crate) const SPORE_BLASTING_SKILL_ID: u32 = 0x195;
const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SCOPE: [u8; 9] = [1, 1, 1, 1, 0, 1, 1, 1, 1];

fn send_start(game: &CGame, region: &CServerRegion, source: &CShape, skill_level: u16) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(SPORE_BLASTING_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_fire(game: &CGame, region: &CServerRegion, source: &CShape, skill_level: u16) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(SPORE_BLASTING_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(source.get_tile_x().unwrap_or_default());
    message.add_long(source.get_tile_y().unwrap_or_default());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(
    clippy::too_many_arguments,
    reason = "граница сохраняет владельца, цель и часы каждого состояния"
)]
pub(crate) fn execute_owned_spore_blasting<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    let Some(region_owner) = owner.as_mut() else { return false; };
    let region_id = region_owner.region_id();
    let Some((source, property, attack_interval_ms, cast, last_used_ms)) = region_owner.base_mut()
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            let attack_interval_ms = monster
                .is_tamed()
                .then(|| monster.pet_attack_properties(&property))
                .map_or(property.attack_speed, |pet| pet.attack_interval);
            Some((
                monster.move_shape().shape().clone(),
                property,
                attack_interval_ms,
                monster.current_active_attack_cast(game.skill_factory()),
                monster.skill_last_used_ms(SPORE_BLASTING_SKILL_ID, game.skill_factory()),
            ))
        })
    else {
        return false;
    };

    if cast.is_none() {
        let Some(target) = resolve_owned_monster_attack_target(game, region_owner, target_identity)
        else {
            if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target(game.skill_factory());
            }
            return true;
        };
        if !approach_attack_range(
            game,
            region_owner.base_mut(),
            monster_id,
            MonsterTraceTarget::Shape(target.view),
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
            runtime,
        ) {
            return true;
        }
        if let Some(attack_interval_ms) = schedule_attack_interval(property.ai, attack_interval_ms)
        {
            let attack_started = region_owner.base_mut()
                .find_monster_by_id_mut(monster_id)
                .is_some_and(|monster| {
                    monster.begin_ai_attack_attempt(now_ms, attack_interval_ms)
                });
            if !attack_started {
                return true;
            }
        }
        if !crate::gameserver::appserver::skills::kernel::skill_is_restored(
                last_used_ms,
                properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
                now_ms,
            )
        {
            return true;
        }
        let target_object = resolve_owned_skill_begin_object(game, region_owner.base_mut(), target_identity);
        if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(
                target_identity,
                SPORE_BLASTING_SKILL_ID,
                skill_level,
                now_ms,
                target_object,
                game.skill_factory(),
            );
        }
        send_start(game, region_owner.base_mut(), &source, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение взрыва споры проверено выше");
    if cast.dispatch().skill_id != SPORE_BLASTING_SKILL_ID {
        return false;
    }
    if !time_reached(
        now_ms,
        cast.started_at_ms(),
        properties.query_property(SKILL_USAGE_DELAY_TIME),
    ) {
        return true;
    }
    let (Ok(center_x), Ok(center_y)) = (source.get_tile_x(), source.get_tile_y()) else {
        return true;
    };

    send_fire(game, region_owner.base_mut(), &source, skill_level);
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    for x in 0_i32..3 {
        for y in 0_i32..3 {
            if SCOPE[(x + 3 * y) as usize] == 0 {
                continue;
            }
            let cell_x = center_x.wrapping_sub(1).wrapping_add(x);
            let cell_y = center_y.wrapping_sub(1).wrapping_add(y);
            let Some(region_owner) = owner.as_ref() else { return true; };
            for identity in
                monster_attack_cell_candidates(game, region_owner, monster_id, cell_x, cell_y)
            {
                let Some(region_owner) = owner.as_ref() else { return true; };
                if !game.live_skill_target_attackable_in(region_owner, source.identity(), identity) {
                    continue;
                }
                let state = KnockOutState::new(0, keep_time_ms);
                let _ = game.with_published_region(owner, |game| {
                    let Some(target) = resolve_state_move_shape(game, region_id, identity) else { return; };
                    let target_region = target.shape().get_region_id();
                    let placement = if let Some((index, key)) = target.find_state_position(|state| state.state_id() == KNOCK_OUT_STATE_ID) {
                        let Some(location) = target.applied_state_replacement_location(key) else { return; };
                        end_and_destroy_state_at(game, target_region, identity, index);
                        Some(location)
                    } else { None };
                    begin_primary_blind_state_at(
                        game, target_region, identity, Some((region_id, source.identity())),
                        Some((target_region, identity)), state, placement, &mut || runtime.now_milliseconds(),
                    );
                });
            }
        }
    }

    let Some(region_owner) = owner.as_mut() else { return true; };
    if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
        monster.stage_for_delete();
    }
    let mut died = CMessage::new(0x000b_f60b);
    died.add_long(0);
    died.add_long(0);
    died.add_long(MONSTER_TYPE);
    died.add_long(monster_id);
    died.add_ulong(0);
    died.add_byte(2);
    let _ = game.send_game_shape_around(region_owner.base_mut(), &source, None, &died);
    if let Some(monster) = region_owner.base_mut().find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SPORE_BLASTING_SKILL_ID, SkillStage::Check, SkillStage::Calculate, game.skill_factory());
        let _ = monster.advance_base_attack_cast(SPORE_BLASTING_SKILL_ID, SkillStage::Calculate, SkillStage::Attack, game.skill_factory());
        let _ = monster.advance_base_attack_cast(SPORE_BLASTING_SKILL_ID, SkillStage::Attack, SkillStage::Apply, game.skill_factory());
        let _ = monster.finish_base_attack_cast_with_clock(SPORE_BLASTING_SKILL_ID, game.skill_factory(), || runtime.now_milliseconds());
    }
    true
}
