//! Малая звезда `CLittleStar` (`0x1a4`) для достигнутого пути монстра.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает однократное
//! построение прямого пути после задержки, остановку на `BLOCK_UNFLY`, повторные
//! проходы по всем клеткам с заданной частотой и строгий конец длительности.
//! Каждая допустимая цель получает отдельный исходный RNG-вызов. Ветвь игрока
//! с расходом MP и ещё не достигнутые перегрузки входа сохранены ниже.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\littlestar.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\littlestar.h

// ============================================================================
// FUNCTION: CLittleStarEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// Действия 0/1/3 монстра достигнуты функциями `send_start`, `send_fire` и
// `send_end`; клиентские ответы об ошибках игрока остаются исходным материалом.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\littlestar.cpp:538
// RVA: 0x00134DC0
// ADDRESS: 00534dc0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLittleStar::~CLittleStar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\littlestar.cpp:30
// RVA: 0x001352F0
// ADDRESS: 005352f0
// PROTOTYPE: void __thiscall ~CLittleStar(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLittleStar::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Объектная ветвь монстра достигнута в `execute_owned_little_star`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\littlestar.cpp:111
// RVA: 0x00135350
// ADDRESS: 00535350
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLittleStar::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\littlestar.cpp:131
// RVA: 0x00135430
// ADDRESS: 00535430
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLittleStar::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\littlestar.cpp:150
// RVA: 0x00135510
// ADDRESS: 00535510
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLittleStar::End
// STATUS: PARTIALLY_IMPLEMENTED
// Завершение достигнутой ветви монстра выполняет `execute_owned_little_star`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\littlestar.cpp:169
// RVA: 0x001355F0
// ADDRESS: 005355f0
// PROTOTYPE: void __thiscall End(int param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLittleStar::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// Повторное использование монстра достигнуто; расход MP и сообщения игрока
// остаются неподключёнными.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\littlestar.cpp:39
// RVA: 0x00135760
// ADDRESS: 00535760
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLittleStar::CLittleStar
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\littlestar.cpp:21
// RVA: 0x00135940
// ADDRESS: 00535940
// PROTOTYPE: undefined __thiscall CLittleStar(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLittleStar::CalculateAttackPower
// STATUS: PARTIALLY_IMPLEMENTED
// Формула монстра достигнута в `attack_path`; отличающаяся ветвь игрока
// сохранена ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\littlestar.cpp:410
// RVA: 0x001359C0
// ADDRESS: 005359c0
// PROTOTYPE: void __thiscall CalculateAttackPower(CMoveShape * param_1, CMoveShape * param_2, tagAttackInformation * param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLittleStar::Attack
// STATUS: PARTIALLY_IMPLEMENTED
// Атака монстра достигнута в `attack_path`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\littlestar.cpp:386
// RVA: 0x00135B80
// ADDRESS: 00535b80
// PROTOTYPE: void __thiscall Attack(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CLittleStar::AI
// STATUS: PARTIALLY_IMPLEMENTED
// Длительный путь монстра достигнут в `execute_owned_little_star`; ветвь
// игрока сохранена ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\littlestar.cpp:191
// RVA: 0x00135CA0
// ADDRESS: 00535ca0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//













// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, time_reached};
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
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_SKILL_PERSIST_TIME: u32 = 10_007;
const SKILL_USAGE_MIN_ATTACK: u32 = 20_008;
const SKILL_USAGE_MAX_ATTACK: u32 = 20_009;
pub(crate) const LITTLE_STAR_SKILL_ID: u32 = 0x1a4;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LittleStarProgress {
    path: Vec<(i32, i32, u8)>,
    last_attack_ms: u32,
}

impl LittleStarProgress {
    fn new(path: Vec<(i32, i32, u8)>) -> Self {
        Self {
            path,
            last_attack_ms: 0,
        }
    }

    fn attack_due(&self, now_ms: u32, frequency_ms: u32) -> bool {
        self.last_attack_ms == 0
            || self.last_attack_ms.wrapping_add(frequency_ms) < now_ms
    }

    fn record_attack(&mut self, now_ms: u32) {
        self.last_attack_ms = now_ms;
    }
}

