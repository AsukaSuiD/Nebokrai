//! Ядовитая атака паука `CSpiderPoison` (`0x191`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/spiderpoison.cpp`. Достигнутый monster-путь сохраняет
//! проверку пути, задержку, блокировку движения, прямой удар с обязательными
//! вызовами RNG и условную замену `CSpiderPoisonState`. Пакеты навыка и формулы
//! остаются здесь; `CGame` только координирует владельцев и смерть. Перегрузки
//! игрока и наложение `DaubPoison` остаются RAW ниже.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoison.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoison.h

// ============================================================================
// FUNCTION: CSpiderPoison::CSpiderPoison
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoison.cpp:22
// RVA: 0x00185350
// ADDRESS: 00585350
// PROTOTYPE: undefined __thiscall CSpiderPoison(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoison::~CSpiderPoison
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoison.cpp:30
// RVA: 0x001853C0
// ADDRESS: 005853c0
// PROTOTYPE: void __thiscall ~CSpiderPoison(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoison::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoison.cpp:109
// RVA: 0x001853E0
// ADDRESS: 005853e0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoison::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoison.cpp:126
// RVA: 0x001854B0
// ADDRESS: 005854b0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoison::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoison.cpp:91
// RVA: 0x001855B0
// ADDRESS: 005855b0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoisonEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoison.cpp:433
// RVA: 0x00185680
// ADDRESS: 00585680
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoison::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `execute_owned_spider_poison` для достигнутого monster-пути.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoison.cpp:37
// RVA: 0x00185BB0
// ADDRESS: 00585bb0
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoison::CalculateAttackPower
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `execute_owned_spider_poison` сохраняет monster-формулу и RNG.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoison.cpp:368
// RVA: 0x00185CE0
// ADDRESS: 00585ce0
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoison::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `execute_owned_spider_poison` для достигнутого monster-пути.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoison.cpp:344
// RVA: 0x00185F10
// ADDRESS: 00585f10
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderPoison::AI
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: `execute_owned_spider_poison` замыкает достигнутый monster-путь.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderpoison.cpp:156
// RVA: 0x00186020
// ADDRESS: 00586020
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//










// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, time_reached};
use super::basemagic::SKILL_USAGE_TARGET_MAX_DISTANCE;
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    owned_monster_attackable, resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use super::spiderpoisonstate::{SpiderPoisonState, send_spider_poison_state_visual};
use crate::gameserver::appserver::ai::monsterai::approach_attack_range;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const CURE_SKILL_ID: u32 = 0x131;
const LEGACY_UNKNOWN_SKILL_ID: u32 = i32::MAX as u32;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_CONST: u32 = 20_010;
const SKILL_USAGE_BASE_PROBABILITY: u32 = 40_001;
pub(crate) const SPIDER_POISON_SKILL_ID: u32 = 0x191;

fn send_visual(
    game: &CGame,
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_level: u16,
    action: u8,
    target: Option<(ShapeIdentity, i32, i32)>,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(action);
    message.add_long(SPIDER_POISON_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    if action == 1 {
        message.add_long(source.get_direction());
    } else if let Some((identity, x, y)) = target {
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.add_long(x);
        message.add_long(y);
    } else {
        return;
    }
    let _ = game.send_game_shape_around(region, source, None, &message);
}

pub(crate) fn target_has_cure(
    game: &CGame,
    region: &CServerRegion,
    target: ShapeIdentity,
) -> bool {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).is_some_and(|player| player.has_state_by_skill_id(CURE_SKILL_ID)),
        MONSTER_TYPE => region.find_monster_by_id(target.id).is_some_and(|monster| monster.move_shape().has_state_by_skill_id(CURE_SKILL_ID)),
        _ => false,
    }
}

