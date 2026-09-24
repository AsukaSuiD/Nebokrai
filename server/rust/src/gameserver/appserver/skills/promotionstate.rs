//! Живой Begin/restart усиления `CPromotionState` (`0x142`) в переходном Game.
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/promotionstate.cpp`. Данные, срок и сохраняемая запись
//! перенесены в Zone `effects/promotion.rs` (там же адреса конструкторов,
//! vtable, Serialize/Unserialize и Restart).
//!
//! Состояние остаётся в упорядоченной ветви `CFightDefense::PreDefense`:
//! для стихийной части удара множитель применяется в точном месте исходного
//! `m_vStates`. Второй коэффициент принадлежит лечению. Начальный пакет
//! состояния имеет исходный формат `0xBFE03`; отдельного пакета завершения
//! этот владелец не создаёт. Restart меняет только timestamp: прежние
//! длительность и коэффициенты сохраняются без нового пакета.
//! `AI` получает ключ достигнутого экземпляра из общего обхода состояний:
//! истечение удаляет только этот Promotion, не соседний одноимённый щит.
//! Primary Begin проверяет S, читает часы базы при U и отправляет одноразовый
//! visual до append. Запись хранит действительные U/S и собственный DB-span.

use super::shieldstate::DefenseShieldState;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) use nebokrai_zone::effects::{
    PROMOTION_STATE_BYTES, PROMOTION_STATE_ID, PromotionState,
};

pub(crate) const PROMOTION_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;

/// Повторный AI вызывает только Restart; новый primary проходит
/// CState::Begin(U,S) → visual loop0/Update0 → append в общую арену.
pub(crate) fn begin_or_restart_promotion_state(
    game: &mut CGame,
    user: Option<(i32, ShapeIdentity)>,
    sufferer: (i32, ShapeIdentity),
    new_parameters: impl FnOnce() -> (u32, u16, u16),
    now: &mut dyn FnMut() -> u32,
) -> Option<bool> {
    let shape = resolve_state_move_shape(game, sufferer.0, sufferer.1)?;
    let sufferer = (shape.shape().get_region_id(), shape.shape().identity());
    if let Some(key) = shape.defense_shield_key(PROMOTION_STATE_ID) {
        let state = resolve_state_move_shape_mut(game, sufferer.0, sufferer.1)?
            .applied_state_mut::<DefenseShieldState>(key)?;
        let DefenseShieldState::Promotion(state) = state else { return None; };
        state.restart(now());
        return Some(false);
    }
    let (keep_time_ms, magic_attack_factor, heal_recover_factor) = new_parameters();
    let mut state = PromotionState::new(keep_time_ms, magic_attack_factor, heal_recover_factor);
    if user.is_some() { state.restart(now()); }
    let user = match user {
        Some((region, identity)) => {
            let shape = resolve_state_move_shape(game, region, identity)?.shape();
            Some((shape.get_region_id(), shape.identity()))
        }
        None => None,
    };
    let mut message = CMessage::new(PROMOTION_STATE_BEGIN_MESSAGE);
    message.add_long(sufferer.1.object_type);
    message.add_long(sufferer.1.id);
    message.add_long(state.skill_id() as i32);
    message.add_long(state.client_time(&mut *now));
    message.add_long(0);
    let _ = game.send_move_shape_around(sufferer.0, sufferer.1, &message);
    let record = state.encoded_for_install();
    let shape = resolve_state_move_shape_mut(game, sufferer.0, sufferer.1)?;
    let key = shape.append_applied_state_record(DefenseShieldState::Promotion(state), &record);
    shape.mark_applied_state_begun(key);
    shape.set_applied_state_user(key, user);
    shape.set_applied_state_sufferer(key, Some(sufferer));
    // Общий каталог хранит уже завершённый loop0 visual после Update(0).
    Some(true)
}
