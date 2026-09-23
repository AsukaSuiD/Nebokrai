//! Постоянное полное уклонение CEnlargeFullMissState.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/enlargefullmissstate.cpp/.h.
//! Property читает свежую S без ended-gate; только игрок получает WORD
//! wrapping-add младших 16 бит gain. Собственных visual и часов AI нет.
//! Primary Begin(U,S) в immediatestateinstallation читает базовые часы;
//! DB-restart Begin(NULL,S) сохраняет U/timestamp, обновляет S и снимает ended.
//! End отмечает ended и удаляет себя через свежий U, не подставляя держателя;
//! SetRegion меняет только регион U. SlotMap хранит базу отдельно от payload.
//! Default задаёт нулевой gain; DB8 — little-endian ID + i32 gain.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_base_applied_state, end_base_applied_state, resolve_state_move_shape,
};
use crate::gameserver::gameserver::game::CGame;
pub(crate) use nebokrai_zone::effects::{ENLARGE_FULL_MISS_STATE_BYTES, EnlargeFullMissState};

/// OnUpdateProperties 0x005E2120: GetSufferer, затем только player-формула.
/// Visual, IsEnded-gate и чтения часов у этого override отсутствуют.
pub(crate) fn update_enlarge_full_miss_state_properties(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    crate::gameserver::appserver::states::state::update_player_state_properties::<EnlargeFullMissState>(
        game, region_id, holder, key, |state, player| {
            player.update_state_combat_properties(|mut properties| { properties.full_miss = state.apply(properties.full_miss); properties });
        },
    )
}

pub(crate) fn restart_enlarge_full_miss_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<EnlargeFullMissState>(key)).is_none()
    {
        return false;
    }
    begin_base_applied_state(game, region_id, holder, key)
}

pub(crate) fn end_enlarge_full_miss_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<EnlargeFullMissState>(key)).is_none()
    {
        return false;
    }
    end_base_applied_state(game, region_id, holder, key, ENLARGE_FULL_MISS_STATE_BYTES)
}
