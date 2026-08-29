//! Шипастая атака `CMonsterThorn` (`0x197`) для достигнутого владельца-монстра.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/monsterthorn.cpp`. Модуль хранит задержку повторного
//! применения, запрет движения,
//! стадии, два визуальных пакета, формулу и два исходных RNG-вызова. Общая
//! `CMonster::IsAttackAble`, защита и применение уже рассчитанного удара идут
//! через узкий `monsterattack`; `CGame` оставляет только возврат региона и
//! межвладельческий хвост смерти. Проверки списка питомцев и повозки для
//! игрока, а также координатные перегрузки остаются RAW до реального
//! вызывающего пути игрока.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterthorn.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterthorn.h

// ============================================================================
// FUNCTION: CMonsterThorn::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterthorn.cpp:102
// RVA: 0x00141520
// ADDRESS: 00541520
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterThorn::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterthorn.cpp:118
// RVA: 0x001415F0
// ADDRESS: 005415f0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterThorn::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterthorn.cpp:85
// RVA: 0x001416E0
// ADDRESS: 005416e0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// IMPLEMENTED: объектная ветвь владельца-монстра материализована ниже;
// вариант игрока сохранён в этом RAW.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterThornEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: действия 0 и 1 для владельца-монстра материализованы ниже;
// клиентские ошибки варианта игрока сохранены в этом RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterthorn.cpp:368
// RVA: 0x001417A0
// ADDRESS: 005417a0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterThorn::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: задержка повторного применения и запрет движения
// владельца-монстра материализованы ниже; ограничения списка питомцев относятся
// к недостигнутому вызывающему пути игрока.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterthorn.cpp:32
// RVA: 0x00141C40
// ADDRESS: 00541c40
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterThorn::CalculateAttackPower
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: формула владельца-монстра и оба RNG-вызова материализованы ниже;
// критический удар игрока и дополнительные атаки сохранены в этом RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterthorn.cpp:300
// RVA: 0x00141E10
// ADDRESS: 00541e10
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterThorn::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: вызов общей защиты и применения удара источником-монстром
// материализован ниже; разрешения игрока сохранены в этом RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterthorn.cpp:276
// RVA: 0x00142060
// ADDRESS: 00542060
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterThorn::AI
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: объектная последовательность источника-монстра между тактами
// материализована ниже; ветви питомцев и повозки игрока сохранены в этом RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterthorn.cpp:147
// RVA: 0x00142180
// ADDRESS: 00542180
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//



















// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, time_reached};
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    owned_monster_attackable, resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const MONSTER_TYPE: i32 = 600;
pub(crate) const MONSTER_THORN_SKILL_ID: u32 = 0x197;

