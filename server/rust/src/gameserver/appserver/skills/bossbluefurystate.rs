//! Живые restart/AI/End и пересчёт монстра ярости синего босса
//! `CBossBlueFuryState` (`0x1f7`) в переходном Game.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/bossbluefurystate.cpp`. Данные, срок и 12-байтная запись
//! перенесены в Zone `effects/bossbluefury.rs` (там же адреса конструктора,
//! vtable и обеих сторон кодека). Достигнутые пути игрока и монстра заменяют
//! первый найденный экземпляр до установки нового, сохраняя остальные
//! загруженные записи. Движение и бой запрещены на слабой фазе, после неё
//! состояние сохраняется до общего срока. Только для монстра модификаторы
//! атаки увеличиваются указанной долей текущих getter-ов. Визуальные начало
//! и завершение сохраняют `0xBFE03/04`. `weak_time` после загрузки остаётся
//! нулевым. AI получает один ключ общей арены; общий CMoveShape задаёт
//! порядок прохода. После слабой границы запреты снимаются на каждом AI,
//! без one-shot флага. End отправляет эффект, удаляет тот же экземпляр и
//! отдельно снимает оба запрета; окончание срока не подменяет проверку
//! слабой границы. Restart воспроизводит только Begin(NULL, holder): базовый
//! Begin сохраняет timestamp/user; готовая запись и её ключ не заменяются.
//! Begin создаёт принадлежащий записи loop=1 visual без немедленного пакета.
//! Оба запрета добавляются до выделения visual, включая loaded weak_time=0.
//! OnUpdateProperties: GetSufferer → существующий visual Update(0) →
//! type600/RTTI CMonster → maximum, затем minimum. Каждый процент вычисляется
//! от соответствующего живого getter; +0x1AC/+0x1A8 прибавляют signed delta
//! к накопленным модификаторам. Ограничения и pet-множитель остаются у getter.

use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, begin_applied_state_visual,
};
use crate::gameserver::appserver::states::state::{
    resolve_applied_state_sufferer, update_property_state_visual, StatePropertyTarget,
};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) use nebokrai_zone::effects::{
    BOSS_BLUE_FURY_STATE_BYTES, BOSS_BLUE_FURY_STATE_ID, BossBlueFuryState,
};

pub(crate) fn update_boss_blue_fury_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((target_region, target)) = resolve_applied_state_sufferer(game, region_id, holder, key)
    else { return false; };
    let _ = update_property_state_visual::<BossBlueFuryState>(
        game, region_id, holder, key, StatePropertyTarget::Sufferer, now,
        |state, now| state.client_state_time(now),
    );
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BossBlueFuryState>(key)).copied()
    else { return false; };
    if target.object_type != 600 {
        return true;
    }
    let base = game.find_region(target_region)
        .and_then(|region| region.base().find_monster_by_id(target.id))
        .and_then(|monster| {
            let property = game.find_monster_property_by_origin_name(monster.original_name())?;
            Some((property.minimum_attack, property.maximum_attack))
        });
    let Some((minimum_base, maximum_base)) = base else { return true; };
    let Some(monster) = game.find_region_mut(target_region)
        .and_then(|region| region.base_mut().find_monster_by_id_mut(target.id))
    else { return true; };
    let maximum = monster.state_attack_bounds(minimum_base, maximum_base).1;
    let maximum_gain = state.attack_modifier(maximum);
    let modifiers = monster.move_shape_mut().property_modifiers_mut();
    modifiers.maximum_attack = modifiers.maximum_attack.wrapping_add(maximum_gain);
    let minimum = monster.state_attack_bounds(minimum_base, maximum_base).0;
    let minimum_gain = state.attack_modifier(minimum);
    let modifiers = monster.move_shape_mut().property_modifiers_mut();
    modifiers.minimum_attack = modifiers.minimum_attack.wrapping_add(minimum_gain);
    true
}

pub(crate) fn restart_boss_blue_fury_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BossBlueFuryState>(key)).is_none() {
        return false;
    }
    if !begin_base_applied_state(game, region_id, holder, key) {
        return false;
    }
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_moveable(false);
        shape.set_fightable(false);
    }
    let _ = begin_applied_state_visual(game, region_id, holder, key, 1);
    true
}

pub(crate) fn update_boss_blue_fury_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    mut now_milliseconds: impl FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BossBlueFuryState>(key)).copied()
        else { return false };
    if state.weak_elapsed(now_milliseconds()) {
        if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
            shape.set_moveable(true);
            shape.set_fightable(true);
        }
    }
    if state.expired(now_milliseconds()) {
        let _ = end_boss_blue_fury_state(game, region_id, holder, key);
    }
    true
}

pub(crate) fn end_boss_blue_fury_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<BossBlueFuryState>(key)).copied()
        else { return false };
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state.skill_id() as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.remove_applied_state_record::<BossBlueFuryState>(
            key, BOSS_BLUE_FURY_STATE_BYTES,
        )).is_some();
    if removed {
        let _ = game.update_move_shape_properties(region_id, holder);
    }
    if let Some(shape) = resolve_state_move_shape_mut(game, region_id, holder) {
        shape.set_moveable(true);
        shape.set_fightable(true);
    }
    removed
}

pub(crate) fn end_player_boss_blue_fury_state_key(
    game: &mut CGame,
    player_id: i32,
    key: StateKey,
    _now_ms: u32,
) -> bool {
    let Some((region_id, holder)) = game.find_player(player_id)
        .and_then(|player| Some((player.server_region_id()?, player.shape().identity())))
        else { return false };
    end_boss_blue_fury_state(game, region_id, holder, key)
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp

// ============================================================================
// FUNCTION: CBossBlueFuryState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:89
// RVA: 0x001E8B60
// ADDRESS: 005e8b60
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CBossBlueFuryState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\bossbluefurystate.cpp:106
// RVA: 0x001E8C30
// ADDRESS: 005e8c30
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
