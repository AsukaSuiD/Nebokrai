//! Оглушение CKnockOutState (0x192): запрет движения и боя до истечения
//! срока либо защитного действия. Источник: gameserver.exe + GameServer.pdb,
//! исходный владелец appserver/skills/knockoutstate.cpp.
//!
//! Vtable наследует AI/End/OnAction/codec от CBlindState. Строгий wrapping
//! deadline действует и при нулевом сроке; ключ арены отличает одноимённые
//! экземпляры. End: visual → fight/move-unlock → RemoveState → UpdateProperty.
//! Общий объектный Begin требует S, обновляет timestamp при U,
//! отправляет BFE03 и ставит оба запрета до публикации в выбранном caller-ом слоте.
//! Опубликованный derived region сохраняет доставку и callback для всех целей.
//! DB-запись ID/remaining занимает 8 байт; StartAllStates восстанавливает
//! блокировки после загрузки. RAW координатной и типизированной перегрузок
//! Begin (0x005F5020/0x005F5100) сохранён отдельно от объектного пути.
//! Mosou и KnockOut заменяют первый одноимённый объект через полный End,
//! destructor и объектный Begin, публикуя новый экземпляр в прежней позиции.
//! BoaLock использует тот же Begin, но снимает первый ID 0x73 и добавляет
//! новый KnockOut в конец; эта отдельная политика остаётся у boalockattack.

use nebokrai_shared::protocol::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    resolve_state_move_shape, timed_client_state_time,
};
use crate::gameserver::gameserver::game::CGame;
use super::sealstate::SEAL_STATE_ID;
use super::blindstate::BLIND_STATE_ID;
use super::knightcutstate::KNIGHT_CUT_STATE_ID;

pub(crate) const KNOCK_OUT_STATE_ID: u32 = 0x192;
pub(crate) const KNOCK_OUT_STATE_BYTES: usize = 8;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct KnockOutState { started_at_ms: u32, keep_time_ms: u32 }

impl KnockOutState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32) -> Self { Self { started_at_ms, keep_time_ms } }
    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != KNOCK_OUT_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(now_ms, reader.read_u32()?))
    }
    pub(crate) const fn skill_id(self) -> u32 { KNOCK_OUT_STATE_ID }
    pub(crate) const fn expired(self, now_ms: u32) -> bool { self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms }
    pub(crate) fn encoded_for_install(self) -> [u8; KNOCK_OUT_STATE_BYTES] {
        let mut bytes = [0; KNOCK_OUT_STATE_BYTES];
        bytes[..4].copy_from_slice(&KNOCK_OUT_STATE_ID.to_le_bytes());
        bytes[4..].copy_from_slice(&self.keep_time_ms.to_le_bytes());
        bytes
    }
    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }
}

impl super::blindstate::BlindStatePayload for KnockOutState {
    fn blind_state_id(&self) -> u32 { KNOCK_OUT_STATE_ID }
    fn begin_at(&mut self, now_ms: u32) { self.started_at_ms = now_ms; }
    fn remaining(&self, now: &mut dyn FnMut() -> u32) -> u32 { self.client_time(now) as u32 }
    fn install_record(&self) -> [u8; KNOCK_OUT_STATE_BYTES] { self.encoded_for_install() }
}

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

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\knockoutstate.cpp
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\knockoutstate.h

// ============================================================================
// FUNCTION: CKnockOutState::CKnockOutState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\knockoutstate.cpp:24
// RVA: 0x001F4FA0
// ADDRESS: 005f4fa0
// PROTOTYPE: undefined __thiscall CKnockOutState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CKnockOutState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\knockoutstate.cpp:57
// RVA: 0x001F5020
// ADDRESS: 005f5020
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, long param_2, long param_3)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// ============================================================================
// FUNCTION: CKnockOutState::Begin
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\knockoutstate.cpp:73
// RVA: 0x001F5100
// ADDRESS: 005f5100
// PROTOTYPE: int __thiscall Begin(CMoveShape * param_1, OBJECT_TYPE param_2, long param_3, long param_4)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//

// COMPONENT_VARIANT_END: GameServer
