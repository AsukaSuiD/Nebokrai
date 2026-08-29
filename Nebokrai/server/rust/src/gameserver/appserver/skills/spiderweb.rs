//! Паутина паука `CSpiderWeb` (`0x199`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/spiderweb.cpp`. Достигнутый monster/pet-путь сохраняет
//! проверку прямого пути и `BLOCK_UNFLY`, запрет движения на задержке,
//! сохранённое время полёта снаряда, проверку уровней, `Cure` и wrapping-
//! длительность. `CGame` только предоставляет владельцев и доставку; стадии,
//! пакет `0xBFE01` и состояние принадлежат этому модулю. Координатная и
//! объектная player-перегрузки остаются RAW ниже.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderweb.cpp

// ============================================================================
// FUNCTION: CSpiderWeb::CSpiderWeb
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderweb.cpp:18
// RVA: 0x0013F8B0
// ADDRESS: 0053f8b0
// PROTOTYPE: undefined __thiscall CSpiderWeb(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderWeb::~CSpiderWeb
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderweb.cpp:28
// RVA: 0x0013F920
// ADDRESS: 0053f920
// PROTOTYPE: void __thiscall ~CSpiderWeb(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderWeb::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderweb.cpp:111
// RVA: 0x0013F940
// ADDRESS: 0053f940
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderWeb::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderweb.cpp:130
// RVA: 0x0013FA20
// ADDRESS: 0053fa20
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderWeb::Begin(CMoveShape *, CMoveShape *)
// STATUS: IMPLEMENTED
// Достигнутый monster/pet-вход создаёт `MonsterBaseAttackCast` и передаёт
// дальнейшие стадии владельцу `execute_owned_spider_web`.

// ============================================================================
// FUNCTION: CSpiderWebEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// Действия начала и выстрела материализованы; player-only failure-доставка
// остаётся RAW до появления соответствующего production caller-а.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spiderweb.cpp:351
// RVA: 0x0013FBF0
// ADDRESS: 0053fbf0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpiderWeb::CheckCastCondition
// STATUS: IMPLEMENTED
// Проверки cooldown, дальности и `BLOCK_UNFLY` принадлежат
// `execute_owned_spider_web`; движение блокируется до исходной задержки.

// ============================================================================
// FUNCTION: CSpiderWeb::AI
// STATUS: IMPLEMENTED
// Достигнутый monster/pet-путь материализован в `execute_owned_spider_web`.
// Не достигнутые перегрузки входа сохранены выше.

// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{SKILL_USAGE_DELAY_TIME, time_reached};
use super::basemagic::SKILL_USAGE_TARGET_MAX_DISTANCE;
use super::monsterattack::{owned_monster_attackable, resolve_owned_monster_attack_target};
use super::skillbaseproperties::CSkillBaseProperties;
use super::spiderwebstate::{SpiderWebState, send_spider_web_state_visual};
use crate::gameserver::appserver::ai::monsterai::approach_attack_range;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use crate::public::tools::get_line_direction;

const MONSTER_TYPE: i32 = 600;
const PLAYER_TYPE: i32 = 400;
const BLOCK_UNFLY: u8 = 2;
const CURE_SKILL_ID: u32 = 0x131;
const SKILL_USAGE_REUSE_SKILL_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_MISSILE_FLYING_TIME: u32 = 10_008;
const SKILL_USAGE_CONST: u32 = 20_010;
pub(crate) const SPIDER_WEB_SKILL_ID: u32 = 0x199;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SpiderWebProgress {
    missile_flying_time_ms: u32,
}

impl SpiderWebProgress {
    pub(crate) const fn new(missile_flying_time_ms: u32) -> Self {
        Self {
            missile_flying_time_ms,
        }
    }

    pub(crate) const fn missile_flying_time_ms(self) -> u32 {
        self.missile_flying_time_ms
    }
}