pub(crate) fn install_spider_poison_state(
    game: &mut CGame,
    region: &mut CServerRegion,
    target: ShapeIdentity,
    state: SpiderPoisonState,
    now_ms: u32,
) {
    let replaced = match target.object_type {
        PLAYER_TYPE => game.find_player_mut(target.id).and_then(|player| {
            let identity = player.shape().identity();
            let x = player.shape().get_tile_x().ok()?;
            let y = player.shape().get_tile_y().ok()?;
            Some((player.replace_spider_poison_state(state), identity, x, y))
        }),
        MONSTER_TYPE => region.find_monster_by_id_mut(target.id).and_then(|monster| {
            let identity = monster.move_shape().shape().identity();
            let x = monster.move_shape().shape().get_tile_x().ok()?;
            let y = monster.move_shape().shape().get_tile_y().ok()?;
            Some((monster.move_shape_mut().replace_spider_poison_state(state), identity, x, y))
        }),
        _ => None,
    };
    let Some((previous, identity, x, y)) = replaced else { return };
    if let Some(previous) = previous {
        send_spider_poison_state_visual(game, region.id, identity, x, y, previous, false, now_ms);
    }
    send_spider_poison_state_visual(game, region.id, identity, x, y, state, true, now_ms);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт исходного навыка")]
pub(crate) fn execute_owned_spider_poison<Runtime: GameMainLoopRuntime>(
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
    let Some((source_shape, property, attacker_master, attacker_tamed, pet_attack, cast, last_used_ms)) =
        region.find_monster_by_id(monster_id).and_then(|monster| {
            let property = game.find_monster_property_by_origin_name(monster.base_property_key()?)?.clone();
            let pet_attack = monster.is_tamed().then(|| monster.pet_attack_properties(&property));
            Some((monster.move_shape().shape().clone(), property, monster.master_info(), monster.is_tamed(), pet_attack, monster.base_attack_cast(), monster.last_base_attack_ms()))
        })
    else { return false };
    let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity) else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
            monster.clear_ai_target();
        }
        return true;
    };
    if target.dead || target.god || target.city_dead || !owned_monster_attackable(
        game, region.id, &property, attacker_tamed, attacker_master, target_identity, &target,
    ) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
            monster.clear_ai_target();
        }
        return true;
    }
    let (Ok(source_x), Ok(source_y), Ok(target_x), Ok(target_y)) = (
        source_shape.get_tile_x(), source_shape.get_tile_y(), target.shape.get_tile_x(), target.shape.get_tile_y(),
    ) else { return true };
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let path_too_long = maximum_distance != 0
        && region.straight_skill_path(source_x, source_y, target_x, target_y, None).len() > maximum_distance as usize;
    if cast.is_none() {
        if !approach_attack_range(
            game,
            region,
            monster_id,
            target_x,
            target_y,
            maximum_distance,
            now_ms,
        ) {
            return true;
        }
        if path_too_long {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        let reuse_delay = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
        let attack_interval = pet_attack.map_or(property.attack_speed, |pet| pet.attack_interval);
        if last_used_ms != 0 && (!time_reached(now_ms, last_used_ms, reuse_delay)
            || !time_reached(now_ms, last_used_ms, attack_interval)) {
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(target_identity, SPIDER_POISON_SKILL_ID, skill_level, now_ms);
        }
        let source = region.find_monster_by_id(monster_id).map(|monster| monster.move_shape().shape()).unwrap_or(&source_shape);
        send_visual(game, region, source, monster_id, skill_level, 1, None);
        return true;
    }
    let cast = cast.expect("выполнение ядовитой атаки проверено выше");
    if cast.dispatch().skill_id != SPIDER_POISON_SKILL_ID || cast.dispatch().target != target_identity { return false; }
    if !time_reached(now_ms, cast.started_at_ms(), properties.query_property(SKILL_USAGE_DELAY_TIME)) { return true; }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) { monster.move_shape_mut().set_moveable(true); }
    if path_too_long {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) { monster.clear_ai_target(); }
        return true;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) { let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate); }
    send_visual(game, region, &source_shape, monster_id, skill_level, 2, Some((target_identity, target_x, target_y)));
    let (minimum, maximum, element) = region.find_monster_by_id(monster_id).map(|monster| {
        let (minimum, maximum) = monster.battle_fairy_attack_bounds(property.minimum_attack, property.maximum_attack);
        let minimum = pet_attack.map_or(minimum, |pet| pet.minimum_attack);
        let maximum = pet_attack.map_or(maximum, |pet| pet.maximum_attack);
        (minimum as i32, maximum as i32, monster.battle_fairy_element_modify(0))
    }).unwrap_or((property.minimum_attack as i32, property.maximum_attack as i32, 0));
    let span = maximum.wrapping_sub(minimum).unsigned_abs().wrapping_add(1) as i32;
    let physical = minimum.wrapping_add(game.skill_random_below(span));
    let _critical_roll = game.skill_random_below(100);
    let attack = AttackInformation {
        skill_id: LEGACY_UNKNOWN_SKILL_ID, skill_level: 0, attacker_type: MONSTER_TYPE,
        attacker_id: monster_id, attacker_team_id: 0, attacker_faction_id: 0,
        attacker_union_id: 0, hit_modifier: 0, damage_factor: 1.0, damage_modifier: 0,
        critical: false, blast_attack: false, full_miss: 0,
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical.max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: element.max(0), mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: (property.yao_attack & 0xffff) as i32, mp_damage: 0 },
        ],
    };
    let attack = defend_owned_monster_attack(game, target_identity, target.mana, target.war_soul_mana, target.player_properties, target.monster_properties, attack);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
    }
    apply_owned_monster_attack_hit(game, region, runtime, now_ms, monster_id, attacker_master,
        target_identity, &target.shape, target.health, target.mana, target.master,
        target.monster_property, target.tamed, target.carriage, attack, deaths);
    if !target_has_cure(game, region, target_identity)
        && game.skill_random_below(100) <= properties.query_property(SKILL_USAGE_BASE_PROBABILITY) as i32 {
        install_spider_poison_state(game, region, target_identity, SpiderPoisonState::new(
            MasterInfo { master_type: MONSTER_TYPE, master_id: monster_id, ..MasterInfo::default() },
            now_ms, properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
            properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY),
            properties.query_property(SKILL_USAGE_CONST)), now_ms);
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().shape_mut().set_action(1);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}
