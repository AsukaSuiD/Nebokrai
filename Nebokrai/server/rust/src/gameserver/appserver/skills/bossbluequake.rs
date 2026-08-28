//! Землетрясение синего босса `CBossBlueQuake` (`0x1f8`) для достигнутого пути монстра.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/bossbluequake.cpp`. Путь монстра сохраняет перезарядку,
//! задержку, направление и поражение передней клетки в порядке региона. Для
//! каждой цели последовательно выполняются исходные вызовы `random`, защита,
//! изменение здоровья, состояние и `ForceMove`. Ветви игрока и координатные
//! перегрузки остаются исходным материалом ниже.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequake.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequake.h

// ============================================================================
// FUNCTION: CBossBlueQuake::CBossBlueQuake
// STATUS: PARTIALLY_IMPLEMENTED
// Идентификатор достигнутого пути задаёт `BOSS_BLUE_QUAKE_SKILL_ID`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequake.cpp:34
// RVA: 0x0012CC60
// ADDRESS: 0052cc60
// PROTOTYPE: undefined __thiscall CBossBlueQuake(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueQuake::~CBossBlueQuake
// STATUS: PARTIALLY_IMPLEMENTED
// Техническое разрушение экземпляра заменено владением Rust.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequake.cpp:43
// RVA: 0x0012CCE0
// ADDRESS: 0052cce0
// PROTOTYPE: void __thiscall ~CBossBlueQuake(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueQuake::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Координатная перегрузка не подключена к достигнутому пути монстра.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequake.cpp:195
// RVA: 0x0012CD00
// ADDRESS: 0052cd00
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueQuake::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Перегрузка с отдельными типом и идентификатором цели остаётся исходным материалом.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequake.cpp:212
// RVA: 0x0012CDC0
// ADDRESS: 0052cdc0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueQuake::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Достигнутая перегрузка цели проходит через `execute_owned_boss_blue_quake`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequake.cpp:177
// RVA: 0x0012CEB0
// ADDRESS: 0052ceb0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueQuakeEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// Достигнутые начало и завершение визуального эффекта формирует `send_visual`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequake.cpp:660
// RVA: 0x0012CF70
// ADDRESS: 0052cf70
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueQuake::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// Условия достигнутого пути монстра проверяются перед началом и продолжением исполнения.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequake.cpp:51
// RVA: 0x0012D360
// ADDRESS: 0052d360
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueQuake::AddBossBlueQuakeState
// STATUS: PARTIALLY_IMPLEMENTED
// Достигнутая ветвь цели проходит через `replace_quake_state`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequake.cpp:462
// RVA: 0x0012D680
// ADDRESS: 0052d680
// PROTOTYPE: void __thiscall AddBossBlueQuakeState(CMoveShape * param_1, CMoveShape * param_2, ulong param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueQuake::CalculateAttackPower
// STATUS: PARTIALLY_IMPLEMENTED
// Формула достигнутой атаки монстра находится в `attack_target`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequake.cpp:592
// RVA: 0x0012D960
// ADDRESS: 0052d960
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueQuake::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// Поражение достигнутой цели выполняет `attack_target`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequake.cpp:565
// RVA: 0x0012DBD0
// ADDRESS: 0052dbd0
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueQuake::AI
// STATUS: PARTIALLY_IMPLEMENTED
// Достигнутый путь монстра выполняет стадии в `execute_owned_boss_blue_quake`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluequake.cpp:243
// RVA: 0x0012DD00
// ADDRESS: 0052dd00
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//























// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::bossbluequakestate::{BossBlueQuakeState, send_boss_blue_quake_state_visual};
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    monster_attack_cell_candidates, owned_monster_attackable, resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeAreaCoordinates, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::appserver::states::attackpower::{AttackInformation, AttackPower, AttackPowerType};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TIME_PERCENT: u32 = 20_004;
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;
const SKILL_USAGE_TARGET_BACK_STEP: u32 = 30_002;
const SKILL_USAGE_TARGET_MOVE_SPEED: u32 = 30_003;

pub(crate) const BOSS_BLUE_QUAKE_SKILL_ID: u32 = 0x1f8;

