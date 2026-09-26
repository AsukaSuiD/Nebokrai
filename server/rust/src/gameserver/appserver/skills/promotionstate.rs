//! Живой Begin/restart усиления `CPromotionState` (`0x142`) в переходном Game.
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/promotionstate.cpp`. Тело перенесено буквально в Zone
//! `skills/promotionstate.rs` (порция №6a «state-касты пятёрки +
//! heal-квартет»; основание — Restart-fold `0x005FD450` — см. там);
//! данные, срок и сохраняемая запись перенесены ранее в Zone
//! `effects/promotion.rs`. Здесь — делегация с прежней сигнатурой;
//! прежний реэкспорт байт записи убран волной Z-M2b: его единственный
//! потребитель (арена hub moveshape) перенесён в Zone `skills::state`.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;

/// Повторный AI вызывает только Restart; новый primary проходит
/// CState::Begin(U,S) → visual loop0/Update0 → append в общую арену.
pub(crate) fn begin_or_restart_promotion_state(
    game: &mut CGame,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: (i32, ShapeIdentity),
    new_parameters: impl FnOnce() -> (u32, u16, u16),
    now: &mut dyn FnMut() -> u32,
) -> Option<bool> {
    nebokrai_zone::skills::promotionstate::begin_or_restart_promotion_state(
        game, user, sufferer, new_parameters, now,
    )
}
