//! Владелец навыка `CCorpsePtomaine` (`0x19F`). Достигнутый путь монстра
//! сохраняет повторное применение, задержку, пакеты и обход полного квадрата
//! 3×3 в исходном порядке X → Y. Сам яд остаётся каноническим
//! `SpiderPoisonState`; расход MP и варианты игрока ниже остаются RAW.
//! Декомпилятор: Ghidra 12.1.2
//! Полный декомпилят хранится локально и не входит в распространяемый код.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\corpseptomaine.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\corpseptomaine.h

// ============================================================================
// FUNCTION: CCorpsePtomaine::CCorpsePtomaine
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\corpseptomaine.cpp:27
// RVA: 0x001397B0
// ADDRESS: 005397b0
// PROTOTYPE: undefined __thiscall CCorpsePtomaine(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCorpsePtomaine::~CCorpsePtomaine
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\corpseptomaine.cpp:35
// RVA: 0x00139820
// ADDRESS: 00539820
// PROTOTYPE: void __thiscall ~CCorpsePtomaine(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCorpsePtomaine::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\corpseptomaine.cpp:112
// RVA: 0x00139840
// ADDRESS: 00539840
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCorpsePtomaine::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\corpseptomaine.cpp:128
// RVA: 0x00139910
// ADDRESS: 00539910
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCorpsePtomaine::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован ниже;
// отличающиеся ветви игрока и недостигнутого вызова сохранены в RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\corpseptomaine.cpp:42
// RVA: 0x00139A00
// ADDRESS: 00539a00
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCorpsePtomaine::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован ниже;
// отличающиеся ветви игрока и недостигнутого вызова сохранены в RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\corpseptomaine.cpp:95
// RVA: 0x00139AE0
// ADDRESS: 00539ae0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCorpsePtomaineEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован ниже;
// отличающиеся ветви игрока и недостигнутого вызова сохранены в RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\corpseptomaine.cpp:339
// RVA: 0x00139BA0
// ADDRESS: 00539ba0
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCorpsePtomaine::AddState
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован ниже;
// отличающиеся ветви игрока и недостигнутого вызова сохранены в RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\corpseptomaine.cpp:271
// RVA: 0x00139FF0
// ADDRESS: 00539ff0
// PROTOTYPE: void __thiscall AddState(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CCorpsePtomaine::AI
// STATUS: PARTIALLY_IMPLEMENTED
// IMPLEMENTED: достигнутый путь владельца-монстра материализован ниже;
// отличающиеся ветви игрока и недостигнутого вызова сохранены в RAW.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\corpseptomaine.cpp:157
// RVA: 0x0013A230
// ADDRESS: 0053a230
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

















// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{SKILL_USAGE_DELAY_TIME, time_reached};
use super::monsterattack::{
    monster_attack_cell_candidates, owned_monster_attackable,
    resolve_owned_monster_attack_target,
};
use super::skillbaseproperties::CSkillBaseProperties;
use super::spiderpoison::{install_spider_poison_state, target_has_cure};
use super::spiderpoisonstate::SpiderPoisonState;
use crate::gameserver::appserver::ai::monsterai::{
    approach_attack_range, schedule_attack_interval,
};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::CShape;
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_REUSE_DELAY_TIME: u32 = 10_005;
const SKILL_USAGE_TARGET_MAX_DISTANCE: u32 = 5_003;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_CONST: u32 = 20_010;
pub(crate) const CORPSE_PTOMAINE_SKILL_ID: u32 = 0x19f;

fn send_start(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(CORPSE_PTOMAINE_SKILL_ID as i32);
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
    tile_x: i32,
    tile_y: i32,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(CORPSE_PTOMAINE_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(0);
    message.add_long(0);
    message.add_long(tile_x);
    message.add_long(tile_y);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "граница сохраняет владельца, цель выбора ИИ и текущий такт")]
pub(crate) fn execute_owned_corpse_ptomaine<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: crate::gameserver::appserver::shape::ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    let Some((source, property, master, tamed, attack_interval_ms, cast, last_used_ms)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            let property = game
                .find_monster_property_by_origin_name(monster.base_property_key()?)?
                .clone();
            let attack_interval_ms = monster
                .is_tamed()
                .then(|| monster.pet_attack_properties(&property))
                .map_or(property.attack_speed, |pet| pet.attack_interval);
            Some((
                monster.move_shape().shape().clone(),
                property,
                monster.master_info(),
                monster.is_tamed(),
                attack_interval_ms,
                monster.base_attack_cast(),
                monster.skill_last_used_ms(CORPSE_PTOMAINE_SKILL_ID),
            ))
        })
    else {
        return false;
    };

    if cast.is_none() {
        let Some(target) = resolve_owned_monster_attack_target(game, region, target_identity)
        else {
            if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
                monster.clear_ai_target();
            }
            return true;
        };
        let (Ok(target_x), Ok(target_y)) =
            (target.shape.get_tile_x(), target.shape.get_tile_y())
        else {
            return true;
        };
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
        if let Some(attack_interval_ms) = schedule_attack_interval(property.ai, attack_interval_ms)
        {
            let attack_started = region
                .find_monster_by_id_mut(monster_id)
                .is_some_and(|monster| {
                    monster.begin_ai_attack_attempt(now_ms, attack_interval_ms)
                });
            if !attack_started {
                return true;
            }
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
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.begin_base_attack_cast(
                target_identity,
                CORPSE_PTOMAINE_SKILL_ID,
                skill_level,
                now_ms,
            );
        }
        send_start(game, region, &source, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение трупного яда проверено выше");
    if cast.dispatch().skill_id != CORPSE_PTOMAINE_SKILL_ID {
        return false;
    }
    if !time_reached(
        now_ms,
        cast.started_at_ms(),
        properties.query_property(SKILL_USAGE_DELAY_TIME),
    ) {
        return true;
    }
    let (Ok(center_x), Ok(center_y)) = (source.get_tile_x(), source.get_tile_y()) else {
        return true;
    };
    send_fire(game, region, &source, skill_level, center_x, center_y);
    let state_master = MasterInfo {
        master_type: MONSTER_TYPE,
        master_id: monster_id,
        ..MasterInfo::default()
    };
    for offset_x in -1..=1 {
        for offset_y in -1..=1 {
            let candidates = monster_attack_cell_candidates(
                game,
                region,
                monster_id,
                center_x.wrapping_add(offset_x),
                center_y.wrapping_add(offset_y),
            );
            for identity in candidates {
                let Some(target) = resolve_owned_monster_attack_target(game, region, identity)
                else {
                    continue;
                };
                if target.dead
                    || target.god
                    || target.city_dead
                    || target_has_cure(game, region, identity)
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
                let state_now_ms = runtime.now_milliseconds();
                install_spider_poison_state(
                    game,
                    region,
                    identity,
                    SpiderPoisonState::new(
                        state_master,
                        state_now_ms,
                        properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
                        properties.query_property(SKILL_USAGE_TARGET_AFFECT_FREQUENCY),
                        properties.query_property(SKILL_USAGE_CONST),
                    ),
                    state_now_ms,
                );
            }
        }
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}
