//! Базовая атака монстра и приручённого питомца (`CMonsterBaseAttack`, навык `0x2bd`).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/skills/monsterbaseattack.cpp`. Модуль навыка хранит выбор цели,
//! стадии атаки, преследование, два исходных броска урона, защиту, применение
//! повреждений и точные пакеты. `CGame` оставляет возврат владельца региона и
//! межвладельческие последствия смерти. Неиспользуемые координатный и
//! типизированно-координатный варианты `Begin` сохранены как RAW: их реальный
//! вызывающий путь и отличия от достигнутого объектного пути пока не подтверждены.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterbaseattack.cpp

// ============================================================================
// FUNCTION: CMonsterBaseAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterbaseattack.cpp:111
// RVA: 0x00113B80
// ADDRESS: 00513b80
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterBaseAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterbaseattack.cpp:127
// RVA: 0x00113C50
// ADDRESS: 00513c50
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{
    BASE_ATTACK_SKILL_ID, SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME,
    SKILL_USAGE_TARGET_MAX_DISTANCE, SKILL_USAGE_USER_HIT_MODIFIER, real_distance, time_reached,
};
use super::fightdefense::{
    defend_monster_from_monster_base_attack, defend_player_from_monster_base_attack,
};
use crate::gameserver::appserver::monster::{CMonster, MonsterKillingAttack};
use crate::gameserver::appserver::moveshape::CMoveShape;
use crate::gameserver::appserver::player::CPlayer;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{
    CShape, ShapeAreaCoordinates, ShapeIdentity,
};
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::gameserver::game::{
    CGame, GameMainLoopRuntime, PlayerKillingBlow,
};
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;
use crate::public::tools::get_line_direction;
use crate::setup::monsterlist::MonsterProperties;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct MonsterVictimDeath {
    pub(crate) victim_id: i32,
    pub(crate) attacker_id: i32,
    pub(crate) master_id: i32,
    pub(crate) target_x: i32,
    pub(crate) target_y: i32,
    pub(crate) pos_x_bits: u32,
    pub(crate) pos_y_bits: u32,
    pub(crate) property: MonsterProperties,
}

