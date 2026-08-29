//! Проникающая атака демона-босса `CBossFiendPenetrate` (`0x1FA`).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный владелец
//! `appserver/skills/bossfiendpenetrate.cpp`. Достигнутый объектный путь монстра
//! сохраняет задержку и направление, единственный пакет запуска полёта,
//! поражение одной клетки за такт до первого `BLOCK_UNFLY` и запрет повторного
//! поражения одной цели на всём пути. Порядок кандидатов задаёт регион, формула
//! и исходная последовательность `random` остаются у навыка, а общие защита, изменение
//! цели и последствия смерти проходят через `monsterattack`. Варианты игрока,
//! координатные перегрузки и `OnChangeRegion` остаются исходным материалом.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendpenetrate.cpp

// ============================================================================
// FUNCTION: CBossFiendPenetrate::OnChangeRegion
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendpenetrate.cpp:721
// RVA: 0x0012AAC0
// ADDRESS: 0052aac0
// PROTOTYPE: void __thiscall OnChangeRegion(long param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossFiendPenetrateEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// Действия `0/1/3` объектного пути монстра формируют `send_start`,
// `send_fire` и `send_empty_path`; клиентские ошибки игрока остаются ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendpenetrate.cpp:816
// RVA: 0x0012AAD0
// ADDRESS: 0052aad0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossFiendPenetrate::~CBossFiendPenetrate
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendpenetrate.cpp:34
// RVA: 0x0012B060
// ADDRESS: 0052b060
// PROTOTYPE: void __thiscall ~CBossFiendPenetrate(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossFiendPenetrate::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Объектный путь монстра начинает `execute_owned_boss_fiend_penetrate`;
// вариант игрока остаётся ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendpenetrate.cpp:173
// RVA: 0x0012B0F0
// ADDRESS: 0052b0f0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossFiendPenetrate::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendpenetrate.cpp:196
// RVA: 0x0012B1F0
// ADDRESS: 0052b1f0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossFiendPenetrate::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendpenetrate.cpp:218
// RVA: 0x0012B300
// ADDRESS: 0052b300
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossFiendPenetrate::End
// STATUS: PARTIALLY_IMPLEMENTED
// Завершение пути монстра выполняет `finish_base_attack_cast`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendpenetrate.cpp:240
// RVA: 0x0012B410
// ADDRESS: 0052b410
// PROTOTYPE: void __thiscall End(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossFiendPenetrate::CBossFiendPenetrate
// STATUS: PARTIALLY_IMPLEMENTED
// Идентификатор достигнутого пути задаёт `BOSS_FIEND_PENETRATE_SKILL_ID`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendpenetrate.cpp:22
// RVA: 0x0012B4B0
// ADDRESS: 0052b4b0
// PROTOTYPE: undefined __thiscall CBossFiendPenetrate(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossFiendPenetrate::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// Дальность, задержка повторного применения и блокировка движения монстра
// проверяются в `execute_owned_boss_fiend_penetrate`; экипировка игрока ниже
// остаётся исходным материалом.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendpenetrate.cpp:47
// RVA: 0x0012B540
// ADDRESS: 0052b540
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossFiendPenetrate::CalculateAttackPower
// STATUS: PARTIALLY_IMPLEMENTED
// Формула и последовательность `random` пути монстра принадлежат
// `attack_target`; свойства игрока остаются исходным материалом.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendpenetrate.cpp:652
// RVA: 0x0012B8A0
// ADDRESS: 0052b8a0
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossFiendPenetrate::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// Уникальность цели и один удар объектного пути монстра выполняют
// `BossFiendPenetrateProgress` и `attack_target`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendpenetrate.cpp:609
// RVA: 0x0012BB30
// ADDRESS: 0052bb30
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossFiendPenetrate::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// Упорядоченный снимок клетки пути монстра даёт
// `monster_attack_cell_candidates`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendpenetrate.cpp:580
// RVA: 0x0012BC80
// ADDRESS: 0052bc80
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossFiendPenetrate::AI
// STATUS: PARTIALLY_IMPLEMENTED
// Стадии объектного пути монстра выполняет
// `execute_owned_boss_fiend_penetrate`; расход и ошибки игрока остаются ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossfiendpenetrate.cpp:259
// RVA: 0x0012BD80
// ADDRESS: 0052bd80
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


















// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE, SKILL_USAGE_USER_HIT_MODIFIER,
    time_reached,
};
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    monster_attack_cell_candidates, owned_monster_attackable, resolve_owned_monster_attack_target,
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

const MONSTER_TYPE: i32 = 600;
const BLOCK_UNFLY: u8 = 2;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_CAN_BE_BREAKED: u32 = 10_006;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;

pub(crate) const BOSS_FIEND_PENETRATE_SKILL_ID: u32 = 0x1fa;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BossFiendPenetrateProgress {
    destination_x: i32,
    destination_y: i32,
    path: Vec<(i32, i32, u8)>,
    current_cell: usize,
    attack_cell_count: usize,
    attacked: Vec<ShapeIdentity>,
    fired: bool,
}

impl BossFiendPenetrateProgress {
    pub(crate) const fn new(destination_x: i32, destination_y: i32) -> Self {
        Self {
            destination_x,
            destination_y,
            path: Vec::new(),
            current_cell: 0,
            attack_cell_count: 0,
            attacked: Vec::new(),
            fired: false,
        }
    }

    const fn destination(&self) -> (i32, i32) {
        (self.destination_x, self.destination_y)
    }

    fn fire(&mut self, path: Vec<(i32, i32, u8)>) {
        self.attack_cell_count = path
            .iter()
            .position(|cell| cell.2 == BLOCK_UNFLY)
            .unwrap_or(path.len());
        self.path = path;
        self.current_cell = 0;
        self.fired = true;
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
    message.add_long(BOSS_FIEND_PENETRATE_SKILL_ID as i32);
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
    target_x: i32,
    target_y: i32,
    missile_flying_time_ms: u32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(BOSS_FIEND_PENETRATE_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(target.map_or(0, |identity| identity.object_type));
    message.add_long(target.map_or(0, |identity| identity.id));
    message.add_long(target_x);
    message.add_long(target_y);
    message.add_ulong(missile_flying_time_ms);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_empty_path(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(3);
    message.add_long(BOSS_FIEND_PENETRATE_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет формулу и порядок последствий одного поражения")]
fn attack_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    runtime: &mut Runtime,
    now_ms: u32,
    monster_id: i32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    attacker_property: &crate::setup::monsterlist::MonsterProperties,
    attacker_master: MasterInfo,
    attacker_tamed: bool,
    identity: ShapeIdentity,
    deaths: &mut Vec<MonsterAttackDeath>,
) {
    let Some(target) = resolve_owned_monster_attack_target(game, region, identity) else {
        return;
    };
    if target.dead
        || target.god
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
        return;
    }

    let (minimum, maximum) = region
        .find_monster_by_id(monster_id)
        .map(|monster| {
            monster.state_attack_bounds(
                attacker_property.minimum_attack,
                attacker_property.maximum_attack,
            )
        })
        .unwrap_or((attacker_property.minimum_attack, attacker_property.maximum_attack));
    let minimum = minimum as i32;
    let maximum = maximum as i32;
    let span = 1_i32.wrapping_sub(minimum).wrapping_add(maximum);
    let physical = minimum.wrapping_add(game.skill_random_below(span)).max(0);
    let element_minimum = attacker_property.minimum_element as i32;
    let element_maximum = attacker_property.maximum_element as i32;
    let element_span = 1_i32
        .wrapping_sub(element_minimum)
        .wrapping_add(element_maximum);
    let element = element_minimum
        .wrapping_add(game.skill_random_below(element_span))
        .max(0);
    // `CMonster::GetCriticalChance` возвращает ноль, но исходный вызов
    // `random(100)` всё равно продвигает общий генератор.
    let _critical_roll = game.skill_random_below(100);
    let attack = AttackInformation {
        skill_id: BOSS_FIEND_PENETRATE_SKILL_ID,
        skill_level: skill_level as u8,
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
            AttackPower {
                kind: AttackPowerType::Physical,
                hp_damage: physical,
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Element,
                hp_damage: element,
                mp_damage: 0,
            },
            AttackPower {
                kind: AttackPowerType::Soul,
                hp_damage: (attacker_property.yao_attack & 0xffff) as i32,
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
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, путь и текущий такт навыка")]
pub(crate) fn execute_owned_boss_fiend_penetrate<Runtime: GameMainLoopRuntime>(
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
            Some((
                monster.move_shape().shape().clone(),
                game.find_monster_property_by_origin_name(monster.base_property_key()?)?.clone(),
                monster.master_info(),
                monster.is_tamed(),
                monster.base_attack_cast(),
                monster.boss_fiend_penetrate_progress().cloned(),
                monster.skill_last_used_ms(BOSS_FIEND_PENETRATE_SKILL_ID),
            ))
        })
    else {
        return false;
    };
    let (Ok(source_x), Ok(source_y)) = (source.get_tile_x(), source.get_tile_y()) else {
        return true;
    };
    let target = resolve_owned_monster_attack_target(game, region, target_identity);
    let live_destination = target.as_ref().and_then(|target| {
        (!target.dead).then_some((target.shape.get_tile_x().ok()?, target.shape.get_tile_y().ok()?))
    });
    let Some(destination) = live_destination
        .or_else(|| progress.as_ref().map(BossFiendPenetrateProgress::destination))
    else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
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
        let path = region.straight_skill_path(
            source_x,
            source_y,
            destination.0,
            destination.1,
            None,
        );
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        let _can_be_breaked = properties.query_property(SKILL_USAGE_CAN_BE_BREAKED);
        let direction = get_line_direction(source_x, source_y, destination.0, destination.1);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(
                target_identity,
                BOSS_FIEND_PENETRATE_SKILL_ID,
                skill_level,
                now_ms,
            );
            monster.set_boss_fiend_penetrate_progress(BossFiendPenetrateProgress::new(
                destination.0,
                destination.1,
            ));
        }
        send_start(game, region, &source, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение проникающего удара проверено выше");
    if cast.dispatch().skill_id != BOSS_FIEND_PENETRATE_SKILL_ID {
        return false;
    }
    let Some(mut progress) = progress else { return true };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    let missile_flying_time_ms = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME);
    if !progress.fired {
        if target.as_ref().is_some_and(|target| target.dead) {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.move_shape_mut().set_moveable(true);
                monster.clear_ai_target();
            }
            return true;
        }
        if !time_reached(now_ms, cast.started_at_ms(), delay_ms) {
            return true;
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
        }
        let forced_length = (maximum_distance != 0).then_some(maximum_distance);
        let path = region.straight_skill_path(
            source_x,
            source_y,
            destination.0,
            destination.1,
            forced_length,
        );
        if path.is_empty() {
            send_empty_path(game, region, &source, skill_level);
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                let _ = monster.finish_base_attack_cast(now_ms);
            }
            return true;
        }
        if maximum_distance != 0
            && path.len() > maximum_distance.wrapping_add(1) as usize
        {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        progress.fire(path);
        send_fire(
            game,
            region,
            &source,
            skill_level,
            target.as_ref().map(|_| target_identity),
            destination.0,
            destination.1,
            missile_flying_time_ms,
        );
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.set_boss_fiend_penetrate_progress(progress.clone());
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        }
    }

    if progress.current_cell >= progress.attack_cell_count {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
            let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
            let _ = monster.finish_base_attack_cast(now_ms);
        }
        return true;
    }
    let due_ms = delay_ms.wrapping_add(
        missile_flying_time_ms.wrapping_mul(progress.current_cell as u32),
    );
    if !time_reached(now_ms, cast.started_at_ms(), due_ms) {
        return true;
    }
    let Some(&(cell_x, cell_y, _)) = progress.path.get(progress.current_cell) else {
        return true;
    };
    for identity in monster_attack_cell_candidates(game, region, monster_id, cell_x, cell_y) {
        if progress.attacked.contains(&identity) {
            continue;
        }
        let Some(target) = resolve_owned_monster_attack_target(game, region, identity) else {
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
        progress.attacked.push(identity);
        attack_target(
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
            identity,
            deaths,
        );
    }
    progress.current_cell = progress.current_cell.wrapping_add(1);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.set_boss_fiend_penetrate_progress(progress);
    }
    true
}