fn send_visual(game: &CGame, region: &CServerRegion, source: &CShape, level: u16, begin: bool) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(if begin { 1 } else { 2 });
    message.add_long(BOSS_BLUE_QUAKE_SKILL_ID as i32);
    message.add_short(level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    if begin {
        message.add_long(source.get_direction());
    } else if let Ok(face) = source.get_face_position() {
        message.add_long(0);
        message.add_long(0);
        message.add_long(face.x);
        message.add_long(face.y);
    } else {
        return;
    }
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn target_level(game: &CGame, region: &CServerRegion, identity: ShapeIdentity) -> Option<u8> {
    match identity.object_type {
        PLAYER_TYPE => game.find_player(identity.id).map(|player| player.level()),
        MONSTER_TYPE => region.find_monster_by_id(identity.id).and_then(|monster| {
            game.find_monster_property_by_origin_name(monster.base_property_key()?)
                .map(|property| property.level as u8)
        }),
        _ => None,
    }
}

fn replace_quake_state(
    game: &mut CGame,
    region: &mut CServerRegion,
    identity: ShapeIdentity,
    state: BossBlueQuakeState,
    now_ms: u32,
) {
    let Some((tile_x, tile_y, previous)) = (if identity.object_type == PLAYER_TYPE {
        game.find_player_mut(identity.id).and_then(|target| {
            let tile_x = target.shape().get_tile_x().ok()?;
            let tile_y = target.shape().get_tile_y().ok()?;
            let previous = target.replace_boss_blue_quake_state(state);
            if previous.is_some() {
                target.set_skill_moveable(true);
                target.set_skill_fightable(true);
            }
            target.set_skill_moveable(false);
            target.set_skill_fightable(false);
            Some((tile_x, tile_y, previous))
        })
    } else {
        region.find_monster_by_id_mut(identity.id).and_then(|target| {
            let tile_x = target.move_shape().shape().get_tile_x().ok()?;
            let tile_y = target.move_shape().shape().get_tile_y().ok()?;
            let previous = target.move_shape_mut().replace_boss_blue_quake_state(state);
            if previous.is_some() {
                target.move_shape_mut().set_moveable(true);
                target.move_shape_mut().set_fightable(true);
            }
            target.move_shape_mut().set_moveable(false);
            target.move_shape_mut().set_fightable(false);
            Some((tile_x, tile_y, previous))
        })
    }) else { return };
    if let Some(previous) = previous {
        send_boss_blue_quake_state_visual(game, region.id, identity, tile_x, tile_y, previous, false, now_ms);
    }
    send_boss_blue_quake_state_visual(game, region.id, identity, tile_x, tile_y, state, true, now_ms);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет формулу, состояние и ForceMove одной цели")]
fn attack_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    runtime: &mut Runtime,
    now_ms: u32,
    monster_id: i32,
    level: u16,
    properties: &CSkillBaseProperties,
    attacker_property: &crate::setup::monsterlist::MonsterProperties,
    master: MasterInfo,
    tamed: bool,
    identity: ShapeIdentity,
    source_x: i32,
    source_y: i32,
    deaths: &mut Vec<MonsterAttackDeath>,
) {
    let Some(target) = resolve_owned_monster_attack_target(game, region, identity) else { return };
    if target.dead || target.god || target.city_dead || !owned_monster_attackable(
        game, region.id, attacker_property, tamed, master, identity, &target,
    ) { return; }
    let bounds = region.find_monster_by_id(monster_id)
        .map(|monster| monster.state_attack_bounds(attacker_property.minimum_attack, attacker_property.maximum_attack))
        .unwrap_or((attacker_property.minimum_attack, attacker_property.maximum_attack));
    let minimum = bounds.0 as i32;
    let physical = minimum.wrapping_add(game.skill_random_below(
        1_i32.wrapping_sub(minimum).wrapping_add(bounds.1 as i32),
    )).max(0);
    let element_minimum = attacker_property.minimum_element as i32;
    let element = element_minimum.wrapping_add(game.skill_random_below(
        1_i32.wrapping_sub(element_minimum).wrapping_add(attacker_property.maximum_element as i32),
    )).max(0);
    let _critical_roll = game.skill_random_below(100);
    let attack = AttackInformation {
        skill_id: BOSS_BLUE_QUAKE_SKILL_ID,
        skill_level: level as u8,
        attacker_type: MONSTER_TYPE,
        attacker_id: monster_id,
        attacker_team_id: 0,
        attacker_faction_id: 0,
        attacker_union_id: 0,
        hit_modifier: properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32,
        damage_factor: properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR) as f32 * 0.01,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: element, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Soul, hp_damage: (attacker_property.yao_attack & 0xffff) as i32, mp_damage: 0 },
        ],
    };
    let attack = defend_owned_monster_attack(game, identity, target.mana, target.war_soul_mana,
        target.player_properties, target.monster_properties, attack);
    apply_owned_monster_attack_hit(game, region, runtime, now_ms, monster_id, master, identity,
        &target.shape, target.health, target.mana, target.master, target.monster_property,
        target.tamed, target.carriage, attack, deaths);

    if target_level(game, region, identity).is_some_and(|target_level| target_level < attacker_property.level as u8) {
        let persist = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
        let duration = if identity.object_type == PLAYER_TYPE { persist } else {
            (persist as f32 * properties.query_property(SKILL_USAGE_TIME_PERCENT) as f32 * 0.01)
                .round_ties_even() as u32
        };
        replace_quake_state(game, region, identity, BossBlueQuakeState::new(now_ms, duration), now_ms);

        let Ok(target_x) = target.shape.get_tile_x() else { return };
        let Ok(target_y) = target.shape.get_tile_y() else { return };
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        let mut position = ShapeAreaCoordinates { x: target_x, y: target_y };
        let mut moved = 0_u32;
        let back_steps = properties.query_property(SKILL_USAGE_TARGET_BACK_STEP);
        while moved < back_steps {
            let Ok(next) = CShape::get_direction_position(direction, position) else { break };
            if region.region.get_block(next.x, next.y).map_or(true, |block| block & 7 != 0) { break; }
            position = next;
            moved = moved.wrapping_add(1);
        }
        let duration_ms = properties.query_property(SKILL_USAGE_TARGET_MOVE_SPEED).wrapping_mul(moved);
        let _ = game.force_move_owned_shape(region, identity, position.x, position.y, duration_ms, runtime);
    }
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и последствия всех целей клетки")]
pub(crate) fn execute_owned_boss_blue_quake<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    deaths: &mut Vec<MonsterAttackDeath>,
) -> bool {
    let Some((source, property, master, tamed, cast, last_used_ms)) = region.find_monster_by_id(monster_id).and_then(|monster| Some((
        monster.move_shape().shape().clone(),
        game.find_monster_property_by_origin_name(monster.base_property_key()?)?.clone(),
        monster.master_info(), monster.is_tamed(), monster.base_attack_cast(), monster.last_base_attack_ms(),
    ))) else { return false };
    let target = resolve_owned_monster_attack_target(game, region, target_identity);
    let Some((target_x, target_y)) = target.as_ref().and_then(|target| Some((target.shape.get_tile_x().ok()?, target.shape.get_tile_y().ok()?))) else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) { monster.clear_ai_target(); }
        return true;
    };
    if cast.is_none() {
        if last_used_ms != 0 && !time_reached(now_ms, last_used_ms, properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME)) { return true; }
        let direction = get_line_direction(source.get_tile_x().unwrap_or_default(), source.get_tile_y().unwrap_or_default(), target_x, target_y);
        let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(target_identity, BOSS_BLUE_QUAKE_SKILL_ID, level, now_ms);
        }
        send_visual(game, region, &source, level, true);
        return true;
    }
    let cast = cast.expect("выполнение землетрясения проверено выше");
    if cast.dispatch().skill_id != BOSS_BLUE_QUAKE_SKILL_ID { return false; }
    if !time_reached(now_ms, cast.started_at_ms(), properties.query_property(SKILL_USAGE_DELAY_TIME)) { return true; }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().set_moveable(true);
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
    }
    send_visual(game, region, &source, level, false);
    let Ok(face) = source.get_face_position() else { return true };
    let source_x = source.get_tile_x().unwrap_or_default();
    let source_y = source.get_tile_y().unwrap_or_default();
    for identity in monster_attack_cell_candidates(game, region, monster_id, face.x, face.y) {
        attack_target(game, region, runtime, now_ms, monster_id, level, properties, &property,
            master, tamed, identity, source_x, source_y, deaths);
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}
