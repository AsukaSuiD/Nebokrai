//! Паучий туман `CSpiderMist` (`0x198`) для достигнутого пути монстра и питомца.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/spidermist.cpp`. Навык сохраняет координаты цели в момент
//! `Begin`, прямой путь и `BLOCK_UNFLY`, задержку с запретом движения, точные
//! `0xBFE01` и создание `CSpiderMistPhalanx`. После успешного сближения
//! attack-speed расписания и reuse навыка проверяются раздельно. Формулы области и яда остаются
//! у соответствующих владельцев; `CGame` выполняет только регистрацию в регионе
//! и доставку. Перегрузки для игрока и координатной цели остаются RAW ниже.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spidermist.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spidermist.h

// ============================================================================
// FUNCTION: CSpiderMist::CSpiderMist
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spidermist.cpp:20
// RVA: 0x00140660
// ADDRESS: 00540660
// PROTOTYPE: undefined __thiscall CSpiderMist(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderMist::~CSpiderMist
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spidermist.cpp:28
// RVA: 0x001406D0
// ADDRESS: 005406d0
// PROTOTYPE: void __thiscall ~CSpiderMist(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderMist::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spidermist.cpp:117
// RVA: 0x001406F0
// ADDRESS: 005406f0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderMist::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spidermist.cpp:133
// RVA: 0x001407C0
// ADDRESS: 005407c0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderMist::Begin(CMoveShape *, CMoveShape *)
// STATUS: IMPLEMENTED
// Достигнутый вход монстра и питомца сохраняет координаты объекта-цели и передаёт
// стадии владельцу `execute_owned_spider_mist`.

// ============================================================================
// FUNCTION: CSpiderMist::AI
// STATUS: IMPLEMENTED
// Задержка, блокировка движения, Check→Calculate→Attack→Apply, создание
// фаланги и завершение каста принадлежат `execute_owned_spider_mist`.

// ============================================================================
// FUNCTION: CSpiderMistEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// Действия начала и срабатывания достигнутого пути монстра и питомца материализованы;
// пакеты отказа только для игрока остаются в RAW-теле до появления вызывающей стороны.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spidermist.cpp:309
// RVA: 0x00140AB0
// ADDRESS: 00540ab0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderMist::CheckCastCondition
// STATUS: IMPLEMENTED
// Достигнутый путь монстра и питомца сохраняет задержку повторного применения,
// прямой путь, максимальную
// дальность, запрет `BLOCK_UNFLY` и блокировку движения до завершения задержки.

// ============================================================================
// FUNCTION: CSpiderMist::Summon
// STATUS: PARTIALLY_IMPLEMENTED
// Создание, регистрация фаланги монстра и пакет входа материализованы.
// Данные владельца-игрока и вычитание перекрытия из уже существующего
// `SKILL_POISON_FOG` остаются в RAW-теле до достижения этих владельцев.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spidermist.cpp:232
// RVA: 0x00141140
// ADDRESS: 00541140
// PROTOTYPE: int __thiscall Summon(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//










// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{SKILL_USAGE_DELAY_TIME, time_reached};
use super::basemagic::SKILL_USAGE_TARGET_MAX_DISTANCE;
use super::skillbaseproperties::CSkillBaseProperties;
use super::spidermistphalanx::CSpiderMistPhalanx;
use crate::gameserver::appserver::ai::monsterai::{
    approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const MONSTER_TYPE: i32 = 600;
const BLOCK_UNFLY: u8 = 2;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_CONST: u32 = 20_010;
const SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME: u32 = 30_001;
pub(crate) const SPIDER_MIST_SKILL_ID: u32 = 0x198;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpiderMistProgress {
    pub(crate) destination_x: i32,
    pub(crate) destination_y: i32,
}

fn send_start(
    game: &CGame,
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(SPIDER_MIST_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_fire(
    game: &CGame,
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_level: u16,
    destination_x: i32,
    destination_y: i32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(SPIDER_MIST_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(destination_x);
    message.add_long(destination_y);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_phalanx_entry(game: &CGame, region: &CServerRegion, phalanx_id: i32) {
    let Some(crate::gameserver::appserver::summonshape::SummonedSkillShape::SpiderMist(phalanx)) =
        region.find_skill_phalanx(phalanx_id)
    else {
        return;
    };
    let Some(payload) = phalanx.encode_client_snapshot() else { return };
    let identity = phalanx.shape().identity();
    let mut message = CMessage::new(0x000b_f502);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.base_mut().add_guid(identity.ex_id);
    message.add_long(payload.len() as i32);
    message.base_mut().add(&payload);
    message.base_mut().add_char(0);
    let _ = game.send_game_shape_around(region, phalanx.shape(), None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт исходного навыка")]
pub(crate) fn execute_owned_spider_mist<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    let Some((source_shape, cast, progress, last_used_ms, ai_type, attack_interval)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?;
            let attack_interval = if monster.is_tamed() {
                monster.pet_attack_properties(property).attack_interval
            } else {
                property.attack_speed
            };
            Some((
                monster.move_shape().shape().clone(),
                monster.base_attack_cast(),
                monster.spider_mist_progress(),
                monster.skill_last_used_ms(SPIDER_MIST_SKILL_ID),
                property.ai,
                attack_interval,
            ))
        })
    else {
        return false;
    };
    if let Some(cast) = cast {
        if cast.dispatch().skill_id != SPIDER_MIST_SKILL_ID {
            return false;
        }
        let Some(progress) = progress else {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.move_shape_mut().set_moveable(true);
                monster.cancel_base_attack_cast();
            }
            return true;
        };
        if !time_reached(
            now_ms,
            cast.started_at_ms(),
            properties.query_property(SKILL_USAGE_DELAY_TIME),
        ) {
            return true;
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        }
        send_fire(
            game,
            region,
            &source_shape,
            monster_id,
            skill_level,
            progress.destination_x,
            progress.destination_y,
        );
        let phalanx_started_at_ms = runtime.now_milliseconds();
        let phalanx = CSpiderMistPhalanx::new(
            game.allocate_summon_shape_id(),
            MasterInfo {
                master_type: MONSTER_TYPE,
                master_id: monster_id,
                ..MasterInfo::default()
            },
            phalanx_started_at_ms,
            properties.query_property(SKILL_USAGE_SUMMONED_CREATURE_LIFE_TIME),
            i32::from(skill_level),
            properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
            properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY),
            properties.query_property(SKILL_USAGE_CONST),
        );
        let (area_width, area_height) = game.area_dimensions();
        if let Ok(phalanx_id) = region.add_spider_mist_phalanx(
            phalanx,
            progress.destination_x,
            progress.destination_y,
            area_width,
            area_height,
            phalanx_started_at_ms,
            runtime,
        ) {
            send_phalanx_entry(game, region, phalanx_id);
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
            let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
            let _ = monster.finish_base_attack_cast(now_ms);
        }
        return true;
    }

    let target_shape = match target.object_type {
        400 => game.find_player(target.id).and_then(|player| {
            (player.server_region_id() == Some(region.id)).then(|| player.shape().clone())
        }),
        MONSTER_TYPE => region
            .find_monster_by_id(target.id)
            .map(|monster| monster.move_shape().shape().clone()),
        _ => None,
    };
    let Some(target_shape) = target_shape else { return true };
    let (Ok(source_x), Ok(source_y), Ok(destination_x), Ok(destination_y)) = (
        source_shape.get_tile_x(),
        source_shape.get_tile_y(),
        target_shape.get_tile_x(),
        target_shape.get_tile_y(),
    ) else { return true };
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);
    if !approach_attack_range(
        game, region, monster_id, destination_x, destination_y, maximum_distance, now_ms,
    ) {
        return true;
    }
    let path = region.straight_skill_path(source_x, source_y, destination_x, destination_y, None);
    if (maximum_distance != 0 && path.len() > maximum_distance as usize)
        || path.iter().any(|cell| cell.2 == BLOCK_UNFLY)
    {
        return true;
    }
    let schedule_ready = schedule_attack_interval(ai_type, attack_interval).is_none_or(|interval| {
        region
            .find_monster_by_id_mut(monster_id)
            .is_some_and(|monster| monster.begin_ai_attack_attempt(now_ms, interval))
    });
    if !schedule_ready {
        return true;
    }
    let reuse_delay = properties.query_property(SKILL_USAGE_REUSE_DELAY_TIME);
    if last_used_ms != 0 && !time_reached(now_ms, last_used_ms, reuse_delay) {
        return true;
    }
    let direction = get_line_direction(source_x, source_y, destination_x, destination_y);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().shape_mut().set_direction(direction);
        monster.move_shape_mut().set_moveable(false);
        monster.begin_base_attack_cast(target, SPIDER_MIST_SKILL_ID, skill_level, now_ms);
        monster.set_spider_mist_progress(SpiderMistProgress { destination_x, destination_y });
    }
    let source = region
        .find_monster_by_id(monster_id)
        .map(|monster| monster.move_shape().shape())
        .unwrap_or(&source_shape);
    send_start(game, region, source, monster_id, skill_level);
    true
}
