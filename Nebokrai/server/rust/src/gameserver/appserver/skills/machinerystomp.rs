//! Механический топот `CMachineryStomp` (`0x1a7`) для достигнутого пути монстра.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/machinerystomp.cpp`. Точечная проверка EXE подтвердила
//! отдельные глобальные данные этого владельца: полную маску `5×5`,
//! `g_dwBesideCells == 3`
//! и адреса `0x006A16C8..0x006A16EC`. После основной площади навык обходит две
//! трёхклеточные дуги вокруг исходной цели; каждая допустимая цель отдельно
//! потребляет вызовы RNG для физического урона и критического удара. `CGame`
//! только чередует владельца региона
//! с немедленными последствиями смерти. Совпадающий `CLordWiderangingAttack`
//! использует тот же узкий семейный путь исполнения с собственным ID. Варианты игрока
//! остаются RAW ниже.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\machinerystomp.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\machinerystomp.h

// ============================================================================
// FUNCTION: CMachineryStomp::CMachineryStomp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\machinerystomp.cpp:30
// RVA: 0x001312B0
// ADDRESS: 005312b0
// PROTOTYPE: undefined __thiscall CMachineryStomp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMachineryStomp::~CMachineryStomp
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\machinerystomp.cpp:38
// RVA: 0x00131320
// ADDRESS: 00531320
// PROTOTYPE: void __thiscall ~CMachineryStomp(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMachineryStomp::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Объектный вход монстра материализован `prepare_owned_machinery_stomp`;
// координатные варианты и варианты игрока остаются в телах ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\machinerystomp.cpp:141
// RVA: 0x00131340
// ADDRESS: 00531340
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMachineryStomp::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\machinerystomp.cpp:161
// RVA: 0x00131420
// ADDRESS: 00531420
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMachineryStomp::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\machinerystomp.cpp:120
// RVA: 0x00131520
// ADDRESS: 00531520
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMachineryStompEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// Действия 0/1 монстра формирует достигнутый runtime-владелец; клиентские
// ошибки игрока остаются исходным материалом.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\machinerystomp.cpp:637
// RVA: 0x001315F0
// ADDRESS: 005315f0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMachineryStomp::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// Перезарядка, дальность, `BLOCK_UNFLY` и блокировка движения для достигнутого
// входа монстра выполняются `prepare_owned_machinery_stomp`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\machinerystomp.cpp:45
// RVA: 0x00131B80
// ADDRESS: 00531b80
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMachineryStomp::GetOutsideCells
// STATUS: IMPLEMENTED
// Точная signed-граница и обе трёхклеточные дуги реализованы `outside_cells`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\machinerystomp.cpp:360
// RVA: 0x00131D70
// ADDRESS: 00531d70
// PROTOTYPE: void __thiscall GetOutsideCells(long param_1, long param_2, long param_3, long param_4, vector<CSkill::tagCell,std::allocator<CSkill::tagCell>_> * param_5)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMachineryStomp::CalculateAttackPower
// STATUS: PARTIALLY_IMPLEMENTED
// Формула монстра и оба вызова RNG реализованы `wide_arc_attack`;
// свойства игрока остаются в исходном теле.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\machinerystomp.cpp:569
// RVA: 0x00132020
// ADDRESS: 00532020
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMachineryStomp::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// Цель-монстр проходит `execute_owned_wide_arc_attack_target`; источник-игрок
// и снимок его разрешений остаются в теле ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\machinerystomp.cpp:545
// RVA: 0x00132290
// ADDRESS: 00532290
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMachineryStomp::AI
// STATUS: PARTIALLY_IMPLEMENTED
// Стадии выполнения монстра, точный порядок клеток и завершение материализованы
// рабочими вызовами ниже; вход игрока остаётся RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\machinerystomp.cpp:194
// RVA: 0x001323A0
// ADDRESS: 005323a0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//













// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{
    SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, SKILL_USAGE_TARGET_MAX_DISTANCE,
    SKILL_USAGE_USER_HIT_MODIFIER, time_reached,
};
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    monster_attack_cell_candidates, owned_monster_attackable,
    resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::ai::monsterai::approach_attack_range;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::monster::PetAttackProperties;
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
const SCOPE_HALF_SIDE: i32 = 2;
const BESIDE_CELLS: usize = 3;
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;

pub(crate) const MACHINERY_STOMP_SKILL_ID: u32 = 0x1a7;

