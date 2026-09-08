//! Взрыв трупной свечи `CCorpseCandleBlasting` (`0x194`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/corpsecandleblasting.cpp`. Навык взрывает самого монстра
//! после задержки, обходит восемь клеток подтверждённой маски 3×3 без центра,
//! сохраняет порядок X→Y и отдельный бросок урона для каждой цели. Формула,
//! визуальные пакеты и самоубийственный жизненный цикл находятся здесь; `CGame`
//! остаётся владельцем защиты, применения смерти и сценарной очереди.
//! End (0x00582810, общий со SporeBlasting) сбрасывает флаги, снимает один
//! запрет движения и вызывает CAttackSkill::End. Общая очистка CMonster
//! выполняет его после сообщения смерти либо при отмене/Stiffen без взрыва;
//! скрипт, урон и пометка удаления не являются побочными эффектами End.

use crate::gameserver::appserver::states::state::resolve_owned_skill_begin_object;
use super::baseattack::{SKILL_USAGE_DELAY_TIME, time_reached};
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    monster_attack_cell_candidates, owned_monster_attackable,
    resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::monsterai::{
    MonsterTraceTarget, approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::script::script::ScriptExecutionContext;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

pub(crate) const CORPSE_CANDLE_BLASTING_SKILL_ID: u32 = 0x194;
const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_USER_HIT_MODIFIER: u32 = 3;
const SKILL_USAGE_MIN_ATTACK: u32 = 20_001;
const SKILL_USAGE_MAX_ATTACK: u32 = 20_002;
const SCOPE: [u8; 9] = [1, 1, 1, 1, 0, 1, 1, 1, 1];

fn send_start(game: &CGame, region: &CServerRegion, source: &CShape, skill_level: u16) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(CORPSE_CANDLE_BLASTING_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_fire(game: &CGame, region: &CServerRegion, source: &CShape, skill_level: u16) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(CORPSE_CANDLE_BLASTING_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(source.get_tile_x().unwrap_or_default());
    message.add_long(source.get_tile_y().unwrap_or_default());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn calculate_attack(
    game: &mut CGame,
    source_id: i32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
) -> AttackInformation {
    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let width = maximum.wrapping_sub(minimum).wrapping_abs().wrapping_add(1);
    // Виртуальный `CMonster::GetAddElementAtk` возвращает ноль; здесь остаётся
    // ровно один вызов генератора случайных чисел на допустимую цель.
    let damage = minimum
        .wrapping_add(game.skill_random_below(width))
        .max(0);
    AttackInformation {
        skill_id: CORPSE_CANDLE_BLASTING_SKILL_ID,
        skill_level: skill_level as u8,
        attacker_type: MONSTER_TYPE,
        attacker_id: source_id,
        attacker_team_id: 0,
        attacker_faction_id: 0,
        attacker_union_id: 0,
        hit_modifier: properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32,
        damage_factor: 1.0,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![AttackPower {
            kind: AttackPowerType::Element,
            hp_damage: damage,
            mp_damage: 0,
        }],
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "граница сохраняет владельца, исходную цель и очередь смертей"
)]
pub(crate) fn execute_owned_corpse_candle_blasting<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    deaths: &mut Vec<MonsterAttackDeath>,
) -> bool {
    let Some((
        source,
        property,
        master,
        tamed,
        attack_interval_ms,
        script_file,
        cast,
        last_used_ms,
    )) = region
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
                monster.master_info(),
                monster.is_tamed(),
                attack_interval_ms,
                monster.script_file().to_vec(),
                monster.current_active_attack_cast(game.skill_factory()),
                monster.skill_last_used_ms(CORPSE_CANDLE_BLASTING_SKILL_ID, game.skill_factory()),
            ))
        })
    else {
        return false;
    };

    if cast.is_none() {
        let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity)
        else {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target(game.skill_factory());
            }
            return true;
        };
        if !approach_attack_range(
            game,
            region,
            monster_id,
            MonsterTraceTarget::Shape(target.view),
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
            runtime,
        ) {
            return true;
        }
        if let Some(attack_interval_ms) = schedule_attack_interval(property.ai, attack_interval_ms)
        {
            let attack_started = region
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
        let target_object = resolve_owned_skill_begin_object(game, region, target_identity);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(
                target_identity,
                CORPSE_CANDLE_BLASTING_SKILL_ID,
                skill_level,
                now_ms,
                target_object,
                game.skill_factory(),
            );
        }
        send_start(game, region, &source, skill_level);
        return true;
    }
    let cast = cast.expect("выполнение взрыва трупной свечи проверено выше");
    if cast.dispatch().skill_id != CORPSE_CANDLE_BLASTING_SKILL_ID {
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
    let attacker = MasterInfo {
        master_type: MONSTER_TYPE,
        master_id: monster_id,
        ..MasterInfo::default()
    };
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
                if identity.object_type != PLAYER_TYPE {
                    continue;
                }
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
                let attack = calculate_attack(game, monster_id, skill_level, properties);
                let attack = defend_owned_monster_attack(
                    game,
                    identity,
                    target.mana,
                    target.war_soul_mana,
                    target.player_properties,
                    target.monster_properties,
                    attack,
                );
                apply_owned_monster_attack_hit(
                    game,
                    region,
                    runtime,
                    now_ms,
                    monster_id,
                    attacker,
                    identity,
                    &target.shape,
                    target.health,
                    target.mana,
                    target.master,
                    target.monster_property,
                    target.tamed,
                    target.carriage,
                    attack,
                    deaths,
                );
            }
        }
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.stage_for_delete();
    }
    if target_identity.object_type == PLAYER_TYPE
        && !script_file.is_empty()
        && script_file[0] != b'0'
    {
        let _ = game.run_script_file(
            &script_file,
            ScriptExecutionContext {
                player_id: Some(target_identity.id),
                region_id: Some(region.id),
                ..ScriptExecutionContext::default()
            },
            runtime,
        );
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
        let _ = monster.advance_base_attack_cast(CORPSE_CANDLE_BLASTING_SKILL_ID, SkillStage::Check, SkillStage::Calculate, game.skill_factory());
        let _ = monster.advance_base_attack_cast(CORPSE_CANDLE_BLASTING_SKILL_ID, SkillStage::Calculate, SkillStage::Attack, game.skill_factory());
        let _ = monster.advance_base_attack_cast(CORPSE_CANDLE_BLASTING_SKILL_ID, SkillStage::Attack, SkillStage::Apply, game.skill_factory());
        let _ = monster.finish_base_attack_cast_with_clock(CORPSE_CANDLE_BLASTING_SKILL_ID, game.skill_factory(), || runtime.now_milliseconds());
    }
    true
}
