//! Каноническое краткоживущее состояние `CCureState`.
//!
//! Источник: `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/skills/curestate.cpp`. Vtable `0x0065fb0c +0x0c` направляет AI
//! на `CBlindState::AI` (0x005d5ba0): при нулевом сроке состояние живо на
//! равенстве started == now и завершается лишь при started < now.
//! До завершения оно участвует в `OnChangeStates`.
//! End (`0x005FD420`) отправляет эффект до удаления; RemoveState затем
//! вызывает UpdateProperty. Та же цепочка действует при замене через LifeShield.
//! Exact vtable 0x0065FB0C направляет Serialize (+0x40) на 0x005F51E0,
//! Unserialize (+0x44) на 0x005EAAC0: запись — ID и remaining (8 байт),
//! не четыре identity поля базового CState. Unserialize сначала читает
//! clock, затем remaining в +0x38; runtime constructor задаёт там ноль.
//! GetRemainedTime 0x005F2CD0 использует +0x2C/+0x38 и условный второй clock.
//! restart_cure_state переносит object Begin 0x005EA0F0: nonnull sufferer,
//! base Begin(NULL, holder) без изменения timestamp → visual SetRun(1)
//! → Update(0)/Begin-пакет → base visual tail. Повторного clock Begin нет.
//! AI и замена монстрового Cure используют опубликованный настоящий region;
//! direct End заново разрешает holder после visual, не извлекает payload заранее.
//! Fury может накопить несколько Cure: общая арена CMoveShape сохраняет
//! идентичность каждого экземпляра, отдельный список — порядок и пустые места.
//! AI фиксирует начальную длину и перечитывает позиции; End удаляет ключ после visual,
//! не подменяя его новым одноимённым состоянием, созданным вложенным вызовом.
//! Загрузка активирует каждую запись; удаление синхронизирует DB-кодек и свойства.
//! Опубликованный generic End используется также AddCure щита жизни:
//! visual предшествует точному удалению, payload не извлекается до доставки.
//! После фактического удаления общий virtual UpdateProperty пересчитывает
//! живой tagProperty игрока либо состояние модификаторов региональной формы.

pub(crate) const CURE_STATE_SKILL_ID: u32 = 305;
pub(crate) const CURE_STATE_BYTES: usize = 8;

use super::manashieldstate::{
    MANA_SHIELD_STATE_BEGIN_MESSAGE, MANA_SHIELD_STATE_END_MESSAGE,
};
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::states::state::{
    timed_client_state_time,
    resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::gameserver::game::{CGame, game_tick_milliseconds};
use crate::nets::netserver::message::CMessage;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CureState {
    started_at_ms: u32,
    keep_time_ms: u32,
}

impl CureState {
    pub(crate) const fn new(_user: ShapeIdentity, _sufferer: ShapeIdentity) -> Self {
        Self { started_at_ms: 0, keep_time_ms: 0 }
    }

    /// Соответствует timestamp-записи унаследованного `CState::Begin`.
    pub(crate) fn begin_now(mut self) -> Self {
        self.started_at_ms = game_tick_milliseconds();
        self
    }


    pub(crate) const fn skill_id(self) -> u32 {
        CURE_STATE_SKILL_ID
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_time(self) -> i32 {
        self.client_state_time(game_tick_milliseconds) as i32
    }

    pub(crate) fn client_state_time(self, now: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now)
    }

    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != CURE_STATE_SKILL_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self { started_at_ms: now_ms, keep_time_ms: reader.read_u32()? })
    }

    pub(crate) fn encoded(self, now: impl FnMut() -> u32) -> [u8; CURE_STATE_BYTES] {
        self.encoded_with_remaining(self.client_state_time(now))
    }

    pub(crate) fn encoded_for_install(self) -> [u8; CURE_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }

    fn encoded_with_remaining(self, remaining: u32) -> [u8; CURE_STATE_BYTES] {
        let mut bytes = [0; CURE_STATE_BYTES];
        bytes[..4].copy_from_slice(&CURE_STATE_SKILL_ID.to_le_bytes());
        bytes[4..].copy_from_slice(&remaining.to_le_bytes());
        bytes
    }
}

