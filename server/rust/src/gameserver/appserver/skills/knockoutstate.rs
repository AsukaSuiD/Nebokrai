//! Оглушение CKnockOutState (0x192): запрет движения и боя до истечения
//! срока либо защитного действия. Источник: gameserver.exe + GameServer.pdb,
//! исходный владелец appserver/skills/knockoutstate.cpp.
//! Данные и 8-байтная запись перенесены в Zone `effects/blind.rs`
//! (конструкторы VA 0x005F4F30/0x005F4FA0, vtable 0x00660894).
//!
//! Vtable наследует AI/End/OnAction/codec от CBlindState. Строгий wrapping
//! deadline действует и при нулевом сроке; ключ арены отличает одноимённые
//! экземпляры. End: visual → fight/move-unlock → RemoveState → UpdateProperty.
//! Общий объектный Begin требует S, обновляет timestamp при U,
//! отправляет BFE03 и ставит оба запрета до публикации в выбранном caller-ом слоте.
//! Опубликованный derived region сохраняет доставку и callback для всех целей.
//! StartAllStates восстанавливает блокировки после загрузки.
//! Mosou и KnockOut заменяют первый одноимённый объект через полный End,
//! destructor и объектный Begin, публикуя новый экземпляр в прежней позиции.
//! BoaLock использует тот же Begin, но снимает первый ID 0x73 и добавляет
//! новый KnockOut в конец; эта отдельная политика остаётся у boalockattack.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::resolve_state_move_shape;
use crate::gameserver::gameserver::game::CGame;
use super::sealstate::SEAL_STATE_ID;
use super::blindstate::BLIND_STATE_ID;
use super::knightcutstate::KNIGHT_CUT_STATE_ID;

pub(crate) use nebokrai_zone::effects::{KNOCK_OUT_STATE_ID, KnockOutState};

pub(crate) fn replace_knock_out_state(
    game: &mut CGame,
    source: (i32, ShapeIdentity),
    target: (i32, ShapeIdentity),
    state: KnockOutState,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    super::blindstate::replace_primary_blind_state(game, source, target, state, now)
}


pub(crate) fn finish_player_knock_out_state_on_defense(game: &mut CGame, player_id: i32, _now_ms: u32) -> bool {
    let context = game.find_player(player_id).and_then(|player| {
        Some((player.server_region_id()?, player.shape().identity(),
            player.move_shape().applied_state_key::<KnockOutState>()?))
    });
    let Some((region_id, identity, key)) = context else { return false };
    super::blindstate::end_blind_state(game, region_id, identity, key)
}

pub(crate) fn finish_player_blind_states_on_defense(game: &mut CGame, player_id: i32, _now_ms: u32) -> bool {
    let context = game.find_player(player_id).and_then(|player| {
        Some((player.server_region_id()?, player.shape().identity(),
            player.move_shape().blind_state_instances()))
    });
    let Some((region_id, identity, order)) = context else { return false };
    let mut changed = false;
    for (key, state_id) in order {
        if matches!(state_id, BLIND_STATE_ID | KNOCK_OUT_STATE_ID | SEAL_STATE_ID | KNIGHT_CUT_STATE_ID) {
            changed |= super::blindstate::end_blind_state(game, region_id, identity, key);
        }
    }
    changed
}

pub(crate) fn finish_blind_states_on_defense(game: &mut CGame, region_id: i32, target: ShapeIdentity, now_ms: u32) -> bool {
    if target.object_type == 400 {
        return finish_player_blind_states_on_defense(game, target.id, now_ms);
    }
    let order = resolve_state_move_shape(game, region_id, target)
        .map(|shape| shape.blind_state_instances()).unwrap_or_default();
    let mut changed = false;
    for (key, state_id) in order {
        if matches!(state_id, BLIND_STATE_ID | KNOCK_OUT_STATE_ID | SEAL_STATE_ID | KNIGHT_CUT_STATE_ID) {
            changed |= super::blindstate::end_blind_state(game, region_id, target, key);
        }
    }
    changed
}
