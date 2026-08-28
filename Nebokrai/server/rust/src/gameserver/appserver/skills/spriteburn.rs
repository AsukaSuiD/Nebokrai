//! Огненная область `CSpriteBurn` (`0x1a6`) для достигнутого пути монстра.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/spriteburn.cpp`. После задержки навык обходит подтверждённую
//! маску `7×7` в порядке X → Y и живой порядок объектов каждой клетки, исключает
//! мёртвые и недоступные цели с `CureState`, затем заменяет на них канонический
//! `SpiderPoisonState` (`0x191`). Состояние создаётся с отдельным чтением часов
//! для каждой цели. Варианты игрока и координатные перегрузки остаются RAW ниже.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburn.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburn.h

// ============================================================================
// FUNCTION: CSpriteBurn::CSpriteBurn
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburn.cpp:27
// RVA: 0x00132850
// ADDRESS: 00532850
// PROTOTYPE: undefined __thiscall CSpriteBurn(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpriteBurn::~CSpriteBurn
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburn.cpp:35
// RVA: 0x001328C0
// ADDRESS: 005328c0
// PROTOTYPE: void __thiscall ~CSpriteBurn(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpriteBurn::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburn.cpp:112
// RVA: 0x001328E0
// ADDRESS: 005328e0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpriteBurn::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburn.cpp:128
// RVA: 0x001329B0
// ADDRESS: 005329b0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpriteBurn::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// Повторное применение для монстра проверяет `execute_owned_sprite_burn`;
// расход MP и клиентские ошибки игрока остаются в теле ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburn.cpp:42
// RVA: 0x00132AA0
// ADDRESS: 00532aa0
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpriteBurn::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Объектный вход монстра достигается через `execute_owned_sprite_burn`;
// объектный вход игрока остаётся в теле ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburn.cpp:95
// RVA: 0x00132B70
// ADDRESS: 00532b70
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpriteBurnEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// Действия начала и срабатывания монстра формирует `execute_owned_sprite_burn`;
// клиентские ответы об ошибках игрока остаются в теле ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburn.cpp:338
// RVA: 0x00132C30
// ADDRESS: 00532c30
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpriteBurn::AddState
// STATUS: PARTIALLY_IMPLEMENTED
// Достигнутый путь монстра использует общий канонический `SpiderPoisonState`;
// снимок разрешений владельца-игрока остаётся в теле ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburn.cpp:270
// RVA: 0x00133080
// ADDRESS: 00533080
// PROTOTYPE: void __thiscall AddState(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CSpriteBurn::AI
// STATUS: PARTIALLY_IMPLEMENTED
// Задержка, маска области, упорядоченный выбор целей и завершение monster-cast
// материализованы `execute_owned_sprite_burn`; расход MP игрока остаётся ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\spriteburn.cpp:157
// RVA: 0x001332C0
// ADDRESS: 005332c0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//














// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, time_reached};
use super::monsterattack::{
    monster_attack_cell_candidates, owned_monster_attackable,
    resolve_owned_monster_attack_target,
};
use super::monsterrangeattack::range_attack_scope_cells;
use super::skillbaseproperties::CSkillBaseProperties;
use super::spiderpoison::{install_spider_poison_state, target_has_cure};
use super::spiderpoisonstate::SpiderPoisonState;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_AFFECT_FREQUENCY: u32 = 6_001;
const SKILL_USAGE_CONST: u32 = 20_010;

pub(crate) const SPRITE_BURN_SKILL_ID: u32 = 0x1a6;

fn send_start(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(SPRITE_BURN_SKILL_ID as i32);
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
    message.add_long(SPRITE_BURN_SKILL_ID as i32);
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
pub(crate) fn execute_owned_sprite_burn<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    target_identity: ShapeIdentity,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
    runtime: &mut Runtime,
) -> bool {
    let Some((source, property, master, tamed, cast, last_used_ms)) = region
        .find_monster_by_id(monster_id)
        .and_then(|monster| {
            Some((
                monster.move_shape().shape().clone(),
                game.find_monster_property_by_origin_name(monster.base_property_key()?)?
                    .clone(),
                monster.master_info(),
                monster.is_tamed(),
                monster.base_attack_cast(),
                monster.last_base_attack_ms(),
            ))
        })
    else {
        return false;
    };

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
        if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
            monster.begin_base_attack_cast(
                target_identity,
                SPRITE_BURN_SKILL_ID,
                skill_level,
                now_ms,
            );
        }
        send_start(game, region, &source, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение огненной области проверено выше");
    if cast.dispatch().skill_id != SPRITE_BURN_SKILL_ID {
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
    for (offset_x, offset_y) in range_attack_scope_cells() {
        let candidates = monster_attack_cell_candidates(
            game,
            region,
            monster_id,
            center_x.wrapping_add(offset_x),
            center_y.wrapping_add(offset_y),
        );
        for identity in candidates {
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
                || target_has_cure(game, region, identity)
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
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}
