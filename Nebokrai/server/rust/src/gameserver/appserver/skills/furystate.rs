//! Каноническое накапливаемое состояние `CFuryState` (`0x1a3`).
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает добавление без
//! замены, строгую проверку истечения и последовательное
//! процентное увеличение только максимальной атаки игрока или монстра.
//! Округление использует исходное
//! правило дробной части `> 0.5`, а визуальные начало/завершение сохраняют
//! `0xBFE03/04`.
//! `GetRemainedTime` разделяет exact-тело `CFuryState` по `0x00605E10`:
//! положительный остаток вычисляется после отдельного второго чтения часов.
//! Serializer `0x005E7330` и exact `Unserialize` `0x005FD660` задают
//! 12-байтовую запись `ID + remaining time + attack gain`; несколько записей
//! сохраняют исходный порядок наложения.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};
use crate::nets::netserver::message::CMessage;

pub(crate) const FURY_STATE_SKILL_ID: u32 = 0x1a3;
pub(crate) const FURY_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct FuryState {
    started_at_ms: u32,
    keep_time_ms: u32,
    attack_gain_percent: i32,
}

impl FuryState {
    pub(crate) const fn new(
        started_at_ms: u32,
        keep_time_ms: u32,
        attack_gain_percent: i32,
    ) -> Self {
        Self {
            started_at_ms,
            keep_time_ms,
            attack_gain_percent,
        }
    }

    pub(crate) const fn skill_id(self) -> u32 {
        FURY_STATE_SKILL_ID
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != FURY_STATE_SKILL_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(0, reader.read_u32()?, reader.read_i32()?))
    }

    pub(crate) const fn activate_loaded(mut self, now_ms: u32) -> Self { self.started_at_ms = now_ms; self }
    pub(crate) fn encoded_for_install(self) -> [u8; FURY_STATE_BYTES] { self.encoded_with_remaining(self.keep_time_ms) }
    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> [u8; FURY_STATE_BYTES] { self.encoded_with_remaining(self.client_time(now_milliseconds) as u32) }
    fn encoded_with_remaining(self, remaining: u32) -> [u8; FURY_STATE_BYTES] {
        let mut bytes = [0; FURY_STATE_BYTES];
        bytes[..4].copy_from_slice(&FURY_STATE_SKILL_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&remaining.to_le_bytes());
        bytes[8..].copy_from_slice(&self.attack_gain_percent.to_le_bytes());
        bytes
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    pub(crate) fn apply_to_monster_max_attack(self, maximum: u32) -> u32 {
        let scaled = self.attack_gain_percent as f32 * 0.01 * maximum as f32;
        let truncated = scaled.trunc() as i32;
        let gain = if scaled - truncated as f32 > 0.5 {
            truncated.wrapping_add(1)
        } else {
            truncated
        };
        let result = maximum.wrapping_add(gain as u32);
        if result as i32 <= 0 {
            1
        } else {
            result.min(i32::MAX as u32)
        }
    }

    pub(crate) fn apply_to_player_maximum_attack(self, maximum: u32) -> u32 {
        let scaled = self.attack_gain_percent as f32 * 0.01 * maximum as f32;
        let truncated = scaled.trunc() as i32;
        let rounded = if scaled - truncated as f32 > 0.5 {
            truncated.wrapping_add(1)
        } else {
            truncated
        };
        let mut gain = rounded as u16 as u32;
        if maximum.wrapping_add(gain) > u16::MAX as u32 {
            gain = (u16::MAX as u32).wrapping_sub(maximum);
        }
        maximum.wrapping_add(gain).min(i32::MAX as u32)
    }
}

#[allow(clippy::too_many_arguments, reason = "поля задают точку фактической круговой доставки")]
pub(crate) fn send_fury_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: FuryState,
    begin: bool,
    now_ms: u32,
) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(|| now_ms));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn expire_monster_fury_states(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    now_ms: u32,
) -> usize {
    let expired = region.find_monster_by_id_mut(monster_id).map(|monster| {
        let identity = monster.move_shape().shape().identity();
        let tile_x = monster
            .move_shape()
            .shape()
            .get_tile_x()
            .unwrap_or_default();
        let tile_y = monster
            .move_shape()
            .shape()
            .get_tile_y()
            .unwrap_or_default();
        let states = monster
            .move_shape_mut()
            .take_expired_fury_states(now_ms);
        (identity, tile_x, tile_y, states)
    });
    let Some((identity, tile_x, tile_y, states)) = expired else {
        return 0;
    };
    let count = states.len();
    for state in states {
        send_fury_state_visual(
            game, region.id, identity, tile_x, tile_y, state, false, now_ms,
        );
    }
    count
}

pub(crate) fn expire_player_fury_states<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    now_ms: u32,
    _runtime: &mut Runtime,
) -> usize {
    let context = game.find_player(player_id).and_then(|player| {
        Some((
            player.server_region_id()?,
            player.shape().identity(),
            player.shape().get_tile_x().ok()?,
            player.shape().get_tile_y().ok()?,
        ))
    });
    let states = game
        .find_player_mut(player_id)
        .map(|player| player.take_expired_fury_states(now_ms))
        .unwrap_or_default();
    let count = states.len();
    if let Some((region_id, identity, tile_x, tile_y)) = context {
        for state in states {
            send_fury_state_visual(
                game, region_id, identity, tile_x, tile_y, state, false, now_ms,
            );
        }
    }
    if count != 0 {
        let _ = game.update_player_properties(player_id);
    }
    count
}

// COMPONENT_VARIANT_BEGIN: GameServer
// Точная пара: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SHA-256 EXE: 4F5C98E0FDF6147D8AECF55F7937AAF6E2CF5E4F5A2C44491A6359228762C80E
// SHA-256 PDB: B17BB9B7D69A9CC43E314C0E35C517830BB42CAA89416E173380AB17D2D66016
// Исходный владелец PDB: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\furystate.cpp

// ============================================================================
// FUNCTION: CFuryState::Unserialize
// STATUS: UNKNOWN (сохранены только метаданные исследования)
// COMPONENT: GameServer
// ARTIFACT: GameServer/gameserver.exe + GameServer/GameServer.pdb
// SOURCE: e:\svn\fengyun_russia_dev\server\gameserver\appserver\skills\furystate.cpp:156
// RVA: 0x001FD660
// ADDRESS: 005fd660
// PROTOTYPE: void __thiscall Unserialize(uchar * param_1, long * param_2)
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
//
// COMPONENT_VARIANT_END: GameServer
