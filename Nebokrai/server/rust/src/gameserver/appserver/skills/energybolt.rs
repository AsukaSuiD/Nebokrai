//! Пошаговый энергетический снаряд `CEnergyBolt` для достигнутого monster-owner-а.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает принудительный
//! путь длиной `SKILL_USAGE_TARGET_MAX_DISTANT`, один шаг за единицу времени
//! полёта, живое перечитывание блока клетки и X→Y обход scope. Первый
//! успешный удар либо `BLOCK_UNFLY` посылает промежуточное завершение; на
//! следующем такте исходный owner посылает его повторно и завершает cast.
//! Формула и единственный RNG-вызов на каждую реально атакованную цель остаются
//! здесь, а `CGame` только применяет общую защиту, последствия смерти и сеть.
//! Player MP, `CSoulCollectState` и координатные overload-ы ниже остаются RAW.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\energybolt.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\energybolt.h

// ============================================================================
// FUNCTION: CEnergyBoltEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\energybolt.cpp:777
// RVA: 0x0013B450
// ADDRESS: 0053b450
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnergyBolt::~CEnergyBolt
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\energybolt.cpp:50
// RVA: 0x0013B990
// ADDRESS: 0053b990
// PROTOTYPE: void __thiscall ~CEnergyBolt(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnergyBolt::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\energybolt.cpp:200
// RVA: 0x0013BA50
// ADDRESS: 0053ba50
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnergyBolt::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\energybolt.cpp:232
// RVA: 0x0013BBF0
// ADDRESS: 0053bbf0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnergyBolt::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\energybolt.cpp:263
// RVA: 0x0013BDA0
// ADDRESS: 0053bda0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnergyBolt::End
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\energybolt.cpp:294
// RVA: 0x0013BF50
// ADDRESS: 0053bf50
// PROTOTYPE: void __thiscall End(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnergyBolt::CEnergyBolt
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\energybolt.cpp:33
// RVA: 0x0013BFE0
// ADDRESS: 0053bfe0
// PROTOTYPE: undefined __thiscall CEnergyBolt(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnergyBolt::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\energybolt.cpp:119
// RVA: 0x0013C090
// ADDRESS: 0053c090
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnergyBolt::CalculateAttackPower
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\energybolt.cpp:705
// RVA: 0x0013C2A0
// ADDRESS: 0053c2a0
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnergyBolt::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\energybolt.cpp:659
// RVA: 0x0013C480
// ADDRESS: 0053c480
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnergyBolt::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\energybolt.cpp:577
// RVA: 0x0013C600
// ADDRESS: 0053c600
// PROTOTYPE: int __thiscall Attack(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CEnergyBolt::AI
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\energybolt.cpp:315
// RVA: 0x0013C930
// ADDRESS: 0053c930
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//
























// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    monster_attack_cell_candidates, owned_monster_attackable,
    resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::appserver::states::attackpower::{
    AttackInformation, AttackPower, AttackPowerType,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;
use crate::setup::monsterlist::MonsterProperties;

const MONSTER_TYPE: i32 = 600;
const BLOCK_UNFLY: u8 = 2;
const BLOCK_SHAPE: u8 = 3;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;
const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;
pub(crate) const ENERGY_BOLT_SKILL_ID: u32 = 0x1a0;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnergyBoltProgress {
    destination_x: i32,
    destination_y: i32,
    path: Vec<(i32, i32, u8)>,
    current_position: usize,
    end_x: i32,
    end_y: i32,
    visual_target: Option<ShapeIdentity>,
    missile_flying_time_ms: u32,
    fired: bool,
}

impl EnergyBoltProgress {
    pub(crate) const fn new(destination_x: i32, destination_y: i32) -> Self {
        Self {
            destination_x,
            destination_y,
            path: Vec::new(),
            current_position: 0,
            end_x: 0,
            end_y: 0,
            visual_target: None,
            missile_flying_time_ms: 0,
            fired: false,
        }
    }

    const fn destination(&self) -> (i32, i32) {
        (self.destination_x, self.destination_y)
    }

    const fn fired(&self) -> bool {
        self.fired
    }

    fn fire(&mut self, path: Vec<(i32, i32, u8)>, missile_unit_ms: u32) {
        let unfly = path.iter().position(|cell| cell.2 == BLOCK_UNFLY);
        let end_index = unfly.unwrap_or(path.len());
        if let Some(&(x, y, _)) = unfly
            .and_then(|index| path.get(index))
            .or_else(|| path.last())
        {
            self.end_x = x;
            self.end_y = y;
        }
        self.missile_flying_time_ms = missile_unit_ms.wrapping_mul(end_index as u32);
        self.path = path;
        self.current_position = 1;
        self.visual_target = None;
        self.fired = true;
    }

    fn current_cell(&self) -> Option<(i32, i32)> {
        self.path
            .get(self.current_position)
            .map(|&(x, y, _)| (x, y))
    }

    fn advance(&mut self) {
        self.current_position = self.current_position.wrapping_add(1);
    }

    fn finish_after_collision(&mut self) {
        self.current_position = self.path.len().wrapping_add(1);
    }
}

fn send_start(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(ENERGY_BOLT_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_fire(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
    target: Option<ShapeIdentity>,
    destination: (i32, i32),
    missile_flying_time_ms: u32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(ENERGY_BOLT_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(target.map_or(0, |identity| identity.object_type));
    message.add_long(target.map_or(0, |identity| identity.id));
    message.add_long(destination.0);
    message.add_long(destination.1);
    message.add_ulong(missile_flying_time_ms);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_end(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
    progress: &EnergyBoltProgress,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(3);
    message.add_long(ENERGY_BOLT_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(source.get_direction());
    message.add_long(progress.end_x);
    message.add_long(progress.end_y);
    message.add_long(
        progress
            .visual_target
            .map_or(0, |identity| identity.object_type),
    );
    message.add_long(progress.visual_target.map_or(0, |identity| identity.id));
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет порядок scope, формулу и последствия каждого удара")]
fn attack_scope<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    runtime: &mut Runtime,
    now_ms: u32,
    monster_id: i32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    attacker_property: &MonsterProperties,
    attacker_master: MasterInfo,
    attacker_tamed: bool,
    center_x: i32,
    center_y: i32,
    progress: &mut EnergyBoltProgress,
    deaths: &mut Vec<MonsterAttackDeath>,
) -> bool {
    let scope_radius = (skill_level == 1).then_some(1).unwrap_or(0);
    let mut attacked = Vec::new();
    let mut did_attack = false;
    for offset_x in -scope_radius..=scope_radius {
        for offset_y in -scope_radius..=scope_radius {
            let cell_x = center_x.wrapping_add(offset_x);
            let cell_y = center_y.wrapping_add(offset_y);
            for identity in monster_attack_cell_candidates(
                game, region, monster_id, cell_x, cell_y,
            ) {
                if attacked.contains(&identity) {
                    continue;
                }
                let Some(target) = resolve_owned_monster_attack_target(game, region, identity)
                else {
                    continue;
                };
                if target.dead {
                    continue;
                }
                if cell_x == center_x && cell_y != 0 && progress.visual_target.is_none() {
                    progress.visual_target = Some(identity);
                }
                if target.god
                    || target.city_dead
                    || !owned_monster_attackable(
                        game,
                        region.id,
                        attacker_property,
                        attacker_tamed,
                        attacker_master,
                        identity,
                        &target,
                    )
                {
                    continue;
                }

                // Для monster-owner-а `GetAddElementAtk` и `ElementModify`
                // равны нулю; недостигнутый `CSoulCollectState` не выдумывается.
                let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
                let span = (properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32)
                    .wrapping_sub(minimum)
                    .unsigned_abs()
                    .wrapping_add(1) as i32;
                let damage = minimum.wrapping_add(game.skill_random_below(span)).max(0);
                let attack = AttackInformation {
                    skill_id: ENERGY_BOLT_SKILL_ID,
                    skill_level: skill_level as u8,
                    attacker_type: MONSTER_TYPE,
                    attacker_id: monster_id,
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
                };
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
                    attacker_master,
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
                attacked.push(identity);
                did_attack = true;
            }
        }
    }
    did_attack
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет owner, путь и текущий такт многоцелевого полёта")]
pub(crate) fn execute_owned_energy_bolt<Runtime: GameMainLoopRuntime>(
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
    let Some((source, property, master, tamed, cast, progress, last_used_ms)) = region
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
                monster.energy_bolt_progress().cloned(),
                monster.last_base_attack_ms(),
            ))
        })
    else {
        return false;
    };
    let target = resolve_owned_monster_attack_target(game, region, target_identity);
    let (Ok(source_x), Ok(source_y)) = (source.get_tile_x(), source.get_tile_y()) else {
        return true;
    };
    let live_destination = target.as_ref().and_then(|target| {
        Some((target.shape.get_tile_x().ok()?, target.shape.get_tile_y().ok()?))
    });
    let Some(destination) = live_destination
        .or_else(|| progress.as_ref().map(EnergyBoltProgress::destination))
    else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return true;
    };
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);

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
        let initial_path = region.straight_skill_path(
            source_x, source_y, destination.0, destination.1, None,
        );
        if maximum_distance != 0 && initial_path.len() > maximum_distance as usize {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        let direction = get_line_direction(source_x, source_y, destination.0, destination.1);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.begin_base_attack_cast(
                target_identity,
                ENERGY_BOLT_SKILL_ID,
                skill_level,
                now_ms,
            );
            monster.set_energy_bolt_progress(EnergyBoltProgress::new(
                destination.0,
                destination.1,
            ));
        }
        let source = region
            .find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape())
            .unwrap_or(&source);
        send_start(game, region, source, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение энергетического снаряда проверено выше");
    if cast.dispatch().skill_id != ENERGY_BOLT_SKILL_ID {
        return false;
    }
    let Some(mut progress) = progress else { return true };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let missile_unit_ms = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME);
    if !progress.fired() {
        if !time_reached(now_ms, cast.started_at_ms(), delay_ms) {
            return true;
        }
        let forced_length = (maximum_distance != 0).then_some(maximum_distance);
        let path = region.straight_skill_path(
            source_x,
            source_y,
            destination.0,
            destination.1,
            forced_length,
        );
        if maximum_distance != 0
            && path.len() > maximum_distance.wrapping_add(1) as usize
        {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        progress.fire(path, missile_unit_ms);
        send_fire(
            game,
            region,
            &source,
            skill_level,
            target.as_ref().map(|_| target_identity),
            destination,
            progress.missile_flying_time_ms,
        );
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.set_energy_bolt_progress(progress.clone());
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        }
    }

    let due_ms = delay_ms.wrapping_add(
        missile_unit_ms.wrapping_mul(progress.current_position as u32),
    );
    if !time_reached(now_ms, cast.started_at_ms(), due_ms) {
        return true;
    }
    if progress.current_position >= progress.path.len() {
        send_end(game, region, &source, skill_level, &progress);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
            let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
            let _ = monster.finish_base_attack_cast(now_ms);
        }
        return true;
    }

    let Some((cell_x, cell_y)) = progress.current_cell() else {
        return true;
    };
    progress.end_x = cell_x;
    progress.end_y = cell_y;
    match region.skill_cell_block(cell_x, cell_y) {
        BLOCK_SHAPE => {
            if attack_scope(
                game,
                region,
                runtime,
                now_ms,
                monster_id,
                skill_level,
                properties,
                &property,
                master,
                tamed,
                cell_x,
                cell_y,
                &mut progress,
                deaths,
            ) {
                send_end(game, region, &source, skill_level, &progress);
                progress.finish_after_collision();
                if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                    monster.set_energy_bolt_progress(progress);
                }
                return true;
            }
        }
        BLOCK_UNFLY => {
            send_end(game, region, &source, skill_level, &progress);
            progress.current_position = progress.path.len();
        }
        _ => {}
    }
    progress.advance();
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.set_energy_bolt_progress(progress);
    }
    true
}