fn send_start(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_id: u32,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(skill_id as i32);
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
    target: &CShape,
    skill_id: u32,
    skill_level: u16,
) {
    let (Ok(target_x), Ok(target_y)) = (target.get_tile_x(), target.get_tile_y()) else {
        return;
    };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(skill_id as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(target_x);
    message.add_long(target_y);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn turn_arc_step(
    delta_x: i32,
    delta_y: i32,
    radius: i32,
    second: bool,
    step_x: &mut i32,
    step_y: &mut i32,
) {
    if delta_x == radius && delta_y == radius {
        *step_x = if second { 0 } else { -1 };
        *step_y = if second { -1 } else { 0 };
    } else if delta_x == -radius {
        if delta_y == radius {
            *step_x = if second { 1 } else { 0 };
            *step_y = if second { 0 } else { -1 };
        } else if delta_y == -radius {
            *step_x = if second { 0 } else { 1 };
            *step_y = if second { 1 } else { 0 };
        }
    } else if delta_x == radius && delta_y == -radius {
        *step_x = if second { -1 } else { 0 };
        *step_y = if second { 0 } else { 1 };
    }
}

fn append_arc(
    cells: &mut Vec<(i32, i32)>,
    source_x: i32,
    source_y: i32,
    target_x: i32,
    target_y: i32,
    radius: i32,
    mut step_x: i32,
    mut step_y: i32,
    second: bool,
) {
    let (mut x, mut y) = (target_x, target_y);
    for _ in 0..BESIDE_CELLS {
        turn_arc_step(
            x.wrapping_sub(source_x),
            y.wrapping_sub(source_y),
            radius,
            second,
            &mut step_x,
            &mut step_y,
        );
        x = x.wrapping_add(step_x);
        y = y.wrapping_add(step_y);
        cells.push((x, y));
    }
}

fn outside_cells(
    source_x: i32,
    source_y: i32,
    target_x: i32,
    target_y: i32,
) -> Vec<(i32, i32)> {
    let delta_x = target_x.wrapping_sub(source_x);
    let delta_y = target_y.wrapping_sub(source_y);
    // Исходная проверка намеренно использует signed delta, а не abs.
    if delta_x <= SCOPE_HALF_SIDE && delta_y <= SCOPE_HALF_SIDE {
        return Vec::new();
    }
    let mut cells = vec![(target_x, target_y)];
    let absolute_x = delta_x.wrapping_abs();
    let absolute_y = delta_y.wrapping_abs();
    let radius = absolute_x.max(absolute_y);

    let (first_x, first_y) = if absolute_x == radius {
        if absolute_y == radius { (0, 0) } else { (0, if delta_x < 1 { -1 } else { 1 }) }
    } else {
        (if delta_y < 1 { 1 } else { -1 }, 0)
    };
    append_arc(&mut cells, source_x, source_y, target_x, target_y, radius, first_x, first_y, false);

    let (second_x, second_y) = if absolute_x == radius {
        if absolute_y == radius { (0, 0) } else { (0, if delta_x < 1 { 1 } else { -1 }) }
    } else {
        (if delta_y < 1 { -1 } else { 1 }, 0)
    };
    append_arc(&mut cells, source_x, source_y, target_x, target_y, radius, second_x, second_y, true);
    cells
}

#[derive(Clone, Debug)]
pub(crate) struct WideArcAttackDispatch {
    pub(crate) monster_id: i32,
    pub(crate) cells: Vec<(i32, i32)>,
    skill_id: u32,
    skill_level: u16,
    properties: CSkillBaseProperties,
    property: MonsterProperties,
    attacker_master: MasterInfo,
    attacker_tamed: bool,
    pet_attack: Option<PetAttackProperties>,
    now_ms: u32,
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель выбора ИИ и текущий такт")]
pub(crate) fn prepare_owned_wide_arc_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_id: u32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    _runtime: &mut Runtime,
    dispatch: &mut Option<WideArcAttackDispatch>,
) -> bool {
    let Some((source, property, master, tamed, pet_attack, cast, last_used_ms)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            Some((
                monster.move_shape().shape().clone(),
                property.clone(),
                monster.master_info(),
                monster.is_tamed(),
                monster.is_tamed().then(|| monster.pet_attack_properties(&property)),
                monster.base_attack_cast(),
                monster.last_base_attack_ms(),
            ))
        })
    else {
        return false;
    };
    let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity) else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
            monster.clear_ai_target();
        }
        return true;
    };
    if target.dead {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
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

    if cast.is_none() {
        if !approach_attack_range(
            game,
            region,
            monster_id,
            target_x,
            target_y,
            properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE),
            now_ms,
        ) {
            return true;
        }
        if last_used_ms != 0
            && !time_reached(
                now_ms,
                last_used_ms,
                properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME),
            )
        {
            return true;
        }
        let path = region.straight_skill_path(source_x, source_y, target_x, target_y, None);
        if path.iter().any(|cell| cell.2 == BLOCK_UNFLY) {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(
                target_identity,
                skill_id,
                skill_level,
                now_ms,
            );
        }
        let source = region
            .find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape())
            .unwrap_or(&source);
        send_start(game, region, source, skill_id, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение механического топота проверено выше");
    if cast.dispatch().skill_id != skill_id
        || cast.dispatch().target != target_identity
    {
        return false;
    }
    if !time_reached(
        now_ms,
        cast.started_at_ms(),
        properties.query_property(SKILL_USAGE_DELAY_TIME),
    ) {
        return true;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
    }
    send_fire(game, region, &source, &target.shape, skill_id, skill_level);
    let mut cells = Vec::with_capacity(32);
    for offset_x in -SCOPE_HALF_SIDE..=SCOPE_HALF_SIDE {
        for offset_y in -SCOPE_HALF_SIDE..=SCOPE_HALF_SIDE {
            cells.push((
                source_x.wrapping_add(offset_x),
                source_y.wrapping_add(offset_y),
            ));
        }
    }
    cells.extend(outside_cells(source_x, source_y, target_x, target_y));
    *dispatch = Some(WideArcAttackDispatch {
        monster_id,
        cells,
        skill_id,
        skill_level,
        properties: properties.clone(),
        property,
        attacker_master: master,
        attacker_tamed: tamed,
        pet_attack,
        now_ms,
    });
    true
}

