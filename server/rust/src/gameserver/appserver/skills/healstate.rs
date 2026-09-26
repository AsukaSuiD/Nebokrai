//! Живые Begin/restart/AI/End периодического лечения в переходном Game:
//! CHealState/2 и CSuperHealState/2. Источник: gameserver.exe +
//! GameServer.pdb, appserver/skills/{healstate,healstate2,superhealstate,
//! superhealstate2}.cpp и первичные heal*.cpp. Тела перенесены буквально в
//! Zone `skills/healstate.rs` (порция №6a «state-касты пятёрки +
//! heal-квартет»; машинные факты квартета — AI/End/Unserialize fold
//! `0x1EEDF0/0x1EEBA0/0x1EEC70`, Serialize 5-fold `0x1F65F0` с CRoarState
//! — см. там). Здесь — делегации с прежними сигнатурами и реэкспорт
//! прежнего владельца данных; обход состояний и потребители не меняются.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use nebokrai_zone::effects::HealState;

/// Замена первого состояния своего ID: End/destructor прежнего, затем
/// ctor(J,J) и Begin(U,S) до append; `create` читается после удаления.
pub(super) fn replace_heal_state(
    game: &mut CGame,
    source: (i32, ShapeIdentity),
    storage_target: (i32, ShapeIdentity),
    skill_id: u32,
    create: impl FnOnce() -> HealState,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::healstate::replace_heal_state(game, source, storage_target, skill_id, create, now)
}

pub(crate) fn restart_heal_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    changing_region: bool, now: &mut dyn FnMut() -> u32,
) -> bool {
    nebokrai_zone::skills::healstate::restart_heal_state(game, region_id, holder, key, changing_region, now)
}

pub(crate) fn update_stored_heal_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
    now: impl FnMut() -> u32,
) {
    nebokrai_zone::skills::healstate::update_stored_heal_state(game, region_id, holder, key, now)
}

/// Полный End по ключу: страдальцу сохранённого identity, не записывая ended.
pub(crate) fn end_heal_state(
    game: &mut CGame, region_id: i32, holder: ShapeIdentity, key: StateKey,
) -> bool {
    nebokrai_zone::skills::healstate::end_heal_state(game, region_id, holder, key)
}