fn send_thorn_visual(
    game: &CGame,
    region: &CServerRegion,
    source_shape: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_level: u16,
    action: u8,
    target: Option<(ShapeIdentity, i32, i32)>,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(action);
    message.add_long(MONSTER_THORN_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    if action == 1 {
        message.add_long(source_shape.get_direction());
    } else if let Some((identity, x, y)) = target {
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.add_long(x);
        message.add_long(y);
    } else {
        return;
    }
    let _ = game.send_game_shape_around(region, source_shape, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт исходного навыка")]
pub(crate) fn execute_owned_monster_thorn<Runtime: GameMainLoopRuntime>(
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
        source_shape,
        property,
        attacker_master,
        attacker_tamed,
        pet_attack,
        cast,
        last_used_ms,
    )) = region.find_monster_by_id(monster_id).and_then(|monster| {
        let property = game
            .find_monster_property_by_origin_name(monster.base_property_key()?)?
            .clone();
        Some((
            monster.move_shape().shape().clone(),
            property.clone(),
            monster.master_info(),
            monster.is_tamed(),
            monster
                .is_tamed()
                .then(|| monster.pet_attack_properties(&property)),
            monster.base_attack_cast(),
            monster.skill_last_used_ms(MONSTER_THORN_SKILL_ID),
        ))
    }) else {
        return false;
    };
    let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity) else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            if cast.is_some_and(|execution| {
                execution.dispatch().skill_id == MONSTER_THORN_SKILL_ID
            }) {
                monster.move_shape_mut().set_moveable(true);
            }
            monster.clear_ai_target();
        }
        return true;
    };
    if target.dead
        || target.god
        || target.city_dead
        || !owned_monster_attackable(
            game,
            region.id,
            &property,
            attacker_tamed,
            attacker_master,
            target_identity,
            &target,
        )
    {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            if cast.is_some_and(|execution| {
                execution.dispatch().skill_id == MONSTER_THORN_SKILL_ID
            }) {
                monster.move_shape_mut().set_moveable(true);
            }
            monster.clear_ai_target();
        }
        return true;
    }
    let (Ok(source_x), Ok(source_y), Ok(target_x), Ok(target_y)) = (
        source_shape.get_tile_x(),
        source_shape.get_tile_y(),
        target.shape.get_tile_x(),
        target.shape.get_tile_y(),
    ) else {
        return true;
    };

    if cast.is_none() {
        let reuse_delay_ms = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        let attack_interval = pet_attack.map_or(property.attack_speed, |pet| pet.attack_interval);
        if !region
            .find_monster_by_id_mut(monster_id)
            .is_some_and(|monster| monster.begin_ai_attack_attempt(now_ms, attack_interval))
            || (last_used_ms != 0 && !time_reached(now_ms, last_used_ms, reuse_delay_ms))
        {
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(
                target_identity,
                MONSTER_THORN_SKILL_ID,
                skill_level,
                now_ms,
            );
        }
        let source_shape = region
            .find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape())
            .unwrap_or(&source_shape);
        send_thorn_visual(
            game,
            region,
            source_shape,
            monster_id,
            skill_level,
            1,
            None,
        );
        return true;
    }

    let cast = cast.expect("выполнение шипастой атаки проверено выше");
    if cast.dispatch().skill_id != MONSTER_THORN_SKILL_ID
        || cast.dispatch().target != target_identity
    {
        return false;
    }
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    if !time_reached(now_ms, cast.started_at_ms(), delay_ms) {
        return true;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
    }
    send_thorn_visual(
        game,
        region,
        &source_shape,
        monster_id,
        skill_level,
        2,
        Some((target_identity, target_x, target_y)),
    );

    let ordinary_attack = region
        .find_monster_by_id(monster_id)
        .map(|monster| {
            monster.state_attack_bounds(property.minimum_attack, property.maximum_attack)
        })
        .unwrap_or((property.minimum_attack, property.maximum_attack));
    let physical_minimum = pet_attack.map_or(ordinary_attack.0, |pet| pet.minimum_attack) as i32;
    let physical_maximum = pet_attack.map_or(ordinary_attack.1, |pet| pet.maximum_attack) as i32;
    let physical_span = physical_maximum
        .wrapping_sub(physical_minimum)
        .unsigned_abs()
        .wrapping_add(1) as i32;
    let physical = physical_minimum.wrapping_add(game.skill_random_below(physical_span));
    // `CMonster::GetCriticalChance` равен нулю, но исходный код всё равно
    // выполняет второй RNG-вызов `random(100)`.
    let _critical_roll = game.skill_random_below(100);
    let attack = AttackInformation {
        skill_id: MONSTER_THORN_SKILL_ID,
        skill_level: skill_level as u8,
        attacker_type: MONSTER_TYPE,
        attacker_id: monster_id,
        attacker_team_id: 0,
        attacker_faction_id: 0,
        attacker_union_id: 0,
        hit_modifier: properties
            .query_property(super::baseattack::SKILL_USAGE_USER_HIT_MODIFIER)
            as i32,
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
            // Виртуальные `GetAddElementAtk/GetAddSoulAtk` монстра возвращают
            // ноль, но обе записи остаются частью исходного порядка защиты.
            AttackPower {
                kind: AttackPowerType::Element,
                hp_damage: 0,
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Soul,
                hp_damage: 0,
                mp_damage: 0,
            },
        ],
    };
    let attack = defend_owned_monster_attack(
        game,
        target_identity,
        target.mana,
        target.war_soul_mana,
        target.player_properties,
        target.monster_properties,
        attack,
    );
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
    }
    apply_owned_monster_attack_hit(
        game,
        region,
        runtime,
        now_ms,
        monster_id,
        attacker_master,
        target_identity,
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
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().shape_mut().set_action(1);
        monster.move_shape_mut().set_moveable(true);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}
