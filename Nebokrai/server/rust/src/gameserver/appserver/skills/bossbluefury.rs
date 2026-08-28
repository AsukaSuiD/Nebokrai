//! Ярость синего босса `CBossBlueFury` (`0x1f7`) для достигнутого пути монстра.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/bossbluefury.cpp`. Путь монстра сохраняет проверку
//! перезарядки, действия 0/1 вокруг источника, задержку и замену собственного
//! `BossBlueFuryState`. Расход RP и клиентские ошибки относятся только к
//! неподключённой ветви игрока и остаются в исходном материале ниже.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefury.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefury.h

// ============================================================================
// FUNCTION: CBossBlueFury::CBossBlueFury
// STATUS: PARTIALLY_IMPLEMENTED
// Идентификатор достигнутого пути задаёт `BOSS_BLUE_FURY_SKILL_ID`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefury.cpp:18
// RVA: 0x0012E250
// ADDRESS: 0052e250
// PROTOTYPE: undefined __thiscall CBossBlueFury(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFury::~CBossBlueFury
// STATUS: PARTIALLY_IMPLEMENTED
// Завершение достигнутого пути выполняет `finish_base_attack_cast`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefury.cpp:26
// RVA: 0x0012E2C0
// ADDRESS: 0052e2c0
// PROTOTYPE: void __thiscall ~CBossBlueFury(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFury::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefury.cpp:123
// RVA: 0x0012E2E0
// ADDRESS: 0052e2e0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFury::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefury.cpp:139
// RVA: 0x0012E3B0
// ADDRESS: 0052e3b0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFury::Begin
// STATUS: PARTIALLY_IMPLEMENTED
// Объектный путь монстра выполняется `execute_owned_boss_blue_fury`;
// вариант игрока остаётся ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefury.cpp:106
// RVA: 0x0012E4A0
// ADDRESS: 0052e4a0
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, CMoveShape * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryEffect::UpdateVisualEffect
// STATUS: IMPLEMENTED
// Действия 0/1 достигнутого пути выполняют `send_cast_start` и `send_cast_fire`.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefury.cpp:294
// RVA: 0x0012E560
// ADDRESS: 0052e560
// PROTOTYPE: void __thiscall UpdateVisualEffect(CState * param_1, ulong param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFury::CheckCastCondition
// STATUS: PARTIALLY_IMPLEMENTED
// Проверка перезарядки монстра выполняется `execute_owned_boss_blue_fury`;
// расход RP и ошибки игрока остаются ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefury.cpp:38
// RVA: 0x0012E8E0
// ADDRESS: 0052e8e0
// PROTOTYPE: int __thiscall CheckCastCondition(CMoveShape * param_1)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFury::AI
// STATUS: PARTIALLY_IMPLEMENTED
// Стадии монстра, задержка и замена состояния выполняются
// `execute_owned_boss_blue_fury`; ветвь игрока остаётся ниже.
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefury.cpp:169
// RVA: 0x0012EAC0
// ADDRESS: 0052eac0
// PROTOTYPE: void __thiscall AI(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//













// COMPONENT_VARIANT_END: GameServer

use super::baseattack::{SKILL_USAGE_DELAY_TIME, SKILL_USAGE_REUSE_DELAY_TIME, time_reached};
use super::bossbluefurystate::{
    BossBlueFuryState, send_boss_blue_fury_state_visual,
};
use super::skillbaseproperties::CSkillBaseProperties;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::skills::kernel::SkillStage;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;
use crate::public::guid::CGuid;

const MONSTER_TYPE: i32 = 600;
const SKILL_USAGE_STATE_PERSIST_TIME: u32 = 10_002;
const SKILL_USAGE_STATE_PERSIST_TIME_MODIFIER: u32 = 10_003;
const SKILL_USAGE_TARGET_DAMAGE_FACTOR: u32 = 20_003;
pub(crate) const BOSS_BLUE_FURY_SKILL_ID: u32 = 0x1f7;

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
    message.add_long(BOSS_BLUE_FURY_SKILL_ID as i32);
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
    message.add_long(BOSS_BLUE_FURY_SKILL_ID as i32);
    message.add_short(skill_level as i16);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(MONSTER_TYPE);
    message.add_long(source.identity().id);
    message.add_long(tile_x);
    message.add_long(tile_y);
    let _ = game.send_game_shape_around(region, source, None, &message);
}

pub(crate) fn execute_owned_boss_blue_fury(
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
                BOSS_BLUE_FURY_SKILL_ID,
                skill_level,
                now_ms,
            );
        }
        send_cast_start(game, region, &source, skill_level);
        return true;
    }

    let cast = cast.expect("выполнение ярости синего босса проверено выше");
    if cast.dispatch().skill_id != BOSS_BLUE_FURY_SKILL_ID {
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
    let state = BossBlueFuryState::new(
        now_ms,
        properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME),
        properties.query_property(SKILL_USAGE_TARGET_DAMAGE_FACTOR) as i32,
        properties.query_property(SKILL_USAGE_STATE_PERSIST_TIME_MODIFIER),
    );
    let previous = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let previous = monster.move_shape_mut().take_boss_blue_fury_state();
        if previous.is_some_and(BossBlueFuryState::control_locked) {
            monster.move_shape_mut().set_moveable(true);
            monster.move_shape_mut().set_fightable(true);
        }
        previous
    });
    if let Some(previous) = previous {
        send_boss_blue_fury_state_visual(
            game, region.id, identity, tile_x, tile_y, previous, false, now_ms,
        );
    }
    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        monster.move_shape_mut().set_moveable(false);
        monster.move_shape_mut().set_fightable(false);
        monster.move_shape_mut().begin_boss_blue_fury_state(state);
    }
    send_boss_blue_fury_state_visual(
        game, region.id, identity, tile_x, tile_y, state, true, now_ms,
    );

    if let Some(monster) = region.find_monster_by_id_mut(monster_id) {
        let _ = monster.advance_base_attack_cast(SkillStage::Attack, SkillStage::Apply);
        let _ = monster.finish_base_attack_cast(now_ms);
    }
    true
}
