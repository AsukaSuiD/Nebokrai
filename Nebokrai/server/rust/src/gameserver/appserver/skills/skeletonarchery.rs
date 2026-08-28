//! Стрельба скелета `CSkeletonArchery` (`0x1a1`) для владельца-монстра.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/skeletonarchery.cpp`. Модуль сохраняет проверку прямого
//! пути, запрет движения при подготовке, время полёта по первой преграде,
//! текущую клетку объектной цели в момент прилёта, формулу и порядок RNG.
//! Общая допустимость цели, защита и применение удара принадлежат узкому
//! `monsterattack`; `CGame` только возвращает регион между ударами и применяет
//! межвладельческие смерти. Варианты игрока с проверкой оружия, координатные
//! перегрузки и автоматический повтор остаются RAW до реального вызывающего пути.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp

// ============================================================================
// FUNCTION: CSkeletonArchery::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:134
// RVA: 0x00138600
// ADDRESS: 00538600
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkeletonArchery::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:152
// RVA: 0x001386D0
// ADDRESS: 005386d0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkeletonArchery::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:115
// RVA: 0x001387F0
// ADDRESS: 005387f0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// IMPLEMENTED: объектная ветвь владельца-монстра материализована ниже;
// вариант игрока сохранён в этом RAW.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkeletonArcheryEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: действия подготовки и выстрела владельца-монстра
// материализованы ниже; клиентские ошибки игрока сохранены в этом RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:590
// RVA: 0x001388B0
// ADDRESS: 005388b0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkeletonArchery::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: ограничение прямого пути владельца-монстра материализовано
// ниже; проверка оружия относится к недостигнутому вызывающему пути игрока.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:43
// RVA: 0x00138D90
// ADDRESS: 00538d90
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkeletonArchery::CalculateAttackPower
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: формула владельца-монстра и оба RNG-вызова материализованы ниже;
// дополнительные атаки и критический удар игрока сохранены в этом RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:503
// RVA: 0x00138F50
// ADDRESS: 00538f50
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkeletonArchery::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:478
// RVA: 0x001391A0
// ADDRESS: 005391a0
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// IMPLEMENTED: удар источника-монстра по одной цели материализован ниже;
// разрешения игрока сохранены в этом RAW.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkeletonArchery::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:452
// RVA: 0x001392B0
// ADDRESS: 005392b0
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, long param_2, long param_3)
//
// IMPLEMENTED: упорядоченный проход клетки источником-монстром материализован ниже;
// неизвестные разновидности `CMoveShape` сохранены в этом RAW.
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSkeletonArchery::AI
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: объектная последовательность владельца-монстра, подготовка,
// полёт и прилёт материализованы ниже; автоматический повтор сохранён в RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\skeletonarchery.cpp:205
// RVA: 0x001393A0
// ADDRESS: 005393a0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//










// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached,
};
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
const SKILL_USAGE_TARGET_MIN_DISTANCE: u32 = 5_004;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;

pub(crate) const SKELETON_ARCHERY_SKILL_ID: u32 = 0x1a1;

#[derive(Clone, Debug)]
pub(crate) struct SkeletonArcheryDispatch {
    pub(crate) monster_id: i32,
    pub(crate) impact_x: i32,
    pub(crate) impact_y: i32,
    skill_level: u16,
    properties: CSkillBaseProperties,
    property: MonsterProperties,
    attacker_master: MasterInfo,
    attacker_tamed: bool,
    now_ms: u32,
}

