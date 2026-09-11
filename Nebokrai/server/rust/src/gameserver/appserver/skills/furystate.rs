//! Каноническое накапливаемое состояние `CFuryState` (`0x1a3`).
//!
//! Точная пара `gameserver.exe + GameServer.pdb` подтверждает добавление без
//! замены, строгую проверку истечения и последовательное
//! процентное увеличение только максимальной атаки игрока или монстра.
//! Произведение signed-прибавки, `0.01_f32` и полного unsigned-максимума
//! вычисляется в x87, после чего `__ftol2` усекает его к нулю; игрок использует
//! младшие 16 бит результата. Визуальные начало/завершение сохраняют
//! `0xBFE03/04`.
//! `GetRemainedTime` разделяет exact-тело `CFuryState` по `0x00605E10`:
//! положительный остаток вычисляется после отдельного второго чтения часов.
//! Serializer `0x005E7330` и exact `Unserialize` `0x005FD660` задают
//! 12-байтовую запись `ID + remaining time + attack gain`; несколько записей
//! сохраняют исходный порядок наложения. Монстровая доставка использует
//! переданный region owner, пока он вынут из глобальной карты `CGame`.
//! Vtable `0x0065FB6C` связывает AI (`0x005EA4C0`) со строгим сроком,
//! а End (`0x006059A0` → `0x005DBCE0`) — с эффектом перед RemoveState.
//! Каждый экземпляр удаляется отдельно с собственным пересчётом игрока;
//! захваченный поколенческий ключ не подменяется после внешнего эффекта.
//! Порядок живых ключей сохраняет соответствие повторных DB-записей,
//! удаление оставляет пустой слот до общего уплотнения CMoveShape.
//! Общий CMoveShape::UpdateAbnormality передаёт один ключ; этот owner
//! не запускает отдельный семейный обход и сохраняет границы своих callbacks.

use crate::gameserver::appserver::moveshape::StateKey;

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::serverregion::CServerRegion;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::appserver::skills::thunder::truncate_original_i64_low;
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
        let gain = truncate_original_i64_low(
            f64::from(self.attack_gain_percent)
                * f64::from(0.01_f32)
                * f64::from(maximum),
        );
        let result = maximum.wrapping_add(gain as u32);
        if result as i32 <= 0 {
            1
        } else {
            result.min(i32::MAX as u32)
        }
    }

    pub(crate) fn apply_to_player_maximum_attack(self, maximum: u32) -> u32 {
        let mut gain = truncate_original_i64_low(
            f64::from(self.attack_gain_percent)
                * f64::from(0.01_f32)
                * f64::from(maximum),
        ) as u16 as u32;
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
    let message = fury_state_message(identity, state, begin, now_ms);
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}

pub(crate) fn send_fury_state_visual_in_region(
    game: &CGame,
    region: &CServerRegion,
    shape: &CShape,
    state: FuryState,
    begin: bool,
    now_ms: u32,
) {
    let message = fury_state_message(shape.identity(), state, begin, now_ms);
    let _ = game.send_game_shape_around(region, shape, None, &message);
}

fn fury_state_message(
    identity: ShapeIdentity,
    state: FuryState,
    begin: bool,
    now_ms: u32,
) -> CMessage {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(|| now_ms));
        message.add_long(0);
    }
    message
}

pub(crate) fn expire_monster_fury_state(
    game: &mut CGame,
    region: &mut CServerRegion,
    monster_id: i32,
    key: StateKey,
    now_ms: u32,
) -> usize {
    let Some((shape, state)) = region.find_monster_by_id(monster_id).and_then(|monster| {
        Some((monster.move_shape().shape().clone(),
            *monster.move_shape().applied_state::<FuryState>(key)?))
    }) else { return 0; };
    if !state.expired(now_ms) {
        return 0;
    }
    send_fury_state_visual_in_region(game, region, &shape, state, false, now_ms);
    let _ = region.find_monster_by_id_mut(monster_id)
        .and_then(|monster| monster.move_shape_mut().remove_fury_state_key(key));
    1
}

pub(crate) fn expire_player_fury_state<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    player_id: i32,
    key: StateKey,
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
    let Some(state) = game.find_player(player_id)
        .and_then(|player| player.move_shape().applied_state::<FuryState>(key)).copied()
        else { return 0; };
    if !state.expired(now_ms) {
        return 0;
    }
    if let Some((region_id, identity, tile_x, tile_y)) = context {
        send_fury_state_visual(
            game, region_id, identity, tile_x, tile_y, state, false, now_ms,
        );
    }
    let _ = game.find_player_mut(player_id)
        .and_then(|player| player.move_shape_mut().remove_fury_state_key(key));
    let _ = game.update_player_properties(player_id);
    1
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
