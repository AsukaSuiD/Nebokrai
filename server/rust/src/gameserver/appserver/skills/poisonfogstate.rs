//! Наложение, обновление и снятие ядовитого тумана у живой фигуры Game.
//! Источник: `GameServer/gameserver.exe` + `GameServer/GameServer.pdb`,
//! `appserver/skills/poisonfogstate.cpp` и `poisonfogstate.h`.
//! Тела Begin/update/restart/AI/End перенесены буквально в
//! `nebokrai_zone::skills::poisonfogstate` порцией T5 «zonalcast-хаб»; швы
//! арены реализованы над `CGame` в `skills/zonalcast.rs`. Здесь —
//! делегации с прежними сигнатурами; потребители (dispatch таблицы
//! `states/state.rs` и региональный обход `game/poisonfog.rs`) не меняются.
//! Для координатного Begin (0x00607E30) и typed Begin (0x00607EC0)
//! вызывающие цепочки не установлены; основной путь использует объектный Begin.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;

pub(crate) use nebokrai_zone::effects::{POISON_FOG_STATE_ID, PoisonFogState};

pub(crate) fn begin_primary_poison_fog_state(
    game: &mut CGame,
    holder_region: i32,
    holder: ShapeIdentity,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: Option<(i32, ShapeIdentity)>,
    state: PoisonFogState,
    now: &mut dyn FnMut() -> u32,
) -> Option<StateKey> {
    nebokrai_zone::skills::poisonfogstate::begin_primary_poison_fog_state(
        game, holder_region, holder, user, sufferer, state, now,
    )
}

pub(crate) fn update_poison_fog_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::poisonfogstate::update_poison_fog_state_properties(
        game, region_id, holder, key, now,
    )
}

pub(crate) fn restart_poison_fog_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::poisonfogstate::restart_poison_fog_state(
        game, region_id, holder, key, changing_region, now,
    )
}

pub(crate) fn update_poison_fog_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    nebokrai_zone::skills::poisonfogstate::update_poison_fog_state(game, region_id, holder, key, now_ms)
}

pub(crate) fn end_poison_fog_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    nebokrai_zone::skills::poisonfogstate::end_poison_fog_state(game, region_id, holder, key)
}