fn send_cast_start(
    game: &CGame,
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(SPIDER_WEB_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "поля являются точным payload исходного выстрела")]
fn send_cast_fire(
    game: &CGame,
    region: &CServerRegion,
    source: &crate::gameserver::appserver::shape::CShape,
    monster_id: i32,
    skill_level: u16,
    target: ShapeIdentity,
    target_x: i32,
    target_y: i32,
    missile_flying_time_ms: u32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(SPIDER_WEB_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(monster_id);
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(target_x);
    message.add_long(target_y);
    message.add_ulong(missile_flying_time_ms);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn target_level(game: &CGame, region: &CServerRegion, target: ShapeIdentity) -> Option<i32> {
    match target.object_type {
        PLAYER_TYPE => game.find_player(target.id).map(|player| i32::from(player.level())),
        MONSTER_TYPE => region.find_monster_by_id(target.id).and_then(|monster| {
            game.find_monster_property_by_origin_name(monster.base_property_key()?)
                .map(|property| property.level as i32)
        }),
        _ => None,
    }
}

fn target_has_cure(game: &CGame, region: &CServerRegion, target: ShapeIdentity) -> bool {
    match target.object_type {
        PLAYER_TYPE => game
            .find_player(target.id)
            .is_some_and(|player| player.has_state_by_skill_id(CURE_SKILL_ID)),
        MONSTER_TYPE => region
            .find_monster_by_id(target.id)
            .is_some_and(|monster| {
                monster
                    .move_shape()
                    .has_state_by_skill_id(CURE_SKILL_ID)
            }),
        _ => false,
    }
}

fn install_state(
    game: &mut CGame,
    region: &mut CServerRegion,
    target: ShapeIdentity,
    state: SpiderWebState,
    now_ms: u32,
) {
    let installed = match target.object_type {
        PLAYER_TYPE => game.find_player_mut(target.id).and_then(|player| {
            let previous = player.replace_spider_web_state(state);
            if previous.is_some() {
                player.set_skill_moveable(true);
                player.set_skill_fightable(true);
            }
            player.set_skill_moveable(false);
            player.set_skill_fightable(false);
            Some((
                previous,
                player.shape().identity(),
                player.shape().get_tile_x().ok()?,
                player.shape().get_tile_y().ok()?,
            ))
        }),
        MONSTER_TYPE => region.find_monster_by_id_mut(target.id).and_then(|monster| {
            let previous = monster.move_shape_mut().replace_spider_web_state(state);
            if previous.is_some() {
                monster.move_shape_mut().set_moveable(true);
                monster.move_shape_mut().set_fightable(true);
            }
            monster.move_shape_mut().set_moveable(false);
            monster.move_shape_mut().set_fightable(false);
            Some((
                previous,
                monster.move_shape().shape().identity(),
                monster.move_shape().shape().get_tile_x().ok()?,
                monster.move_shape().shape().get_tile_y().ok()?,
            ))
        }),
        _ => None,
    };
    let Some((previous, identity, tile_x, tile_y)) = installed else {
        return;
    };
    if let Some(previous) = previous {
        send_spider_web_state_visual(
            game, region.id, identity, tile_x, tile_y, previous, false, now_ms,
        );
    }
    send_spider_web_state_visual(
        game, region.id, identity, tile_x, tile_y, state, true, now_ms,
    );
}

fn cancel_cast(region: &mut CServerRegion, monster_id: i32) {
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().set_moveable(true);
        monster.cancel_base_attack_cast();
    }
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель и текущий такт исходного навыка")]
pub(crate) fn execute_owned_spider_web(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
) -> bool {
    let Some((source_shape, source_property, source_master, source_tamed, cast, last_used_ms)) =
        region.find_monster_by_id(monster_id).and_then(|monster| {
            Some((
                monster.move_shape().shape().clone(),
                game.find_monster_property_by_origin_name(monster.base_property_key()?)?.clone(),
                monster.master_info(),
                monster.is_tamed(),
                monster.base_attack_cast(),
                monster.skill_last_used_ms(SPIDER_WEB_SKILL_ID),
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
    if target.dead
        || target.god
        || target.city_dead
        || !owned_monster_attackable(
            game,
            region.id,
            &source_property,
            source_tamed,
            source_master,
            target_identity,
            &target,
        )
    {
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
            monster.clear_ai_target();
        }
        return true;
    }
    let (Ok(source_x), Ok(source_y), Ok(target_x), Ok(target_y)) = (
        source_shape.get_tile_x(),
        source_shape.get_tile_y(),
        target.shape.get_tile_x(),
        target.shape.get_tile_y(),
    ) else {
        return true;
    };
    let maximum_distance = properties.query_property(SKILL_USAGE_TARGET_MAX_DISTANCE);

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
        let reuse_delay = properties.query_property(SKILL_USAGE_REUSE_SKILL_DELAY_TIME);
        if last_used_ms != 0 && !time_reached(now_ms, last_used_ms, reuse_delay) {
            return true;
        }
        let direction = get_line_direction(source_x, source_y, target_x, target_y);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().shape_mut().set_direction(direction);
            monster.move_shape_mut().set_moveable(false);
            monster.begin_base_attack_cast(
                target_identity,
                SPIDER_WEB_SKILL_ID,
                skill_level,
                now_ms,
            );
        }
        let source = region
            .find_monster_by_id(monster_id)
            .map(|monster| monster.move_shape().shape())
            .unwrap_or(&source_shape);
        send_cast_start(game, region, source, monster_id, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение паутины проверено выше");
    if cast.dispatch().skill_id != SPIDER_WEB_SKILL_ID
        || cast.dispatch().target != target_identity
    {
        return false;
    }
    let delay_ms = properties.query_property(SKILL_USAGE_DELAY_TIME);
    if cast.stage() == SkillStage::Check {
        if !time_reached(now_ms, cast.started_at_ms(), delay_ms) {
            return true;
        }
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.move_shape_mut().set_moveable(true);
        }
        let Some(target_level) = target_level(game, region, target_identity) else {
            cancel_cast(region, monster_id);
            return true;
        };
        if (source_property.level as i32).wrapping_add(10) < target_level {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                let _ = monster.finish_base_attack_cast(now_ms);
            }
            return true;
        }
        let path = region.straight_skill_path(source_x, source_y, target_x, target_y, None);
        if (maximum_distance != 0 && path.len() > maximum_distance as usize)
            || path.iter().any(|cell| cell.2 == BLOCK_UNFLY)
        {
            cancel_cast(region, monster_id);
            return true;
        }
        let missile_flying_time_ms = properties
            .query_property(SKILL_USAGE_MISSILE_FLYING_TIME)
            .wrapping_mul(path.len() as u32);
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
            monster.set_spider_web_progress(SpiderWebProgress::new(missile_flying_time_ms));
        }
        send_cast_fire(
            game,
            region,
            &source_shape,
            monster_id,
            skill_level,
            target_identity,
            target_x,
            target_y,
            missile_flying_time_ms,
        );
        return true;
    }

    let Some(progress) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| monster.spider_web_progress())
    else {
        cancel_cast(region, monster_id);
        return true;
    };
    if !time_reached(
        now_ms,
        cast.started_at_ms(),
        delay_ms.wrapping_add(progress.missile_flying_time_ms()),
    ) {
        return true;
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
    }
    if !target_has_cure(game, region, target_identity) {
        let target_level = target_level(game, region, target_identity).unwrap_or(1);
        let duration_multiplier = (source_property.level as i32)
            .wrapping_sub(target_level)
            .wrapping_add(properties.query_property(SKILL_USAGE_CONST) as i32)
            .max(1);
        let keep_time_ms = properties
            .query_property(SKILL_USAGE_STATE_PERSIST_TIME)
            .wrapping_mul(duration_multiplier as u32);
        install_state(
            game,
            region,
            target_identity,
            SpiderWebState::new(now_ms, keep_time_ms),
            now_ms,
        );
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().shape_mut().set_action(1);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}
