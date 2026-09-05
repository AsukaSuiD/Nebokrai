//! Каноническое краткоживущее состояние `CCureState`.
//!
//! Источник: `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/skills/curestate.cpp`. Vtable `0x0065fb0c +0x0c` направляет AI
//! на `CBlindState::AI` (0x005d5ba0): при нулевом сроке состояние живо на
//! равенстве started == now и завершается лишь при started < now.
//! До завершения оно участвует в `OnChangeStates`.
//! End (`0x005FD420`) отправляет эффект до удаления; RemoveState затем
//! вызывает UpdateProperty. Та же цепочка действует при замене через LifeShield.
//! Запись сохраняет ID и четыре `long` базового `CState`:
//! user type/ID и sufferer type/ID. Это же представление читается при
//! входе и удаляется вместе с каноническим однотиковым состоянием. Vtable
//! exact EXE направляет `GetRemainedTime` на `CBlindState` (`0x005F2CD0`),
//! а нулевая длительность задаётся конструктором самого `CCureState`.
//! Монстровая доставка использует текущий region owner, а не повторный lookup
//! в `CGame` во время owner-side AI-прохода.

pub(crate) const CURE_STATE_SKILL_ID: u32 = 305;
pub(crate) const CURE_STATE_BYTES: usize = 20;

use super::manashieldstate::{
    MANA_SHIELD_STATE_BEGIN_MESSAGE, MANA_SHIELD_STATE_END_MESSAGE,
};
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::states::state::{
    decode_state_identities, encode_state_identities, timed_client_state_time,
};
use crate::gameserver::gameserver::game::{CGame, game_tick_milliseconds};
use crate::nets::netserver::message::CMessage;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CureState {
    user: ShapeIdentity,
    sufferer: ShapeIdentity,
    started_at_ms: u32,
}

impl CureState {
    pub(crate) const fn new(user: ShapeIdentity, sufferer: ShapeIdentity) -> Self {
        Self { user, sufferer, started_at_ms: 0 }
    }

    /// Соответствует timestamp-записи унаследованного `CState::Begin`.
    pub(crate) fn begin_now(mut self) -> Self {
        self.started_at_ms = game_tick_milliseconds();
        self
    }

    pub(crate) const fn activate_loaded(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }

    pub(crate) const fn skill_id(self) -> u32 {
        CURE_STATE_SKILL_ID
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms < now_ms
    }

    pub(crate) fn client_time(self) -> i32 {
        timed_client_state_time(self.started_at_ms, 0, game_tick_milliseconds) as i32
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != CURE_STATE_SKILL_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        let (user, sufferer) = decode_state_identities(payload, reader.position())?;
        Ok(Self::new(user, sufferer))
    }

    pub(crate) fn encoded(self) -> [u8; CURE_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(CURE_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(CURE_STATE_SKILL_ID);
        writer.write_bytes(&encode_state_identities(self.user, self.sufferer));
        bytes.try_into().expect("размер состояния очищения фиксирован")
    }
}

pub(crate) fn end_player_cure_state(game: &mut CGame, player_id: i32) -> bool {
    let Some(state) = game.find_player(player_id).and_then(|player| player.cure_state()) else {
        return false;
    };
    send_cure_state_visual(game, player_id, state, false);
    let _ = game.find_player_mut(player_id).and_then(|player| player.take_cure_state());
    let _ = game.update_player_properties(player_id);
    true
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

pub(crate) fn expire_monster_cure_state(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    now_ms: u32,
) -> bool {
    let expired = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let state = monster.move_shape_mut().take_cure_state_for_ai(now_ms)?;
        Some((state, monster.move_shape().shape().clone()))
    });
    let Some((state, shape)) = expired else {
        return false;
    };
    send_cure_state_visual_in_region(game, region, &shape, state, false);
    true
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
