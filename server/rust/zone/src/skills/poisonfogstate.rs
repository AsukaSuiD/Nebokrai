//! Наложение, обновление и снятие ядовитого тумана CPoisonFogState (0xC9)
//! у живой фигуры. Источник: `GameServer/gameserver.exe` +
//! `GameServer/GameServer.pdb`, `appserver/skills/poisonfogstate.cpp` и
//! `poisonfogstate.h`. Данные, запись и расчёт потерь находятся в
//! `effects/poisonfog.rs`; здесь — участники, visual, поиск фигуры и
//! запись рассчитанных свойств.
//! Для координатного Begin (0x00607E30) и typed Begin (0x00607EC0)
//! вызывающие цепочки не установлены (UNKNOWN); основной путь использует
//! объектный Begin.

use nebokrai_shared::values::CGuid;

use crate::effects::{PoisonFogState, POISON_FOG_STATE_BYTES};
use crate::regions::ShapeIdentity;

use super::state::StateKey;
use super::zonalcast::{ZonalCastGame, ZonalCastMoveShape, ZonalCastPlayer, ZonalCastPropertyTarget};

/// Объектный Begin первичного экземпляра тумана; часы начинаются только
/// при живом U, участники перечитываются живыми (ex_id сброшен).
pub fn begin_primary_poison_fog_state<Game: ZonalCastGame>(
    game: &mut Game,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    mut state: PoisonFogState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    let sufferer = sufferer?;
    game.resolve_state_move_shape(holder_region, holder)?;
    game.resolve_state_move_shape(sufferer.0, sufferer.1)?;
    if user.is_some() { state.begin_at(now()); }
    let participant = |(region, identity)| {
        let shape = game.resolve_state_move_shape(region, identity)?.shape();
        Some((shape.get_region_id(), ShapeIdentity { ex_id: CGuid::GUID_INVALID, ..shape.identity() }))
    };
    let user = match user { Some(user) => Some(participant(user)?), None => None };
    let sufferer = participant(sufferer)?;
    let record = state.encoded_for_install();
    let shape = game.resolve_state_move_shape_mut(holder_region, holder)?;
    let key = shape.append_applied_state_record(state, &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, Some(sufferer));
    Some(key)
}

/// Property callback: уровень цели по её типу (player — живой уровень,
/// монстр — уровень исходной таблицы свойств, постройкам/прочим 500/1100/
/// 1200 — 1), visual, затем пересчёт и запись тех же свойств.
pub fn update_poison_fog_state_properties<Game: ZonalCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key)).is_none()
    {
        return false;
    }
    let Some((target_region, target)) = game.resolve_applied_state_sufferer(
        region_id, holder, key,
    ) else { return false };
    let target_level = match target.object_type {
        400 => {
            let Some(player) = game.find_player(target.id) else { return false };
            player.level()
        }
        600 => {
            let Some(level) = game.monster_property_level(target_region, target.id)
            else { return false };
            level
        }
        500 | 1100 | 1200 => 1,
        _ => return false,
    };
    let _ = game.update_property_state_visual::<PoisonFogState>(
        region_id, holder, key, ZonalCastPropertyTarget::Sufferer, now,
        |state, now| state.client_time(now) as u32,
    );
    let Some(state) = game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key))
    else { return false };
    match target.object_type {
        400 => {
            let Some(properties) = game.find_player(target.id).map(|player| player.combat_properties())
            else { return false };
            let (defense, element_resistance) = state.player_properties(
                target_level, properties.defense, properties.element_resistance,
            );
            let mut updated = properties;
            updated.defense = defense;
            updated.element_resistance = element_resistance;
            let Some(player) = game.find_player_mut(target.id) else { return false };
            player.update_state_combat_properties(|_| updated);
        }
        600 => {
            let (defense, resistance) = state.monster_losses(target_level);
            if !game.subtract_poison_fog_monster_losses(target_region, target, defense, resistance)
            { return false }
        }
        _ => {}
    }
    true
}

/// Restart прежнего экземпляра: повторный base Begin и visual loop1.
pub fn restart_poison_fog_state<Game: ZonalCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key)).is_none()
    {
        return false;
    }
    if !game.begin_base_applied_state(
        region_id, holder, key,
    ) { return false }
    let _ = game.begin_applied_state_visual(
        region_id, holder, key, 1,
    );
    true
}

/// AI прежнего состояния: завершение только по строгому сроку.
pub fn update_poison_fog_state<Game: ZonalCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    let expired = game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key))
        .is_some_and(|state| state.expired(now_ms));
    if !expired { return false }
    end_poison_fog_state(game, region_id, holder, key)
}

/// Полный End прежнего состояния тумана.
pub fn end_poison_fog_state<Game: ZonalCastGame>(
    game: &mut Game,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if game.resolve_state_move_shape(region_id, holder)
        .and_then(|shape| shape.applied_state::<PoisonFogState>(key)).is_none()
    {
        return false;
    }
    let _ = game.update_applied_state_end_visual(
        region_id, holder, key, ZonalCastPropertyTarget::Sufferer,
    );
    let Some((target_region, target)) = game.resolve_applied_state_sufferer(
        region_id, holder, key,
    ) else { return false };
    game.remove_applied_state_from(
        region_id, holder, key, (target_region, target), POISON_FOG_STATE_BYTES,
    )
}
