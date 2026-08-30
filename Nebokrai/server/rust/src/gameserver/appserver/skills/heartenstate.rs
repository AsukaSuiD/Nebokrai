//! Каноническое достигнутое состояние `CHeartenState`.
//!
//! Состояние `324` хранит wrapping-часы и прибавляет знаковый параметр к
//! максимальному HP через `u32`, затем ограничивает результат `i32::MAX`.
//! Начальный визуальный пакет повторяется при каждом пересчёте свойств;
//! завершение публикуется при замене или строгом истечении срока. DB-запись
//! хранит остаток срока и знаковую прибавку максимального HP. Vtable exact EXE
//! направляет `GetRemainedTime` на общее тело `CBlindState` по `0x005F2CD0`.

use super::hearten::HEARTEN_SKILL_ID;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const HEARTEN_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
pub(crate) const HEARTEN_STATE_END_MESSAGE: i32 = 0x000b_fe04;
pub(crate) const HEARTEN_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HeartenState {
    started_at_ms: u32,
    keep_time_ms: u32,
    max_hp_gain: i32,
}

impl HeartenState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32, max_hp_gain: i32) -> Self {
        Self { started_at_ms, keep_time_ms, max_hp_gain }
    }
    pub(crate) const fn skill_id(self) -> u32 { HEARTEN_SKILL_ID }
    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }
    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }
    pub(crate) const fn apply(self, value: u32) -> u32 {
        let result = value.wrapping_add(self.max_hp_gain as u32);
        if result > i32::MAX as u32 { i32::MAX as u32 } else { result }
    }

    pub(crate) fn decode(
        payload: &[u8],
        offset: usize,
        now_ms: u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != HEARTEN_SKILL_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(now_ms, reader.read_u32()?, reader.read_i32()?))
    }

    pub(crate) fn encoded(
        self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> [u8; HEARTEN_STATE_BYTES] {
        self.encoded_with_remaining(self.client_time(now_milliseconds) as u32)
    }

    fn encoded_with_remaining(self, remaining_time_ms: u32) -> [u8; HEARTEN_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(HEARTEN_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(HEARTEN_SKILL_ID);
        writer.write_u32(remaining_time_ms);
        writer.write_i32(self.max_hp_gain);
        bytes
            .try_into()
            .expect("размер состояния воодушевления фиксирован")
    }

    pub(crate) fn encoded_for_install(self) -> [u8; HEARTEN_STATE_BYTES] {
        self.encoded_with_remaining(self.keep_time_ms)
    }

    pub(crate) fn activate_loaded(&mut self, now_ms: u32) {
        self.started_at_ms = now_ms;
    }
}

pub(crate) fn send_hearten_state_visual(
    game: &mut CGame,
    player_id: i32,
    state: HeartenState,
    begin: bool,
    now_milliseconds: impl FnMut() -> u32,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let identity = player.shape().identity();
    let mut message = CMessage::new(if begin {
        HEARTEN_STATE_BEGIN_MESSAGE
    } else {
        HEARTEN_STATE_END_MESSAGE
    });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(now_milliseconds));
        message.add_long(0);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) fn expire_player_hearten_state(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
) -> bool {
    let Some(state) = game
        .find_player_mut(player_id)
        .and_then(|player| player.take_expired_hearten_state(now_ms))
    else {
        return false;
    };
    send_hearten_state_visual(game, player_id, state, false, || now_ms);
    let _ = game.publish_player_states(player_id);
    true
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранён только не подключённый конструктор по умолчанию.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\heartenstate.cpp

// ============================================================================
// FUNCTION: CHeartenState::CHeartenState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\heartenstate.cpp:25
// RVA: 0x001EE580
// ADDRESS: 005ee580
// PROTOTYPE: undefined __thiscall CHeartenState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
