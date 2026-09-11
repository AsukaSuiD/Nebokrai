//! Каноническое состояние `CCallosityState` и общий владелец пары закалок.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `appserver/skills/callositystate.cpp` и `callositystate2.cpp`. Конкретное
//! второе состояние реализовано в `callositystate2.rs`; enum семейства задаёт
//! конкретный вариант каждого экземпляра общей арены. Обычное наложение
//! заменяет первый найденный экземпляр, загруженные дубли не схлопываются.
//! Vtable первой/второй закалки `0x006607d4/0x006603bc`, слот +0x0c,
//! направляет AI на `0x005d60b0`: строгий абсолютный wrapping deadline.
//! Общий End `0x005fd420` отправляет `0xBFE04` до RemoveState, который
//! вызывает CPlayer::UpdateProperty (`vtable +0x9c`, `0x004593e0`).
//! Exact vtable обеих закалок направляет `GetRemainedTime` на `0x005D5F30`;
//! additional-data остаётся базовым нулём.
//! Общая exact-пара `Serialize/Unserialize` `0x005F1050/0x005F48E0` сохраняет
//! `ID + remaining time + WORD blast factor`; этот формат разделяется с
//! `CAgilityState2` и восстанавливается на player-login до пересчёта свойств.
//! Коэффициент `CCH` применяется только при общем `UpdateProperty`; каждый
//! такой проход повторно публикует начальный визуальный эффект, как
//! `OnUpdateProperties`.
//! Достигнутый AI обходит исходный набор поколенческих ключей общей арены:
//! повторные записи сохраняются, после удаления и публикаций следующий
//! экземпляр разрешается заново; новые экземпляры в этот проход не входят.

use super::callosity::CALLOSITY_SKILL_ID;
use super::callosity2::CALLOSITY_2_SKILL_ID;
use super::callositystate2::CallosityState2;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::appserver::states::state::{default_additional_data, timed_client_state_time};
use crate::gameserver::gameserver::game::{CGame, game_tick_milliseconds};
use crate::nets::netserver::message::CMessage;

pub(crate) const CALLOSITY_STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
pub(crate) const CALLOSITY_STATE_BYTES: usize = 10;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CallosityState {
    blast_factor: u16,
    started_at_ms: u32,
    time_to_keep: i32,
}

impl CallosityState {
    pub(crate) const fn new(blast_factor: u16, started_at_ms: u32, time_to_keep: i32) -> Self {
        Self {
            blast_factor,
            started_at_ms,
            time_to_keep,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        CALLOSITY_SKILL_ID
    }

    pub(crate) const fn blast_factor(self) -> u16 {
        self.blast_factor
    }

    pub(crate) const fn time_to_keep(self) -> i32 {
        self.time_to_keep
    }

    pub(crate) const fn activate_loaded(mut self, now_ms: u32) -> Self {
        self.started_at_ms = now_ms;
        self
    }

    pub(crate) fn client_state_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(
            self.started_at_ms,
            self.time_to_keep as u32,
            now_milliseconds,
        ) as i32
    }

    pub(crate) const fn additional_data(self) -> u32 {
        default_additional_data()
    }

    pub(crate) const fn apply_to_player(
        self,
        mut properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        properties.cch = properties.cch.wrapping_add(self.blast_factor);
        properties
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CallosityFamilyState {
    Callosity(CallosityState),
    Callosity2(CallosityState2),
}

impl CallosityFamilyState {
    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        let (started, keep) = match self {
            Self::Callosity(state) => (state.started_at_ms, state.time_to_keep),
            Self::Callosity2(state) => (state.started_at_ms(), state.time_to_keep()),
        };
        started.wrapping_add(keep as u32) < now_ms
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        let skill_id = reader.read_u32()?;
        let keep_time_ms = reader.read_i32()?;
        let blast_factor = reader.read_u16()?;
        match skill_id {
            CALLOSITY_SKILL_ID => Ok(Self::Callosity(CallosityState::new(
                blast_factor,
                0,
                keep_time_ms,
            ))),
            CALLOSITY_2_SKILL_ID => Ok(Self::Callosity2(CallosityState2::new(
                blast_factor,
                0,
                keep_time_ms,
            ))),
            _ => Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            }),
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        match self {
            Self::Callosity(state) => state.skill_id(),
            Self::Callosity2(state) => state.skill_id(),
        }
    }