pub(crate) fn execute_owned_monster_base_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    runtime: &mut Runtime,
    killing_blow: &mut Option<PlayerKillingBlow>,
    monster_killing_blow: &mut Option<MonsterVictimDeath>,
) -> bool {
    let Some((
        property,
        monster_shape,
        monster_view,
        monster_health,
        mut target,
        cast,
        last_attack_ms,
        tamed,
        attacker_master,
        pet_attack_properties,
        moveable,
        trace_move_delay,
        area_index,
        pet_action,
    )) = region.find_monster_by_id(monster_id).and_then(|monster| {
        let property = game
            .find_monster_property_by_origin_name(monster.base_property_key()?)?
            .clone();
        let monster_view = monster.shape_view(&property)?;
        let pet_attack_properties = monster
            .is_tamed()
            .then(|| monster.pet_attack_properties(&property));
        Some((
            property,
            monster.move_shape().shape().clone(),
            monster_view,
            monster.hit_points(),
            monster.ai_target(),
            monster.base_attack_cast(),
            monster.last_base_attack_ms(),
            monster.is_tamed(),
            monster.master_info(),
            pet_attack_properties,
            monster.move_shape().is_moveable(),
            monster.trace_move_delay(),
            monster.move_shape().shape().area_index(),
            monster.pet_action(),
        ))
    })
    else {
        return false;
    };
    if !tamed && property.tamable == 1 && property.maximum_tame_attempt_count == 0 {
        // Carriage has its own derived AI and never runs generic monster
        // base-attack/search scheduling.
        return false;
    }
    if CMoveShape::is_died(monster_health) {
        return false;
    }
    let [skill] = property.skills.as_slice() else {
        return false;
    };
    if u32::from(skill.id) != BASE_ATTACK_SKILL_ID {
        return false;
    }
    let skill = *skill;
    if target.is_none()
        && cast.is_none()
        && !tamed
        && matches!(property.ai, 0 | 3)
        && let Some(area_index) = area_index
    {
        let mut selected = None;
        let mut selected_distance = i32::MAX;
        for player_id in region.player_ids_around_area(area_index) {
            let Some(player) = game.find_player(player_id) else {
                continue;
            };
            if player.server_region_id() != Some(region.id) || player.is_dead() {
                continue;
            }
            let Some(candidate) = player.shape_view() else {
                continue;
            };
            let distance = real_distance(
                monster_view.tile_x,
                monster_view.tile_y,
                candidate.tile_x,
                candidate.tile_y,
            );
            if distance <= 10 && distance <= selected_distance {
                selected = Some(candidate.identity);
                selected_distance = distance;
            }
        }
        if property.kind != 5 {
            for pet_id in region.pet_ids_around_area(area_index) {
                let Some(candidate) = region
                    .find_monster_by_id(pet_id)
                    .filter(|pet| pet.is_tamed() && !CMoveShape::is_died(pet.hit_points()))
                    .and_then(|pet| {
                        let property = game
                            .find_monster_property_by_origin_name(pet.base_property_key()?)?;
                        pet.shape_view(property)
                    })
                else {
                    continue;
                };
                let distance = real_distance(
                    monster_view.tile_x,
                    monster_view.tile_y,
                    candidate.tile_x,
                    candidate.tile_y,
                );
                if distance <= 10 && distance <= selected_distance {
                    selected = Some(candidate.identity);
                    selected_distance = distance;
                }
            }
        }
        for carriage_id in region.carriage_ids_around_area(area_index) {
            let Some((candidate, target_master)) =
                region.find_monster_by_id(carriage_id).and_then(|carriage| {
                    let carriage_property = game
                        .find_monster_property_by_origin_name(carriage.base_property_key()?)?;
                    if !carriage.is_carriage(carriage_property)
                        || CMoveShape::is_died(carriage.hit_points())
                    {
                        return None;
                    }
                    Some((
                        carriage.shape_view(carriage_property)?,
                        carriage.master_info(),
                    ))
                })
            else {
                continue;
            };
            if !game.carriage_attackable_by_monster(
                &property,
                tamed,
                attacker_master,
                target_master,
                region.id,
            ) {
                continue;
            }
            let distance = real_distance(
                monster_view.tile_x,
                monster_view.tile_y,
                candidate.tile_x,
                candidate.tile_y,
            );
            if distance <= 10 && distance <= selected_distance {
                selected = Some(candidate.identity);
                selected_distance = distance;
            }
        }
        if let Some(selected) = selected {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.set_ai_target(selected);
            }
            target = Some(selected);
        }
    }
    if target.is_none()
        && cast.is_none()
        && !tamed
        && matches!(property.ai, 0 | 3)
        && let Some(area_index) = area_index
        && region.player_ids_around_area(area_index).is_empty()
    {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.hibernate_ai(runtime.now_milliseconds());
            return true;
        }
    }
    let target = cast.map(|cast| cast.dispatch().target).or(target);
    let Some(target) =
        target.filter(|target| matches!(target.object_type, PLAYER_TYPE | MONSTER_TYPE))
    else {
        return false;
    };
    let Some(skill_properties) =
        game.skill_base_properties(BASE_ATTACK_SKILL_ID, i32::from(skill.level))
    else {
        return false;
    };
    let delay_ms = skill_properties.query_property(SKILL_USAGE_DELAY_TIME);
    let reuse_delay_ms = skill_properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    let maximum_distance = skill_properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let hit_modifier = skill_properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32;
    let now_ms = runtime.now_milliseconds();
    let target_snapshot = match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).and_then(|player| {
            (player.server_region_id() == Some(region.id)).then(|| {
                (
                    player.shape().clone(),
                    player.health(),
                    player.mana(),
                    player.war_soul_mana(game.goods_factory()),
                    Some(player.combat_properties()),
                    None,
                    player.is_dead(),
                    player.is_god_mode(),
                    player.city_war_died_state(),
                    player.is_badman(game.globe_setup().pk_count_per_kill()),
                    None,
                    None,
                    false,
                    false,
                )
            })
        }),
        MONSTER_TYPE => region
            .find_monster_by_id(target.id)
            .and_then(|target_monster| {
                let target_property = game.find_monster_property_by_origin_name(
                    target_monster.base_property_key()?,
                )?;
                let guard_attackable =
                    !tamed
                        || target_property.kind != 5
                        || (attacker_master.master_type == PLAYER_TYPE
                            && game.find_player(attacker_master.master_id).is_some_and(
                                |master| master.is_badman(game.globe_setup().pk_count_per_kill()),
                            ));
                let carriage = target_monster.is_carriage(target_property);
                let target_master = target_monster.master_info();
                let target_attackable = if carriage {
                    game.carriage_attackable_by_monster(
                        &property,
                        tamed,
                        attacker_master,
                        target_master,
                        region.id,
                    )
                } else {
                    !(!tamed && property.kind == 5)
                        && target_monster.is_tamed() != tamed
                        && guard_attackable
                };
                target_attackable.then(|| {
                    (
                        target_monster.move_shape().shape().clone(),
                        target_monster.hit_points(),
                        0,
                        None,
                        None,
                        Some(target_monster.combat_properties(target_property)),
                        CMoveShape::is_died(target_monster.hit_points()),
                        target_monster.move_shape().is_god(),
                        false,
                        true,
                        Some(target_master),
                        Some(target_property.clone()),
                        target_monster.is_tamed(),
                        carriage,
                    )
                })
            }),
        _ => None,
    };
    let Some((
        target_shape,
        target_health,
        target_mana,
        target_war_soul_mana,
        target_player_properties,
        target_monster_properties,
        target_dead,
        target_god,
        target_city_dead,
        target_badman,
        target_master,
        target_monster_property,
        target_tamed,
        target_carriage,
    )) = target_snapshot
    else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return true;
    };
    if target_dead
        || target_god
        || target_city_dead
        || (!tamed && property.kind == 5 && !target_badman)
    {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return true;
    }
    let (Ok(monster_x), Ok(monster_y), Ok(target_x), Ok(target_y)) = (
        monster_shape.get_tile_x(),
        monster_shape.get_tile_y(),
        target_shape.get_tile_x(),
        target_shape.get_tile_y(),
    ) else {
        return true;
    };

    if tamed && cast.is_none() && pet_action == 0 {
        let (anchor_x, anchor_y) = (attacker_master.master_type == PLAYER_TYPE
            && attacker_master.master_id != 0)
            .then(|| game.find_player(attacker_master.master_id))
            .flatten()
            .filter(|master| master.server_region_id() == Some(region.id))
            .and_then(|master| {
                Some((
                    master.shape().get_tile_x().ok()?,
                    master.shape().get_tile_y().ok()?,
                ))
            })
            .unwrap_or((monster_x, monster_y));
        let anchor_distance = target_x
            .wrapping_sub(anchor_x)
            .unsigned_abs()
            .max(target_y.wrapping_sub(anchor_y).unsigned_abs());
        if anchor_distance >= game.globe_setup().maximum_pet_tracing_distance() {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
    }
    if tamed
        && target.object_type == PLAYER_TYPE
        && attacker_master.master_type == PLAYER_TYPE
        && attacker_master.master_id != 0
        && game
            .find_player(attacker_master.master_id)
            .is_some_and(|master| master.server_region_id() == Some(region.id))
    {
        let pet_identity = ShapeIdentity {
            object_type: MONSTER_TYPE,
            id: monster_id,
            ex_id: CGuid::GUID_INVALID,
        };
        if let Some((string_id, limit)) =
            game.player_base_attack_level_block(attacker_master.master_id, target.id)
        {
            game.send_base_attack_level_block(attacker_master.master_id, string_id, limit);
            game.release_reciprocal_player_target(target.id, pet_identity);
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        if !game.player_base_attackable(attacker_master.master_id, target.id) {
            game.release_reciprocal_player_target(target.id, pet_identity);
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
    }

    if let Some(cast) = cast {
        if !time_reached(now_ms, cast.started_at_ms(), delay_ms) {
            return true;
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        }
        let dispatch = cast.dispatch();
        let mut fire = CMessage::new(0x000b_fe01);
        fire.add_byte(2);
        fire.add_long(BASE_ATTACK_SKILL_ID as i32);
        fire.add_short(dispatch.skill_level as i16);
        fire.add_long(MONSTER_TYPE);
        fire.add_long(monster_id);
        fire.add_long(target.object_type);
        fire.add_long(target.id);
        fire.add_long(target_x);
        fire.add_long(target_y);
        let _ = game.send_game_shape_around(region, &monster_shape, None, &fire);

        let ordinary_attack = region
            .find_monster_by_id(monster_id)
            .map(|monster| {
                monster.battle_fairy_attack_bounds(
                    property.minimum_attack,
                    property.maximum_attack,
                )
            })
            .unwrap_or((property.minimum_attack, property.maximum_attack));
        let physical_minimum = pet_attack_properties
            .map_or(ordinary_attack.0, |pet| pet.minimum_attack) as i32;
        let physical_maximum = pet_attack_properties
            .map_or(ordinary_attack.1, |pet| pet.maximum_attack) as i32;
        let physical_span = physical_maximum
            .wrapping_sub(physical_minimum)
            .max(0)
            .wrapping_add(1);
        let physical = physical_minimum.wrapping_add(game.skill_random_below(physical_span));
        let element_minimum = property.minimum_element as i32;
        let element_maximum = property.maximum_element as i32;
        let element_span = element_maximum
            .wrapping_sub(element_minimum)
            .max(0)
            .wrapping_add(1);
        let element = element_minimum.wrapping_add(game.skill_random_below(element_span));
        let mut attack = AttackInformation {
            skill_id: BASE_ATTACK_SKILL_ID,
            skill_level: dispatch.skill_level as u8,
            attacker_type: MONSTER_TYPE,
            attacker_id: monster_id,
            attacker_team_id: 0,
            attacker_faction_id: 0,
            attacker_union_id: 0,
            hit_modifier,
            damage_factor: 1.0,
            damage_modifier: 0,
            critical: false,
            blast_attack: false,
            full_miss: 0,
            damages: vec![
                AttackPower {
                    kind: AttackPowerType::Physical,
                    hp_damage: physical.max(0),
                    mp_damage: 0,
                },
                AttackPower {
                    kind: AttackPowerType::Element,
                    hp_damage: element.max(0),
                    mp_damage: 0,
                },
                AttackPower {
                    kind: AttackPowerType::Soul,
                    hp_damage: (property.yao_attack & 0xffff) as i32,
                    mp_damage: 0,
                },
            ],
        };
        let mut defense_shields = (target.object_type == PLAYER_TYPE)
            .then(|| game.find_player_mut(target.id))
            .flatten()
            .map(CPlayer::take_defense_shields)
            .unwrap_or_default();
        let globe_setup = game.globe_setup().clone();
        let mut random = |maximum| game.skill_random_below(maximum);
        if let Some(target_properties) = target_player_properties {
            defend_player_from_monster_base_attack(
                &mut attack,
                target_properties,
                target_mana,
                target_war_soul_mana,
                &globe_setup,
                &mut random,
                &mut defense_shields,
            );
        } else if let Some(target_properties) = target_monster_properties {
            defend_monster_from_monster_base_attack(
                &mut attack,
                target_properties,
                &globe_setup,
                &mut random,
            );
        }
        if let Some(player) = game.find_player_mut(target.id) {
            player.restore_defense_shields(defense_shields);
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
            let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        }
        let (damage, mana_damage) =
            CGame::applied_attack_damage(&attack, target_health, target_mana);
        if attack.full_miss != 0 {
            let mut missed = CMessage::new(0x000b_f612);
            missed.add_byte(attack.full_miss);
            missed.add_long(target.object_type);
            missed.add_long(target.id);
            let _ = game.send_game_shape_around(region, &target_shape, None, &missed);
        } else if damage != 0 || mana_damage != 0 {
            let current_health = target_health - damage;
            if target.object_type == PLAYER_TYPE {
                if let Some(player) = game.find_player_mut(target.id) {
                    player.set_health(current_health);
                    player.set_mana(target_mana - mana_damage);
                    player
                        .movement_shape_mut()
                        .set_action(if current_health == 0 { 6 } else { 5 });
                }
            } else if let Some(pet) = region.find_monster_by_id_mut(target.id) {
                pet.set_hit_points(current_health);
                pet.move_shape_mut()
                    .shape_mut()
                    .set_action(if current_health == 0 { 6 } else { 5 });
                if current_health == 0 {
                    pet.when_been_killed(now_ms);
                    if !target_tamed && !target_carriage {
                        pet.set_killed_by(MonsterKillingAttack {
                            attacker_type: MONSTER_TYPE,
                            attacker_id: monster_id,
                            skill_id: attack.skill_id,
                            skill_level: attack.skill_level,
                            critical: attack.critical,
                            blast_attack: attack.blast_attack,
                        });
                    }
                } else {
                    let attacker = ShapeIdentity {
                        object_type: MONSTER_TYPE,
                        id: monster_id,
                        ex_id: CGuid::GUID_INVALID,
                    };
                    if target_tamed {
                        pet.when_pet_been_hurted_by(attacker, now_ms);
                    } else {
                        pet.when_been_hurted_by(attacker, now_ms);
                    }
                }
                if !target_tamed
                    && !target_carriage
                    && attacker_master.master_type == PLAYER_TYPE
                    && attacker_master.master_id != 0
                {
                    let _ = pet.register_attacking_player(
                        attacker_master.master_id,
                        now_ms,
                        game.globe_setup().attack_monster_protection_ms(),
                    );
                }
            }
            if current_health == 0 {
                let mut died = CMessage::new(0x000b_f60b);
                died.add_long(MONSTER_TYPE);
                died.add_long(monster_id);
                died.add_long(target.object_type);
                died.add_long(target.id);
                died.add_ulong(damage);
                died.base_mut().add_char(1);
                CGame::append_base_attack_tail(&mut died, &attack);
                let _ = game.send_game_shape_around(region, &target_shape, None, &died);
                if target.object_type == PLAYER_TYPE {
                    *killing_blow = Some(PlayerKillingBlow {
                        victim_id: target.id,
                        attacker_type: MONSTER_TYPE,
                        attacker_id: monster_id,
                        attacker_faction_id: 0,
                    });
                } else if target_tamed || target_carriage {
                    if let Some(master) = target_master
                        && master.master_type == PLAYER_TYPE
                        && game
                            .find_player(master.master_id)
                            .is_some_and(|player| player.server_region_id() == Some(region.id))
                        && let Some(player) = game.find_player_mut(master.master_id)
                    {
                        if target_carriage {
                            player.clear_active_carriage(target.id);
                        } else {
                            let _ = player.remove_active_pet(MONSTER_TYPE, target.id);
                        }
                    }
                    if let Some(pet) = region.find_monster_by_id_mut(target.id) {
                        if target_carriage {
                            pet.stage_for_delete();
                        } else {
                            pet.evanish_pet();
                        }
                    }
                    let mut vanished = CMessage::new(0x000b_f504);
                    vanished.add_long(MONSTER_TYPE);
                    vanished.add_long(target.id);
                    vanished.add_long(0);
                    vanished
                        .base_mut()
                        .add(&target_shape.get_pos_x().to_bits().to_le_bytes());
                    vanished
                        .base_mut()
                        .add(&target_shape.get_pos_y().to_bits().to_le_bytes());
                    let _ = game.send_game_shape_around(region, &target_shape, None, &vanished);
                } else if let Some(property) = target_monster_property.clone() {
                    *monster_killing_blow = Some(MonsterVictimDeath {
                        victim_id: target.id,
                        attacker_id: monster_id,
                        master_id: (attacker_master.master_type == PLAYER_TYPE)
                            .then_some(attacker_master.master_id)
                            .unwrap_or(0),
                        target_x,
                        target_y,
                        pos_x_bits: target_shape.get_pos_x().to_bits(),
                        pos_y_bits: target_shape.get_pos_y().to_bits(),
                        property,
                    });
                }
            } else {
                let mut hurt = CMessage::new(0x000b_f60a);
                hurt.add_long(MONSTER_TYPE);
                hurt.add_long(monster_id);
                hurt.add_long(target.object_type);
                hurt.add_long(target.id);
                CGame::append_hurt_damage_records(&mut hurt, damage, mana_damage);
                hurt.add_ulong(current_health);
                CGame::append_base_attack_tail(&mut hurt, &attack);
                let _ = game.send_game_shape_around(region, &target_shape, None, &hurt);
                if target.object_type == PLAYER_TYPE {
                    let pets = game
                        .find_player(target.id)
                        .map(|player| player.active_pets().to_vec())
                        .unwrap_or_default();
                    for pet in pets {
                        if pet.object_type == MONSTER_TYPE
                            && let Some(monster) = region.find_monster_by_id_mut(pet.id)
                        {
                            let _ = monster.retarget_passive_pet(ShapeIdentity {
                                object_type: MONSTER_TYPE,
                                id: monster_id,
                                ex_id: CGuid::GUID_INVALID,
                            });
                        }
                    }
                    game.damage_player_armor(target.id, runtime);
                }
            }
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_action(1);
            let _ = monster.finish_base_attack_cast();
        }
        return true;
    }

    let distance = real_distance(monster_x, monster_y, target_x, target_y);
    if maximum_distance != 0 && distance > maximum_distance as i32 {
        if tamed && pet_action == 2 {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        let chase_range = if tamed {
            game.globe_setup().maximum_pet_tracing_distance()
        } else {
            property.chase_range
        };
        if distance > chase_range as i32 {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        if !moveable {
            return true;
        }
        if trace_move_delay
            .is_some_and(|delay| !time_reached(now_ms, delay.started_at_ms, delay.delay_ms))
        {
            return true;
        }

        const SLIP_ORDER: [[usize; 8]; 8] = [
            [0, 7, 1, 6, 2, 5, 3, 4],
            [1, 0, 2, 7, 3, 6, 4, 5],
            [2, 1, 3, 0, 4, 7, 5, 6],
            [3, 2, 4, 1, 5, 0, 6, 7],
            [4, 3, 5, 2, 6, 1, 7, 0],
            [5, 4, 6, 3, 7, 2, 0, 1],
            [6, 5, 7, 4, 0, 3, 1, 2],
            [7, 6, 0, 5, 1, 4, 2, 3],
        ];
        let desired_direction = get_line_direction(monster_x, monster_y, target_x, target_y);
        let origin = ShapeAreaCoordinates {
            x: monster_x,
            y: monster_y,
        };
        let figure = CMonster::figure(&property);
        let figure_index = usize::from(figure.get(0).min(2));
        let destination =
            SLIP_ORDER[desired_direction as usize]
                .into_iter()
                .find_map(|direction| {
                    let destination =
                        CShape::get_direction_position(direction as i32, origin).ok()?;
                    let cells = game.move_check_cells().get(figure_index, direction)?;
                    let clear = cells.iter().all(|cell| {
                        region
                            .region
                            .get_block(
                                origin.x.wrapping_add(cell.x),
                                origin.y.wrapping_add(cell.y),
                            )
                            .is_ok_and(|block| block == 0)
                    });
                    clear.then_some((direction, destination))
                });
        let Some((direction, destination)) = destination else {
            return true;
        };
        let moved = game.move_owned_monster_for_skill(
            region,
            monster_id,
            destination.x,
            destination.y,
            figure,
        );
        if moved {
            let distance_units = if direction % 2 == 0 {
                1_000_000.0
            } else {
                1_414_000.0
            };
            let speed = pet_attack_properties.map_or(monster_shape.get_speed(), |pet| {
                f32::from_bits(pet.speed_bits)
            });
            let delay_ms = if speed > 0.0 {
                let stop_frame =
                    pet_attack_properties.map_or(property.stop_frame, |pet| pet.stop_frame);
                (distance_units * 0.68 / speed + stop_frame as f32)
                    .round()
                    .max(0.0) as u32
            } else {
                0
            };
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.begin_trace_move_delay(now_ms, delay_ms);
            }
        }
        return true;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.clear_trace_move_delay();
    }
    let attack_interval =
        pet_attack_properties.map_or(property.attack_speed, |pet| pet.attack_interval);
    if last_attack_ms != 0
        && (now_ms.wrapping_sub(last_attack_ms) < attack_interval
            || now_ms.wrapping_sub(last_attack_ms) < reuse_delay_ms)
    {
        return true;
    }
    let direction = get_line_direction(monster_x, monster_y, target_x, target_y);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster
            .move_shape_mut()
            .shape_mut()
            .set_direction(direction);
        monster.begin_base_attack_cast(target, skill.level, now_ms);
    }
    let mut start = CMessage::new(0x000b_fe01);
    start.add_byte(1);
    start.add_long(BASE_ATTACK_SKILL_ID as i32);
    start.add_short(skill.level as i16);
    start.add_long(MONSTER_TYPE);
    start.add_long(monster_id);
    start.add_long(direction);
    let _ = game.send_game_shape_around(region, &monster_shape, None, &start);
    true
}
