//! Каноническое краткоживущее состояние `CCureState`.
//!
//! Источник: `gameserver.exe + GameServer.pdb`, владелец
//! `appserver/skills/curestate.cpp`. Класс не переопределяет `AI`, поэтому
//! унаследованный `CState::AI` завершает его на следующем снимке
//! `UpdateAbnormality`; состояние успевает участвовать в `OnChangeStates`.
//! Запись сохраняет ID и четыре `long` базового `CState`:
//! user type/ID и sufferer type/ID. Это же представление читается при
//! входе и удаляется вместе с каноническим однотиковым состоянием.

pub(crate) const CURE_STATE_SKILL_ID: u32 = 305;
pub(crate) const CURE_STATE_BYTES: usize = 20;

use super::manashieldstate::{
    MANA_SHIELD_STATE_BEGIN_MESSAGE, MANA_SHIELD_STATE_END_MESSAGE,
};
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CureState {
    user: ShapeIdentity,
    sufferer: ShapeIdentity,
}

impl CureState {
    pub(crate) const fn new(user: ShapeIdentity, sufferer: ShapeIdentity) -> Self {
        Self { user, sufferer }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        CURE_STATE_SKILL_ID
    }

    pub(crate) const fn client_time(self) -> i32 {
        0
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
        Ok(Self::new(
            ShapeIdentity {
                object_type: reader.read_i32()?,
                id: reader.read_i32()?,
                ex_id: crate::public::guid::CGuid::GUID_INVALID,
            },
            ShapeIdentity {
                object_type: reader.read_i32()?,
                id: reader.read_i32()?,
                ex_id: crate::public::guid::CGuid::GUID_INVALID,
            },
        ))
    }

    pub(crate) fn encoded(self) -> [u8; CURE_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(CURE_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(CURE_STATE_SKILL_ID);
        writer.write_i32(self.user.object_type);
        writer.write_i32(self.user.id);
        writer.write_i32(self.sufferer.object_type);
        writer.write_i32(self.sufferer.id);
        bytes.try_into().expect("размер состояния очищения фиксирован")
    }
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
) -> bool {
    let expired = region.find_monster_by_id_mut(monster_id).and_then(|monster| {
        let state = monster.move_shape_mut().take_cure_state_for_ai()?;
        Some((
            state,
            monster.move_shape().shape().identity(),
            monster.move_shape().shape().get_tile_x().unwrap_or_default(),
            monster.move_shape().shape().get_tile_y().unwrap_or_default(),
        ))
    });
    let Some((state, identity, tile_x, tile_y)) = expired else {
        return false;
    };
    send_cure_state_visual_at(
        game, region.id, identity, tile_x, tile_y, state, false,
    );
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