pub(crate) fn restart_cure_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    now: &mut dyn FnMut() -> u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.cure_state_by_key(key))
    else { return false };
    if !crate::gameserver::appserver::states::state::begin_base_applied_state(
        game, region_id, holder, key,
    ) { return false }
    if crate::gameserver::appserver::states::state::begin_applied_state_visual(
        game, region_id, holder, key, 1,
    ) {
        let mut message = CMessage::new(MANA_SHIELD_STATE_BEGIN_MESSAGE);
        message.add_long(holder.object_type);
        message.add_long(holder.id);
        message.add_long(state.skill_id() as i32);
        message.add_ulong(state.client_state_time(&mut *now));
        message.add_long(0);
        let _ = game.send_move_shape_around(region_id, holder, &message);
        let _ = crate::gameserver::appserver::states::state::update_applied_state_visual_base(
            game, region_id, holder, key,
        );
    }
    true
}

pub(crate) fn end_player_cure_state(game: &mut CGame, player_id: i32) -> bool {
    let Some(key) = game.find_player(player_id)
        .and_then(|player| player.move_shape().cure_state_key()) else { return false };
    end_player_cure_state_key(game, player_id, key)
}

pub(crate) fn end_player_cure_state_key(game: &mut CGame, player_id: i32, key: StateKey) -> bool {
    let Some(player) = game.find_player(player_id) else { return false };
    let region_id = player.shape().get_region_id();
    let holder = player.shape().identity();
    end_cure_state_key(game, region_id, holder, key)
}


pub(crate) fn update_cure_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    let expired = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.cure_state_by_key(key))
        .is_some_and(|state| state.expired(now_ms));
    expired && end_cure_state_key(game, region_id, holder, key)
}

pub(crate) fn end_cure_state_key(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.cure_state_by_key(key)) else { return false };
    send_cure_state_visual_for_holder(game, region_id, holder, state, false);
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.remove_cure_state_by_key(key)).is_some();
    if removed {
        let _ = game.update_move_shape_properties(region_id, holder);
    }
    removed
}

pub(crate) fn send_cure_state_visual_for_holder(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    state: CureState,
    begin: bool,
) {
    let message = cure_state_message(holder, state, begin);
    let _ = game.send_move_shape_around(region_id, holder, &message);
}

pub(crate) fn send_cure_state_visual(
    game: &mut CGame,
    player_id: i32,
    state: CureState,
    begin: bool,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let identity = player.shape().identity();
    let message = cure_state_message(identity, state, begin);
    let _ = game.send_player_shape_around(player_id, None, &message);
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической круговой доставки")]
pub(crate) fn send_cure_state_visual_at(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: CureState,
    begin: bool,
) {
    let message = cure_state_message(identity, state, begin);
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn send_cure_state_visual_in_region(
    game: &CGame,
    region: &CServerRegion,
    shape: &CShape,
    state: CureState,
    begin: bool,
) {
    let message = cure_state_message(shape.identity(), state, begin);
    let _ = game.send_game_shape_around(region, shape, None, &message);
}

fn cure_state_message(identity: ShapeIdentity, state: CureState, begin: bool) -> CMessage {
    let mut message = CMessage::new(if begin {
        MANA_SHIELD_STATE_BEGIN_MESSAGE
    } else {
        MANA_SHIELD_STATE_END_MESSAGE
    });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(CURE_STATE_SKILL_ID as i32);
    if begin {
        message.add_long(state.client_time());
        message.add_long(0);
    }
    message
}



// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранён только не подключённый конструктор по умолчанию.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\curestate.cpp

// ============================================================================
// FUNCTION: CCureState::CCureState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\curestate.cpp:24
// RVA: 0x001E9EC0
// ADDRESS: 005e9ec0
// PROTOTYPE: undefined __thiscall CCureState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
