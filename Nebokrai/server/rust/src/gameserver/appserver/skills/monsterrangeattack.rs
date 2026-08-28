//! Дальняя круговая атака монстра (`CMonsterRangeAttack`).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/skills/monsterrangeattack.cpp`. Достигнутый путь монстра хранит
//! стадии навыка и формулу здесь, а `CGame` координирует владельцев региона и
//! игроков. Исходная область — подтверждённая из EXE маска 7x7; клетки
//! обходятся сначала по X, затем по Y, а успешно допущенные цели дедуплицируются
//! после проверки `IsAttackAble`. Ветви игрока с MP, critical и сообщениями об
//! ошибках пока сохраняются как RAW, поскольку реальный вызов игроком не найден.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterrangeattack.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterrangeattack.h

// ============================================================================
// FUNCTION: CMonsterRangeAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterrangeattack.cpp:131
// RVA: 0x00111620
// ADDRESS: 00511620
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterRangeAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterrangeattack.cpp:147
// RVA: 0x001116F0
// ADDRESS: 005116f0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterRangeAttack::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterrangeattack.cpp:114
// RVA: 0x001117E0
// ADDRESS: 005117e0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterRangeAttackEffect::UpdateVisualEffect
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterrangeattack.cpp:402
// RVA: 0x001118A0
// ADDRESS: 005118a0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterRangeAttack::CheckCastCondition
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterrangeattack.cpp:45
// RVA: 0x00111CF0
// ADDRESS: 00511cf0
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterRangeAttack::CalculateAttackPower
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterrangeattack.cpp:338
// RVA: 0x00112170
// ADDRESS: 00512170
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterRangeAttack::Attack
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterrangeattack.cpp:314
// RVA: 0x001123E0
// ADDRESS: 005123e0
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CMonsterRangeAttack::AI
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\monsterrangeattack.cpp:176
// RVA: 0x00112500
// ADDRESS: 00512500
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

























// COMPONENT_VARIANT_END: GameServer

use super::baseattack::SKILL_USAGE_USER_HIT_MODIFIER;
use super::basemagic::{SKILL_USAGE_MAX_ATTACK, SKILL_USAGE_MIN_ATTACK};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{
    ShapeIdentity, ShapeResolver, ShapeView,
};
use crate::gameserver::appserver::skills::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::appserver::skills::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit,
    defend_owned_monster_attack, owned_monster_attackable,
    resolve_owned_monster_attack_target,
};
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const RANGE_SCOPE_SIDE: i32 = 7;
const SKILL_USAGE_EM_MODIFIER: u32 = 20_015;

pub(crate) const MONSTER_RANGE_ATTACK_SKILL_ID: u32 = 0x2ef;

// `g_bScope` по адресу 0x006A0ECC при `g_dwLength/g_dwHeight == 7`.
const RANGE_SCOPE: [u8; 49] = [
    0, 0, 1, 1, 1, 0, 0,
    0, 1, 1, 1, 1, 1, 0,
    1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1,
    1, 1, 1, 1, 1, 1, 1,
    0, 1, 1, 1, 1, 1, 0,
    0, 0, 1, 1, 1, 0, 0,
];

struct RangeShapeResolver<'a> {
    game: &'a CGame,
    region: &'a CServerRegion,
}

impl ShapeResolver for RangeShapeResolver<'_> {
    fn resolve_shape(&self, identity: ShapeIdentity) -> Option<ShapeView> {
        match identity.object_type {
            PLAYER_TYPE => self.game.find_player(identity.id)?.shape_view(),
            MONSTER_TYPE => {
                let monster = self.region.find_monster_by_id(identity.id)?;
                let property = self
                    .game
                    .find_monster_property_by_origin_name(monster.base_property_key()?)?;
                monster.shape_view(property)
            }
            _ => None,
        }
    }
}

