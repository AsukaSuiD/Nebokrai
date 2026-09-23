//! Постоянная сила стихии COriginState.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/originstate.cpp/.h.
//! Данные, формат записи и формула игрока находятся в `zone/effects/element.rs`;
//! здесь остаются доступ к живой фигуре и Begin/End.
//! Primary Begin(U,S) в immediatestateinstallation читает базовые часы.
//! DB-restart Begin(NULL,S) сохраняет U/timestamp, обновляет S и снимает ended.
//! End отмечает ended, затем удаляет себя через свежий GetUser; NULL U
//! не заменяется держателем. SetRegion меняет только регион U.
//! Property читает свежую S без ended-gate: игрок знаково расширяет i16 gain
//! и складывает с element_modify с переполнением; монстр прибавляет полный
//! i32 к модификатору. Итоговый monster getter сохраняет свои clamp/factor.
//! Default задаёт нулевой gain; DB8 — little-endian ID + i32 gain.
//! SlotMap хранит базу отдельно от payload. Собственных часов AI и visual нет.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::resolve_applied_state_sufferer;
use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, end_base_applied_state, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::CGame;
pub(crate) use nebokrai_zone::effects::{ELEMENT_STATE_BYTES as ORIGIN_STATE_BYTES, OriginState};

pub(crate) fn update_origin_state_properties(
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
        .and_then(|shape| shape.applied_state::<OriginState>(key))
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
            modifiers.element_modify = modifiers.element_modify.wrapping_add(state.monster_gain());
        }
    } else if target.object_type == 400 {
        if let Some(player) = game.find_player_mut(target.id) {
            player.update_state_combat_properties(|mut properties| {
                properties.element_modify = state.apply_player_modify(properties.element_modify);
                properties
            });
        }
    }
    true
}

pub(crate) fn restart_origin_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<OriginState>(key))
        .is_none()
    {
        return false;
    }
    begin_base_applied_state(game, region_id, holder, key)
}

pub(crate) fn end_origin_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<OriginState>(key))
        .is_none()
    {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, ORIGIN_STATE_BYTES)
}
