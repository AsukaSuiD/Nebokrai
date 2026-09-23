//! Постоянное сопротивление стихиям CTaiJiState.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/taijistate.cpp/.h.
//! Данные, формат записи и формула игрока находятся в `zone/effects/element.rs`;
//! здесь остаются доступ к живой фигуре и Begin/End.
//! Primary Begin(U,S) в immediatestateinstallation читает базовые часы.
//! Begin(NULL,S), включая DB-restart, отказывает до изменения базы/ended.
//! End отмечает ended, затем удаляет себя через свежий GetUser; NULL U
//! не заменяется держателем. SetRegion меняет только регион U.
//! Property читает свежую S без ended-gate: игрок складывает u16 gain с
//! сопротивлением как u32 и ограничивает i32::MAX, монстр прибавляет полный
//! i32 к модификатору. Итоговый monster getter сохраняет свои clamp/factor.
//! Default задаёт нулевой gain; DB8 — little-endian ID + i32 gain.
//! SlotMap хранит базу отдельно от payload. Собственных часов AI и visual нет.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::resolve_applied_state_sufferer;
use crate::gameserver::appserver::states::state::{
    end_base_applied_state, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::CGame;
pub(crate) use nebokrai_zone::effects::{ELEMENT_STATE_BYTES as TAIJI_STATE_BYTES, TaiJiState};

pub(crate) fn update_tai_ji_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some((target_region, target)) =
        resolve_applied_state_sufferer(game, region_id, holder, key)
    else {
        return false;
    };
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<TaiJiState>(key))
        .copied()
    else {
        return false;
    };
    if target.object_type == 600 {
        if let Some(monster) = game
            .find_region_mut(target_region)
            .and_then(|region| region.base_mut().find_monster_by_id_mut(target.id))
        {
            let modifiers = monster.move_shape_mut().property_modifiers_mut();
            modifiers.element_resistance = modifiers
                .element_resistance
                .wrapping_add(state.monster_gain());
        }
    } else if target.object_type == 400 {
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|mut properties| {
                properties.element_resistance =
                    state.apply_player_resistance(properties.element_resistance);
                properties
            });
        }
    }
    true
}

pub(crate) fn restart_tai_ji_state(
    _game: &mut CGame,
    _region_id: i32,
    _holder: ShapeIdentity,
    _key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    false
}

pub(crate) fn end_tai_ji_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<TaiJiState>(key))
        .is_none()
    {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, TAIJI_STATE_BYTES)
}