fn send_start(game: &CGame, region: &CServerRegion, source: &CShape, skill_level: u16) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(LITTLE_STAR_SKILL_ID as i32);
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
    target_x: i32,
    target_y: i32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(LITTLE_STAR_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(target_x);
    message.add_long(target_y);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_end(game: &CGame, region: &CServerRegion, source: &CShape, skill_level: u16) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(3);
    message.add_long(LITTLE_STAR_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет порядок клеток, целей и RNG каждого удара")]
fn attack_path<Runtime: GameMainLoopRuntime>(
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
    path: &[(i32, i32, u8)],
    deaths: &mut Vec<MonsterAttackDeath>,
) {
    for &(cell_x, cell_y, block) in path {
        if block == BLOCK_UNFLY {
            break;
        }
        for identity in monster_attack_cell_candidates(game, region, monster_id, cell_x, cell_y) {
            let Some(target) = resolve_owned_monster_attack_target(game, region, identity) else {
                continue;
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
                continue;
            }
            let minimum = properties.query_property(SKILL_USAGE_MIN_ATTACK) as i32;
            let span = (properties.query_property(SKILL_USAGE_MAX_ATTACK) as i32)
                .wrapping_sub(minimum)
                .unsigned_abs()
                .wrapping_add(1) as i32;
            let damage = minimum.wrapping_add(game.skill_random_below(span)).max(0);
            let attack = AttackInformation {
                skill_id: LITTLE_STAR_SKILL_ID,
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
        }
    }
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт длительного навыка")]
pub(crate) fn execute_owned_little_star<Runtime: GameMainLoopRuntime>(
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
                monster.little_star_progress().cloned(),
                monster.last_base_attack_ms(),
            ))
        })
    else {
        return false;
    };
    let target = resolve_owned_monster_attack_target(game, region, target_identity);
    if progress.is_none() && target.is_none() {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return true;
    }
    if cast.is_none()
        && !target.as_ref().is_some_and(|target| {
            !target.dead
                && !target.god
                && !target.city_dead
                && owned_monster_attackable(
                    game, region.id, &property, tamed, master, target_identity, target,
                )
        })
    {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.clear_ai_target();
        }
        return true;
    }
    let Ok(source_x) = source.get_tile_x() else {
        return true;
    };
    let Ok(source_y) = source.get_tile_y() else {
        return true;
    };
    let target_position = target.as_ref().and_then(|target| {
        Some((target.shape.get_tile_x().ok()?, target.shape.get_tile_y().ok()?))
    });
    let (target_x, target_y) = target_position.unwrap_or((source_x, source_y));

    if cast.is_none() {
        if last_used_ms != 0
            && !time_reached(now_ms, last_used_ms, properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME))
        {
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.begin_base_attack_cast(target_identity, LITTLE_STAR_SKILL_ID, skill_level, now_ms);
        }
        let current_source = region.find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape()).unwrap_or(&source);
        send_start(game, region, current_source, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение малой звезды проверено выше");
    if cast.dispatch().skill_id != LITTLE_STAR_SKILL_ID || cast.dispatch().target != target_identity {
        return false;
    }
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    if !time_reached(now_ms, cast.started_at_ms(), delay_ms) {
        return true;
    }

    let mut progress = if let Some(progress) = progress {
        progress
    } else {
        let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE) as usize;
        let mut path = region.straight_skill_path(source_x, source_y, target_x, target_y, None);
        path.truncate(maximum_distance);
        send_fire(game, region, &source, skill_level, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        }
        LittleStarProgress::new(path)
    };

    if progress.attack_due(now_ms, properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY)) {
        attack_path(
            game, region, runtime, now_ms, monster_id, skill_level, properties, &property,
            master, tamed, &progress.path, deaths,
        );
        progress.record_attack(runtime.now_milliseconds());
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        }
    }

    let expiration_now_ms = runtime.now_milliseconds();
    let expired = cast.started_at_ms()
        .wrapping_add(delay_ms)
        .wrapping_add(properties.query_property(SKILL_USAGE_SKILL_PERSIST_TIME))
        < expiration_now_ms;
    if expired {
        send_end(game, region, &source, skill_level);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
            monster.move_shape_mut().set_moveable(true);
            let _ = monster.finish_base_attack_cast(expiration_now_ms);
        }
    } else if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.set_little_star_progress(progress);
    }
    true
}