    pub(crate) fn client_state_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        match self {
            Self::Callosity(state) => state.client_state_time(now_milliseconds),
            Self::Callosity2(state) => state.client_state_time(now_milliseconds),
        }
    }

    pub(crate) const fn additional_data(self) -> u32 {
        match self {
            Self::Callosity(state) => state.additional_data(),
            Self::Callosity2(state) => state.additional_data(),
        }
    }

    pub(crate) const fn activate_loaded(self, now_ms: u32) -> Self {
        match self {
            Self::Callosity(state) => Self::Callosity(state.activate_loaded(now_ms)),
            Self::Callosity2(state) => Self::Callosity2(state.activate_loaded(now_ms)),
        }
    }

    pub(crate) fn encoded(self, now_ms: u32) -> [u8; CALLOSITY_STATE_BYTES] {
        let mut bytes = [0; CALLOSITY_STATE_BYTES];
        bytes[..4].copy_from_slice(&self.skill_id().to_le_bytes());
        bytes[4..8].copy_from_slice(&self.client_state_time(|| now_ms).to_le_bytes());
        let blast_factor = match self {
            Self::Callosity(state) => state.blast_factor(),
            Self::Callosity2(state) => state.blast_factor(),
        };
        bytes[8..].copy_from_slice(&blast_factor.to_le_bytes());
        bytes
    }

    pub(crate) fn encoded_for_install(self) -> [u8; CALLOSITY_STATE_BYTES] {
        let started_at_ms = match self {
            Self::Callosity(state) => state.started_at_ms,
            Self::Callosity2(state) => state.started_at_ms(),
        };
        self.encoded(started_at_ms)
    }

    pub(crate) const fn apply_to_player(
        self,
        properties: PlayerCombatProperties,
    ) -> PlayerCombatProperties {
        match self {
            Self::Callosity(state) => state.apply_to_player(properties),
            Self::Callosity2(state) => state.apply_to_player(properties),
        }
    }
}

pub(crate) fn end_player_callosity_state(game: &mut CGame, player_id: i32) -> bool {
    let Some(key) = game.find_player(player_id)
        .and_then(|player| player.move_shape().applied_state_key::<CallosityFamilyState>()) else {
        return false;
    };
    end_player_callosity_state_key(game, player_id, key)
}

fn end_player_callosity_state_key(game: &mut CGame, player_id: i32, key: crate::gameserver::appserver::moveshape::StateKey) -> bool {
    let Some((state, identity)) = game.find_player(player_id).and_then(|player| {
        Some((*player.move_shape().applied_state::<CallosityFamilyState>(key)?, player.shape().identity()))
    }) else { return false };
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    let _ = game.send_player_shape_around(player_id, None, &message);
    if let Some(player) = game.find_player_mut(player_id) {
        player.move_shape_mut().remove_applied_state_record::<CallosityFamilyState>(key, CALLOSITY_STATE_BYTES);
    }
    let _ = game.update_player_properties(player_id);
    true
}

pub(crate) fn expire_player_callosity_state(game: &mut CGame, player_id: i32, now_ms: u32) -> bool {
    let keys = game.find_player(player_id)
        .map(|player| player.move_shape().applied_state_keys::<CallosityFamilyState>()).unwrap_or_default();
    let mut ended = false;
    for key in keys {
        if game.find_player(player_id)
            .and_then(|player| player.move_shape().applied_state::<CallosityFamilyState>(key))
            .is_some_and(|state| state.expired(now_ms)) {
            ended |= end_player_callosity_state_key(game, player_id, key);
        }
    }
    ended
}

pub(crate) fn send_callosity_state_begin(
    game: &mut CGame,
    player_id: i32,
    state: CallosityFamilyState,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let identity = player.shape().identity();
    let mut message = CMessage::new(CALLOSITY_STATE_BEGIN_MESSAGE);
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    message.add_long(state.client_state_time(game_tick_milliseconds));
    message.add_ulong(state.additional_data());
    let _ = game.send_player_shape_around(player_id, None, &message);
}

// Статус оставшихся контрактов: UNKNOWN; декомпилят хранится локально
// Декомпилятор: Ghidra 12.1.2
// Сохранён только не подключённый конструктор по умолчанию.

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\callositystate.cpp

// ============================================================================
// FUNCTION: CCallosityState::CCallosityState
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\callositystate.cpp:25
// RVA: 0x001F4600
// ADDRESS: 005f4600
// PROTOTYPE: undefined __thiscall CCallosityState(void)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
//


// COMPONENT_VARIANT_END: GameServer
