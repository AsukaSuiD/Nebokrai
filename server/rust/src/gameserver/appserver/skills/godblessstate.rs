//! Живые callbacks CGodBlessState/CGodBlessState2.
//! Источник: gameserver.exe + GameServer.pdb,
//! appserver/skills/godblessstate{,2}.cpp/.h. Тела перенесены буквально в
//! Zone `skills/godblessstate.rs` (порция №6a «state-касты пятёрки +
//! heal-квартет»; основание — Serialize `0x005EE310`, Unserialize
//! `0x00601830`, GetRemainedTime `0x00601480`, AI `0x00601640`,
//! OnUpdateProperties `0x00601690` — см. там). Здесь — делегации с
//! прежними сигнатурами и реэкспорты прежних владельцев данных; обход
//! состояний и потребители не меняются.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
pub(crate) use nebokrai_zone::effects::{GOD_BLESS_STATE_ID, GodBlessState};

pub(crate) fn update_god_bless_state_properties(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::godblessstate::update_god_bless_state_properties(game, region_id, holder, key, now)
}

pub(crate) fn restart_god_bless_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::godblessstate::restart_god_bless_state(game, region_id, holder, key, changing_region, now)
}

pub(crate) fn update_god_bless_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    nebokrai_zone::skills::godblessstate::update_god_bless_state(game, region_id, holder, key, now_ms)
}

/// Полный End: End-pakет адресату только у state12F, запись ended и
/// снятие с держателя через сохранённого S.
pub(crate) fn end_god_bless_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    nebokrai_zone::skills::godblessstate::end_god_bless_state(game, region_id, holder, key)
}