fn send_archery_visual(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
    action: u8,
    target: Option<(ShapeIdentity, i32, i32)>,
    timing_ms: u32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(action);
    message.add_long(SKELETON_ARCHERY_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    if action == 1 {
        message.add_long(source.get_direction());
        message.add_ulong(timing_ms);
    } else if let Some((identity, x, y)) = target {
        message.add_long(identity.object_type);
        message.add_long(identity.id);
        message.add_long(x);
        message.add_long(y);
        message.add_ulong(timing_ms);
    } else {
        return;
    }
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт полёта")]
pub(crate) fn prepare_owned_skeleton_archery(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    dispatch: &mut Option<SkeletonArcheryDispatch>,
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
                monster.skeleton_archery_progress(),
                monster.last_base_attack_ms(),
            ))
        })
    else {
        return false;
    };
    let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity) else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            if cast.is_some() {
                monster.move_shape_mut().set_moveable(true);
            }
            monster.clear_ai_target();
        }
        return true;
    };
    if target.dead || target.god || target.city_dead {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            if cast.is_some() {
                monster.move_shape_mut().set_moveable(true);
                let _ = monster.finish_base_attack_cast(now_ms);
            }
            monster.clear_ai_target();
        }
        return true;
    }
    if !owned_monster_attackable(
        game,
        region.id,
        &property,
        tamed,
        master,
        target_identity,
        &target,
    ) {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            if cast.is_some() {
                monster.move_shape_mut().set_moveable(true);
            }
            monster.clear_ai_target();
        }
        return true;
    }
    let (Ok(source_x), Ok(source_y), Ok(target_x), Ok(target_y)) = (
        source.get_tile_x(),
        source.get_tile_y(),
        target.shape.get_tile_x(),
        target.shape.get_tile_y(),
    ) else {
        return true;
    };
    let path = region.straight_skill_path(source_x, source_y, target_x, target_y, None);
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    if cast.is_none()
        && maximum_distance != 0
        && path.len() > maximum_distance.wrapping_add(1) as usize
    {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            if cast.is_some() {
                monster.move_shape_mut().set_moveable(true);
            }
            monster.clear_ai_target();
        }
        return true;
    }

    if cast.is_none() {
        let attack_interval = if tamed {
            region
                .find_monster_by_id(monster_id)
                .map(|monster| monster.pet_attack_properties(&property).attack_interval)
                .unwrap_or(property.attack_speed)
        } else {
            property.attack_speed
        };
        if last_used_ms != 0 && !time_reached(now_ms, last_used_ms, attack_interval) {
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(
                target_identity,
                SKELETON_ARCHERY_SKILL_ID,
                skill_level,
                now_ms,
            );
            monster.begin_skeleton_archery_progress();
        }
        let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
        let source = region
            .find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape())
            .unwrap_or(&source);
        send_archery_visual(game, region, source, skill_level, 1, None, delay_ms);
        return true;
    }

    let cast = cast.expect("выполнение стрельбы скелета проверено выше");
    if cast.dispatch().skill_id != SKELETON_ARCHERY_SKILL_ID
        || cast.dispatch().target != target_identity
    {
        return false;
    }
    let Some(mut progress) = progress else {
        return true;
    };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    if !progress.fired() {
        if !time_reached(now_ms, cast.started_at_ms(), delay_ms) {
            return true;
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
        }
        let minimum_distance = properties.query_property(SKILL_USAGE_TARGET_MIN_DISTANCE);
        if (maximum_distance != 0
            && path.len() > maximum_distance.wrapping_add(1) as usize)
            || (minimum_distance != 0 && path.len() < minimum_distance as usize)
        {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        let mut path_index = path.len();
        for (index, &(x, y, block)) in path.iter().enumerate() {
            let blocked = if block == BLOCK_UNFLY {
                true
            } else if block == BLOCK_SHAPE {
                monster_attack_cell_candidates(game, region, monster_id, x, y)
                    .into_iter()
                    .next()
                    .and_then(|identity| {
                        let target = resolve_owned_monster_attack_target(game, region, identity)?;
                        owned_monster_attackable(
                            game, region.id, &property, tamed, master, identity, &target,
                        )
                        .then_some(())
                    })
                    .is_some()
            } else {
                false
            };
            if blocked {
                path_index = index;
                break;
            }
        }
        let missile_flying_time_ms = properties
            .query_property(SKILL_USAGE_MISSILE_FLYING_TIME)
            .wrapping_mul(path_index as u32);
        send_archery_visual(
            game,
            region,
            &source,
            skill_level,
            2,
            Some((target_identity, target_x, target_y)),
            missile_flying_time_ms,
        );
        progress.fire(missile_flying_time_ms);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            *monster
                .skeleton_archery_progress_mut()
                .expect("состояние полёта принадлежит текущему навыку") = progress;
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        }
    }
    if !time_reached(
        now_ms,
        cast.started_at_ms(),
        delay_ms.wrapping_add(progress.missile_flying_time_ms()),
    ) {
        return true;
    }
    *dispatch = Some(SkeletonArcheryDispatch {
        monster_id,
        impact_x: target_x,
        impact_y: target_y,
        skill_level,
        properties: properties.clone(),
        property,
        attacker_master: master,
        attacker_tamed: tamed,
        now_ms,
    });
    true
}

