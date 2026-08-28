//! Взрыв споры `CSporeBlasting` (`0x195`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/sporeblasting.cpp`. После задержки навык обходит восемь
//! клеток подтверждённой маски 3×3 в порядке X→Y, немедленно заменяет
//! `KnockOutState` каждой допустимой цели и помечает монстра-источник на
//! удаление. Каноническое состояние и его `End(old) → Begin(new)` принадлежат
//! `knockoutstate.rs`; `CGame` остаётся координатором поиска и доставки.

use super::baseattack::{SKILL_USAGE_DELAY_TIME, time_reached};
use super::knockoutstate::{
    KnockOutState, replace_monster_knock_out_state, replace_player_knock_out_state,
};
use super::monsterattack::{
    monster_attack_cell_candidates, owned_monster_attackable,
    resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

pub(crate) const SPORE_BLASTING_SKILL_ID: u32 = 0x195;
const PLAYER_TYPE: i32 = 400;
const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
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
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    let Some((source, property, master, tamed, cast, last_used_ms)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            Some((
                monster.move_shape().shape().clone(),
                property,
                monster.master_info(),
                monster.is_tamed(),
                monster.base_attack_cast(),
                monster.last_base_attack_ms(),
            ))
        })
    else {
        return false;
    };

    if cast.is_none() {
        if last_used_ms != 0
            && !time_reached(
                now_ms,
                last_used_ms,
                properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
            )
        {
            return true;
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(
                target_identity,
                SPORE_BLASTING_SKILL_ID,
                skill_level,
                now_ms,
            );
        }
        send_start(game, region, &source, skill_level);
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

    send_fire(game, region, &source, skill_level);
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    for x in 0_i32..3 {
        for y in 0_i32..3 {
            if SCOPE[(x + 3 * y) as usize] == 0 {
                continue;
            }
            let cell_x = center_x.wrapping_sub(1).wrapping_add(x);
            let cell_y = center_y.wrapping_sub(1).wrapping_add(y);
            for identity in
                monster_attack_cell_candidates(game, region, monster_id, cell_x, cell_y)
            {
                let Some(target) = resolve_owned_monster_attack_target(game, region, identity)
                else {
                    continue;
                };
                if target.dead
                    || target.god
                    || target.city_dead
                    || !owned_monster_attackable(
                        game,
                        region.id,
                        &property,
                        tamed,
                        master,
                        identity,
                        &target,
                    )
                {
                    continue;
                }
                let state_now_ms = runtime.now_milliseconds();
                let state = KnockOutState::new(state_now_ms, keep_time_ms);
                let visual_now_ms = runtime.now_milliseconds();
                match identity.object_type {
                    PLAYER_TYPE => {
                        let _ = replace_player_knock_out_state(
                            game,
                            identity.id,
                            state,
                            visual_now_ms,
                        );
                    }
                    MONSTER_TYPE => {
                        let _ = replace_monster_knock_out_state(
                            game,
                            region,
                            identity.id,
                            state,
                            visual_now_ms,
                        );
                    }
                    _ => {}
                }
            }
        }
    }

    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.stage_for_delete();
        monster.move_shape_mut().set_moveable(true);
    }
    let mut died = CMessage::new(0x000b_f60b);
    died.add_long(0);
    died.add_long(0);
    died.add_long(MONSTER_TYPE);
    died.add_long(monster_id);
    died.add_ulong(0);
    died.add_byte(2);
    let _ = game.send_game_shape_around(region, &source, None, &died);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}