fn wide_arc_attack(
    game: &mut CGame,
    region: &CServerRegion,
    dispatch: &WideArcAttackDispatch,
) -> AttackInformation {
    let (minimum, maximum) = region
        .find_monster_by_id(dispatch.monster_id)
        .map(|monster| {
            let bounds = monster.state_attack_bounds(
                dispatch.property.minimum_attack,
                dispatch.property.maximum_attack,
            );
            dispatch
                .pet_attack
                .map_or(bounds, |pet| (pet.minimum_attack, pet.maximum_attack))
        })
        .unwrap_or((dispatch.property.minimum_attack, dispatch.property.maximum_attack));
    let minimum = minimum as i32;
    let maximum = maximum as i32;
    let span = maximum.wrapping_sub(minimum).unsigned_abs().wrapping_add(1) as i32;
    let physical = minimum.wrapping_add(game.skill_random_below(span)).max(0);
    let _critical_roll = game.skill_random_below(100);
    AttackInformation {
        skill_id: dispatch.skill_id,
        skill_level: dispatch.skill_level as u8,
        attacker_type: MONSTER_TYPE,
        attacker_id: dispatch.monster_id,
        attacker_team_id: 0,
        attacker_faction_id: 0,
        attacker_union_id: 0,
        hit_modifier: dispatch.properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32,
        damage_factor: dispatch.properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR) as f32 * 0.01,
        damage_modifier: 0,
        critical: false,
        blast_attack: false,
        full_miss: 0,
        damages: vec![
            AttackPower { kind: AttackPowerType::Physical, hp_damage: physical, mp_damage: 0 },
            AttackPower { kind: AttackPowerType::Element, hp_damage: 0, mp_damage: 0 },
            AttackPower {
                kind: AttackPowerType::Soul,
                hp_damage: (dispatch.property.yao_attack & 0xffff) as i32,
                mp_damage: 0,
            },
        ],
    }
}

pub(crate) fn wide_arc_attack_cell_candidates(
    game: &CGame,
    region: &CServerRegion,
    dispatch: &WideArcAttackDispatch,
    tile_x: i32,
    tile_y: i32,
) -> Vec<ShapeIdentity> {
    monster_attack_cell_candidates(game, region, dispatch.monster_id, tile_x, tile_y)
}

pub(crate) fn execute_owned_wide_arc_attack_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    dispatch: &WideArcAttackDispatch,
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
    let attack = wide_arc_attack(game, region, dispatch);
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

pub(crate) fn finish_owned_wide_arc_attack(
    region: &mut CServerRegion,
    dispatch: &WideArcAttackDispatch,
) {
    if let Some(monster) = region.find_monster_by_id_mut(dispatch.monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        monster.move_shape_mut().shape_mut().set_action(1);
        monster.move_shape_mut().set_moveable(true);
        let _ = monster.finish_base_attack_cast(dispatch.now_ms);
    }
}

#[allow(clippy::too_many_arguments, reason = "обёртка сохраняет конкретного владельца навыка")]
pub(crate) fn prepare_owned_machinery_stomp<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
    dispatch: &mut Option<WideArcAttackDispatch>,
) -> bool {
    prepare_owned_wide_arc_attack(
        game,
        region,
        monster_id,
        target_identity,
        MACHINERY_STOMP_SKILL_ID,
        skill_level,
        properties,
        now_ms,
        runtime,
        dispatch,
    )
}