/// Состояние полёта стрелы между тактами исходного `CSkeletonArchery::AI`.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct SkeletonArcheryProgress {
    missile_flying_time_ms: u32,
    fired: bool,
}

impl SkeletonArcheryProgress {
    pub(crate) const fn fired(self) -> bool {
        self.fired
    }

    pub(crate) const fn missile_flying_time_ms(self) -> u32 {
        self.missile_flying_time_ms
    }

    pub(crate) fn fire(&mut self, missile_flying_time_ms: u32) {
        self.missile_flying_time_ms = missile_flying_time_ms;
        self.fired = true;
    }
}

pub(crate) fn execute_owned_skeleton_archery_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    dispatch: &SkeletonArcheryDispatch,
    identity: ShapeIdentity,
    runtime: &mut Runtime,
    deaths: &mut Vec<MonsterAttackDeath>,
) -> bool {
    let Some(target) = resolve_owned_monster_attack_target(game, region, identity) else {
        return false;
    };
    if target.dead
        || target.god
        || target.city_dead
        || !owned_monster_attackable(
            game,
            region.id,
            &dispatch.property,
            dispatch.attacker_tamed,
            dispatch.attacker_master,
            identity,
            &target,
        )
    {
        return false;
    }
    let bounds = region
        .find_monster_by_id(dispatch.monster_id)
        .map(|monster| {
            if dispatch.attacker_tamed {
                let pet = monster.pet_attack_properties(&dispatch.property);
                (pet.minimum_attack, pet.maximum_attack)
            } else {
                monster.battle_fairy_attack_bounds(
                    dispatch.property.minimum_attack,
                    dispatch.property.maximum_attack,
                )
            }
        })
        .unwrap_or((
            dispatch.property.minimum_attack,
            dispatch.property.maximum_attack,
        ));
    let span = (bounds.1 as i32)
        .wrapping_sub(bounds.0 as i32)
        .unsigned_abs()
        .wrapping_add(1) as i32;
    let physical = (bounds.0 as i32).wrapping_add(game.skill_random_below(span));
    // Нулевая вероятность критического удара монстра не устраняет исходный
    // второй RNG-вызов.
    let _critical_roll = game.skill_random_below(100);
    let attack = AttackInformation {
        skill_id: SKELETON_ARCHERY_SKILL_ID,
        skill_level: dispatch.skill_level as u8,
        attacker_type: MONSTER_TYPE,
        attacker_id: dispatch.monster_id,
        attacker_team_id: 0,
        attacker_faction_id: 0,
        attacker_union_id: 0,
        hit_modifier: dispatch
            .properties
            .query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32,
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
        dispatch.now_ms,
        dispatch.monster_id,
        dispatch.attacker_master,
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
    true
}

pub(crate) fn finish_owned_skeleton_archery(
    region: &mut CServerRegion,
    dispatch: &SkeletonArcheryDispatch,
) {
    if let Some(monster) = region.find_monster_by_id_mut(dispatch.monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        monster.move_shape_mut().shape_mut().set_action(1);
        let _ = monster.finish_base_attack_cast(dispatch.now_ms);
    }
}
