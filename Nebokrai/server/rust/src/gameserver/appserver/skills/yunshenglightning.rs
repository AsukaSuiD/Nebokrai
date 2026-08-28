//! Владелец навыка `CYunShengLightning` (`0x19E`). Достигнутый путь монстра
//! хранит собственные задержки, формулу стихийного урона и пакеты;
//! общие допустимость цели, защита и последствия смерти использует из боевого
//! владельца. Расход MP и отличающиеся варианты игрока ниже остаются RAW.
//! Декомпилятор: Ghidra 12.1.2
//! Полный декомпилят хранится локально и не входит в распространяемый код.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\yunshenglightning.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\yunshenglightning.h

// ============================================================================
// FUNCTION: CYunShengLightning::CYunShengLightning
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\yunshenglightning.cpp:19
// RVA: 0x0013A520
// ADDRESS: 0053a520
// PROTOTYPE: undefined __thiscall CYunShengLightning(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYunShengLightning::~CYunShengLightning
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\yunshenglightning.cpp:29
// RVA: 0x0013A590
// ADDRESS: 0053a590
// PROTOTYPE: void __thiscall ~CYunShengLightning(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYunShengLightning::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\yunshenglightning.cpp:137
// RVA: 0x0013A5B0
// ADDRESS: 0053a5b0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYunShengLightning::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\yunshenglightning.cpp:156
// RVA: 0x0013A680
// ADDRESS: 0053a680
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYunShengLightning::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован ниже;
// отличающиеся ветви игрока и недостигнутого вызова сохранены в RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\yunshenglightning.cpp:117
// RVA: 0x0013A770
// ADDRESS: 0053a770
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYunShengLightningEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован ниже;
// отличающиеся ветви игрока и недостигнутого вызова сохранены в RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\yunshenglightning.cpp:438
// RVA: 0x0013A830
// ADDRESS: 0053a830
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYunShengLightning::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован ниже;
// отличающиеся ветви игрока и недостигнутого вызова сохранены в RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\yunshenglightning.cpp:38
// RVA: 0x0013ACD0
// ADDRESS: 0053acd0
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYunShengLightning::CalculateAttackPower
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован ниже;
// отличающиеся ветви игрока и недостигнутого вызова сохранены в RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\yunshenglightning.cpp:378
// RVA: 0x0013AE70
// ADDRESS: 0053ae70
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYunShengLightning::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован ниже;
// отличающиеся ветви игрока и недостигнутого вызова сохранены в RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\yunshenglightning.cpp:354
// RVA: 0x0013B010
// ADDRESS: 0053b010
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CYunShengLightning::AI
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован ниже;
// отличающиеся ветви игрока и недостигнутого вызова сохранены в RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\yunshenglightning.cpp:191
// RVA: 0x0013B130
// ADDRESS: 0053b130
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//














// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_USER_HIT_MODIFIER, time_reached};
use super::monsterattack::{
    MonsterAttackDeath, apply_owned_monster_attack_hit, defend_owned_monster_attack,
    owned_monster_attackable, resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
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
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;
const SKILL_USAGE_TARGET_FINAL_DAMAGE_MODIFIER: u32 = 20_002;
const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;
pub(crate) const YUNSHENG_LIGHTNING_SKILL_ID: u32 = 0x19e;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct YunShengLightningProgress {
    destination_x: i32,
    destination_y: i32,
    flying_time_ms: u32,
    fired: bool,
}

impl YunShengLightningProgress {
    pub(crate) const fn new(destination_x: i32, destination_y: i32) -> Self {
        Self {
            destination_x,
            destination_y,
            flying_time_ms: 0,
            fired: false,
        }
    }

    pub(crate) const fn fired(self) -> bool {
        self.fired
    }

    pub(crate) const fn destination(self) -> (i32, i32) {
        (self.destination_x, self.destination_y)
    }

    pub(crate) const fn flying_time_ms(self) -> u32 {
        self.flying_time_ms
    }

    pub(crate) fn fire(
        &mut self,
        destination_x: i32,
        destination_y: i32,
        flying_time_ms: u32,
    ) {
        self.destination_x = destination_x;
        self.destination_y = destination_y;
        self.flying_time_ms = flying_time_ms;
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
    message.add_long(YUNSHENG_LIGHTNING_SKILL_ID as i32);
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
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(YUNSHENG_LIGHTNING_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(target.map_or(0, |target| target.object_type));
    message.add_long(target.map_or(0, |target| target.id));
    message.add_long(target_x);
    message.add_long(target_y);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт полёта")]
pub(crate) fn execute_owned_yunsheng_lightning<Runtime: GameMainLoopRuntime>(
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
                monster.yunsheng_lightning_progress(),
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
    let target_coordinates = target.as_ref().and_then(|target| {
        Some((target.shape.get_tile_x().ok()?, target.shape.get_tile_y().ok()?))
    });
    let Some((target_x, target_y)) = target_coordinates
        .or_else(|| progress.map(YunShengLightningProgress::destination))
    else {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return true;
    };
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    let path = region.straight_skill_path(source_x, source_y, target_x, target_y, None);

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
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.begin_base_attack_cast(
                target_identity, YUNSHENG_LIGHTNING_SKILL_ID, skill_level, now_ms,
            );
            monster.set_yunsheng_lightning_progress(YunShengLightningProgress::new(
                target_x, target_y,
            ));
        }
        send_start(game, region, &source, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение молнии проверено выше");
    if cast.dispatch().skill_id != YUNSHENG_LIGHTNING_SKILL_ID {
        return false;
    }
    let Some(mut progress) = progress else { return true };
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    if !progress.fired() {
        if !time_reached(now_ms, cast.started_at_ms(), delay_ms) {
            return true;
        }
        if maximum_distance != 0 && path.len() > maximum_distance as usize {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        }
        send_fire(
            game,
            region,
            &source,
            skill_level,
            target.as_ref().map(|_| target_identity),
            target_x,
            target_y,
        );
        let flying_time_ms = properties.query_property(SKILL_USAGE_MISSILE_FLYING_TIME);
        progress.fire(target_x, target_y, flying_time_ms);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.set_yunsheng_lightning_progress(progress);
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        }
    }
    if !time_reached(
        now_ms,
        cast.started_at_ms(),
        delay_ms.wrapping_add(progress.flying_time_ms()),
    ) {
        return true;
    }

    if let Some(target) = target
        && !target.dead
        && !target.god
        && !target.city_dead
        && owned_monster_attackable(
            game, region.id, &property, tamed, master, target_identity, &target,
        )
    {
        // Windows `CMonster::GetAddElementAtk` возвращает ноль даже для
        // приручённого монстра; поэтому здесь остаётся ровно один RNG-вызов.
        let base_element = 0_i32;
        let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
        let skill_span = (properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32)
            .wrapping_sub(minimum)
            .unsigned_abs()
            .wrapping_add(1) as i32;
        let skill_element = minimum.wrapping_add(game.skill_random_below(skill_span));
        let attack = AttackInformation {
            skill_id: YUNSHENG_LIGHTNING_SKILL_ID,
            skill_level: skill_level as u8,
            attacker_type: MONSTER_TYPE,
            attacker_id: monster_id,
            attacker_team_id: 0,
            attacker_faction_id: 0,
            attacker_union_id: 0,
            hit_modifier: properties.query_property(SKILL_USAGE_USER_HIT_MODIFIER) as i32,
            damage_factor: 1.0,
            damage_modifier: properties
                .query_property(SKILL_USAGE_TARGET_FINAL_DAMAGE_MODIFIER) as i32,
            critical: false,
            blast_attack: false,
            full_miss: 0,
            damages: vec![AttackPower {
                kind: AttackPowerType::Element,
                hp_damage: base_element.wrapping_add(skill_element).max(0),
                mp_damage: 0,
            }],
        };
        let attack = defend_owned_monster_attack(
            game, target_identity, target.mana, target.war_soul_mana,
            target.player_properties, target.monster_properties, attack,
        );
        apply_owned_monster_attack_hit(
            game, region, runtime, now_ms, monster_id, master, target_identity,
            &target.shape, target.health, target.mana, target.master,
            target.monster_property, target.tamed, target.carriage, attack, deaths,
        );
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}
