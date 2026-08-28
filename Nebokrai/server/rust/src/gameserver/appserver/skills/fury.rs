//! Ярость `CFury` (`0x1a3`) для достигнутого пути монстра.
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает задержку и
//! повторное использование, пакеты `0xBFE01`, накопление `CFuryState`, порядок
//! снятия конфликтующих состояний и последующее краткоживущее `CCureState`.
//! Модуль владеет всей семантикой навыка; общий диспетчер только предоставляет
//! владельца региона. Ветвь игрока и ещё не типизированный `CRageBreakState`
//! сохранены ниже как частично достигнутый исходный материал.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fury.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fury.h

// ============================================================================
// FUNCTION: CFury::CFury
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fury.cpp:19
// RVA: 0x00136100
// ADDRESS: 00536100
// PROTOTYPE: undefined __thiscall CFury(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFury::~CFury
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fury.cpp:27
// RVA: 0x00136170
// ADDRESS: 00536170
// PROTOTYPE: void __thiscall ~CFury(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFury::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fury.cpp:129
// RVA: 0x00136190
// ADDRESS: 00536190
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFury::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fury.cpp:145
// RVA: 0x00136260
// ADDRESS: 00536260
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFury::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fury.cpp:112
// RVA: 0x00136350
// ADDRESS: 00536350
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFuryEffect::UpdateVisualEffect
// STATUS: PARTIALLY_IMPLEMENTED
// Действия 0/1 достигнуты функциями `send_cast_start` и `send_cast_fire`;
// ответы об ошибках только для игрока остаются исходным материалом.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fury.cpp:332
// RVA: 0x00136410
// ADDRESS: 00536410
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFury::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// Повторное использование для монстра достигнуто в `execute_owned_fury`;
// расход RP и сообщения игроку пока не подключены.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fury.cpp:39
// RVA: 0x00136790
// ADDRESS: 00536790
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CFury::AI
// STATUS: PARTIALLY_IMPLEMENTED
// Полная достигнутая ветвь монстра находится в `execute_owned_fury`.
// Ветвь игрока и снятие ещё не типизированного `CRageBreakState` сохранены.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\fury.cpp:175
// RVA: 0x00136970
// ADDRESS: 00536970
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//










// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, time_reached};
use super::curestate::{CureState, send_cure_state_visual_at};
use super::furystate::{FuryState, send_fury_state_visual};
use super::skillbaseproperties::CSkillBaseProperties;
use super::spiderpoisonstate::send_spider_poison_state_visual;
use super::spiderwebstate::send_spider_web_state_visual;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_TARGET_ATK_GAIN: u32 = 105;
pub(crate) const FURY_SKILL_ID: u32 = 0x1a3;

fn self_identity(monster_id: i32) -> ShapeIdentity {
    ShapeIdentity {
        object_type: MONSTER_TYPE,
        id: monster_id,
        ex_id: CGuid::GUID_INVALID,
    }
}

fn send_cast_start(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
) {
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(1);
    message.add_long(FURY_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(source.get_direction());
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn send_cast_fire(
    game: &CGame,
    region: &CServerRegion,
    source: &CShape,
    skill_level: u16,
) {
    let Ok(tile_x) = source.get_tile_x() else {
        return;
    };
    let Ok(tile_y) = source.get_tile_y() else {
        return;
    };
    let mut message = CMessage::new(0x000b_fe01);
    message.add_byte(2);
    message.add_long(FURY_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(tile_x);
    message.add_long(tile_y);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

fn remove_reached_conflict_states(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    now_ms: u32,
) {
    let removed_poison = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let state = monster.move_shape_mut().take_spider_poison_state()?;
        Some((
            state,
            monster.move_shape().shape().identity(),
            monster.move_shape().shape().get_tile_x().unwrap_or_default(),
            monster.move_shape().shape().get_tile_y().unwrap_or_default(),
        ))
    });
    if let Some((state, identity, tile_x, tile_y)) = removed_poison {
        send_spider_poison_state_visual(
            game, region.id, identity, tile_x, tile_y, state, false, now_ms,
        );
    }

    let removed_web = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let state = monster.move_shape_mut().take_spider_web_state()?;
        monster.move_shape_mut().set_moveable(true);
        monster.move_shape_mut().set_fightable(true);
        Some((
            state,
            monster.move_shape().shape().identity(),
            monster.move_shape().shape().get_tile_x().unwrap_or_default(),
            monster.move_shape().shape().get_tile_y().unwrap_or_default(),
        ))
    });
    if let Some((state, identity, tile_x, tile_y)) = removed_web {
        send_spider_web_state_visual(
            game, region.id, identity, tile_x, tile_y, state, false, now_ms,
        );
    }
}

pub(crate) fn execute_owned_fury(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    skill_level: u16,
    properties: &CSkillBaseProperties,
    now_ms: u32,
) -> bool {
    let Some((source, cast, last_used_ms)) = region
        .find_monster_by_id(monster_id)
        .map(|monster| {
            (
                monster.move_shape().shape().clone(),
                monster.base_attack_cast(),
                monster.last_base_attack_ms(),
            )
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
                self_identity(monster_id),
                FURY_SKILL_ID,
                skill_level,
                now_ms,
            );
        }
        send_cast_start(game, region, &source, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение ярости проверено выше");
    if cast.dispatch().skill_id != FURY_SKILL_ID {
        return false;
    }
    if !time_reached(
        now_ms,
        cast.started_at_ms(),
        properties.query_property(SKILL_USAGE_DELAY_TIME),
    ) {
        return true;
    }

    send_cast_fire(game, region, &source, skill_level);
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Check, SkillStage::Calculate);
        let _ = monster.advance_base_attack_cast(SkillStage::Calculate, SkillStage::Attack);
    }

    let identity = source.identity();
    let tile_x = source.get_tile_x().unwrap_or_default();
    let tile_y = source.get_tile_y().unwrap_or_default();
    let keep_time_ms = properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME);
    let fury = FuryState::new(
        now_ms,
        keep_time_ms,
        properties.query_property(SKILL_USAGE_TARGET_ATK_GAIN) as i32,
    );
    send_fury_state_visual(
        game, region.id, identity, tile_x, tile_y, fury, true, now_ms,
    );
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().push_fury_state(fury);
    }

    remove_reached_conflict_states(game, region, monster_id, now_ms);

    let cure = CureState::new(keep_time_ms);
    let previous_cure = region
        .find_monster_by_id_mut(monster_id)
        .and_then(|monster| monster.move_shape_mut().replace_cure_state(cure));
    if let Some(previous) = previous_cure {
        send_cure_state_visual_at(
            game, region.id, identity, tile_x, tile_y, previous, false,
        );
    }
    send_cure_state_visual_at(game, region.id, identity, tile_x, tile_y, cure, true);

    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}