pub(crate) fn range_attack_fire_message(
    skill_level: u16,
    monster_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> CMessage {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(MONSTER_RANGE_ATTACK_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(tile_x);
    message.add_long(tile_y);
    message
}

pub(crate) fn range_attack_scope_cells() -> impl Iterator<Item = (i32, i32)> {
    (0..RANGE_SCOPE_SIDE).flat_map(|x| {
        (0..RANGE_SCOPE_SIDE).filter_map(move |y| {
            let index = (x + RANGE_SCOPE_SIDE * y) as usize;
            (RANGE_SCOPE[index] != 0).then_some((x - 3, y - 3))
        })
    })
}

/// Возвращает исходный упорядоченный снимок одной клетки. Следующая клетка
/// читается только после применения предыдущих ударов и их последствий смерти.
pub(crate) fn range_attack_cell_candidates(
    game: &CGame,
    region: &CServerRegion,
    monster_id: i32,
    tile_x: i32,
    tile_y: i32,
) -> Vec<ShapeIdentity> {
    let (area_width, area_height) = game.area_dimensions();
    let resolver = RangeShapeResolver { game, region };
    let mut targets = Vec::new();
    let mut shapes = Vec::new();
    if region
        .get_shapes(
            tile_x,
            tile_y,
            area_width,
            area_height,
            &resolver,
            &mut shapes,
        )
        .is_err()
    {
        return targets;
    }
    for shape in shapes {
        if !matches!(shape.identity.object_type, PLAYER_TYPE | MONSTER_TYPE) {
            continue;
        }
        if shape.identity.object_type == MONSTER_TYPE && shape.identity.id == monster_id {
            continue;
        }
        targets.push(shape.identity);
    }
    targets
}

pub(crate) fn calculate_monster_range_attack(
    properties: &CSkillBaseProperties,
    skill_level: u16,
    monster_id: i32,
    random_below: &mut dyn FnMut(i32) -> i32,
) -> crate::gameserver::appserver::states::attackpower::AttackInformation {
    use crate::gameserver::appserver::states::attackpower::{
        AttackInformation, AttackPower, AttackPowerType,
    };

    let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
    let maximum = properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32;
    let skill_span = maximum
        .wrapping_sub(minimum)
        .unsigned_abs()
        .wrapping_add(1) as i32;
    let skill_damage = minimum.wrapping_add(random_below(skill_span));
    let _element_modifier = properties.query_property(SKILL_USAGE_EM_MODIFIER);

    AttackInformation {
        skill_id: MONSTER_RANGE_ATTACK_SKILL_ID,
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
            // Виртуальный `CMonster::GetAddElementAtk` возвращает ноль;
            // `SKILL_USAGE_EM_MODIFIER` умножается на нулевой ElementModify.
            hp_damage: skill_damage.max(0),
            mp_damage: 0,
        }],
    }
}

#[derive(Clone, Debug)]
pub(crate) struct MonsterRangeAttackDispatch {
    pub(crate) monster_id: i32,
    pub(crate) skill_level: u16,
    properties: CSkillBaseProperties,
    property: crate::setup::monsterlist::MonsterProperties,
    attacker_master: crate::gameserver::appserver::masterinfo::MasterInfo,
    attacker_tamed: bool,
    pub(crate) center_x: i32,
    pub(crate) center_y: i32,
    now_ms: u32,
}

pub(crate) fn prepare_owned_monster_range_cast(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    dispatch: &mut Option<MonsterRangeAttackDispatch>,
) -> bool {
    let Some((shape, property, cast, attacker_master, attacker_tamed)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            Some((
                monster.move_shape().shape().clone(),
                game.find_monster_property_by_origin_name(monster.base_property_key()?)?.clone(),
                monster.base_attack_cast()?,
                monster.master_info(),
                monster.is_tamed(),
            ))
        })
    else {
        return false;
    };
    if cast.dispatch().skill_id != MONSTER_RANGE_ATTACK_SKILL_ID {
        return false;
    }
    let delay_ms = properties.query_property(super::baseattack::SKILL_USAGE_DELAY_TIME);
    if !super::baseattack::time_reached(now_ms, cast.started_at_ms(), delay_ms) {
        return true;
    }
    let (Ok(tile_x), Ok(tile_y)) = (shape.get_tile_x(), shape.get_tile_y()) else {
        return true;
    };
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
    }
    let fire = range_attack_fire_message(
        cast.dispatch().skill_level,
        monster_id,
        tile_x,
        tile_y,
    );
    let _ = game.send_game_shape_around(region, &shape, None, &fire);
    *dispatch = Some(MonsterRangeAttackDispatch {
        monster_id,
        skill_level: cast.dispatch().skill_level,
        properties: properties.clone(),
        property,
        attacker_master,
        attacker_tamed,
        center_x: tile_x,
        center_y: tile_y,
        now_ms,
    });
    true
}

pub(crate) fn execute_owned_monster_range_target<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    dispatch: &MonsterRangeAttackDispatch,
    identity: ShapeIdentity,
    runtime: &mut Runtime,
    deaths: &mut Vec<MonsterAttackDeath>,
) -> bool {
    let Some(target) = resolve_owned_monster_attack_target(game, region, identity) else {
        return false;
    };
    if target.dead || target.god || target.city_dead {
        return false;
    }
    if !owned_monster_attackable(
        game,
        region.id,
        &dispatch.property,
        dispatch.attacker_tamed,
        dispatch.attacker_master,
        identity,
        &target,
    ) {
        return false;
    }
    let mut random = |maximum| game.skill_random_below(maximum);
    let attack = calculate_monster_range_attack(
        &dispatch.properties,
        dispatch.skill_level,
        dispatch.monster_id,
        &mut random,
    );
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

pub(crate) fn finish_owned_monster_range_cast(
    region: &mut CServerRegion,
    dispatch: &MonsterRangeAttackDispatch,
) {
    if let Some(monster) = region.find_monster_by_id_mut(dispatch.monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        monster.move_shape_mut().shape_mut().set_action(1);
        let _ = monster.finish_base_attack_cast(dispatch.now_ms);
    }
}
