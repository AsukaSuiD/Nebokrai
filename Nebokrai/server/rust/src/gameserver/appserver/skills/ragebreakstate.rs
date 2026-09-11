//! Каноническое состояние подготовки яростного удара `CRageBreakState` (`0x6E`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/ragebreakstate.cpp`. Новое наложение заменяет первый
//! найденный экземпляр, не удаляя остальные загруженные записи. Состояние
//! строго истекает после `started + keep`, увеличивает только
//! максимальную атаку. Signed-прибавка, `0.01_f32` и полный unsigned-максимум
//! перемножаются в x87, а общее с `CFuryState` тело `__ftol2` усекает результат
//! к нулю. Для игрока прибавка сужается до `WORD` и ограничивается суммой
//! `0xFFFF`; начало и завершение публикуются как `0xBFE03/04`.
//! Клиентский `GetRemainedTime` подтверждён ссылкой vtable на общее тело
//! `CFuryState` по `0x00605E10` и сохраняет два чтения wrapping clock.
//! Общий serializer `0x005E7330` и exact `Unserialize` `0x005FD660`
//! задают 12 байт: `ID + remaining time + attack gain`; spatial login
//! восстанавливает срок до общего пересчёта свойств.
//! Вызов Restart из Fury (vtable `0x006612B4 +0x20`, `0x005FD450`)
//! меняет только время начала, сохраняя прежние срок, усиление и DB-запись.
//! End (slot +0x1C, `0x005FD420`) сначала отправляет эффект, затем удаляет
//! состояние через RemoveState с пересчётом свойств игрока. Замена и AI
//! используют один этот порядок.
//! Достигнутый AI получает один поколенческий ключ общей арены;
//! порядок вызовов и границу прохода задаёт общий CMoveShape::UpdateAbnormality.
//! Любое удаление адресует тот же экземпляр, а не первый дубль.
//! AI/End разрешают общий CMoveShape по region/type/id; RTTI-ограничения
//! формул игрока не запрещают жизненный цикл региональных держателей.
//! После visual владелец перечитывается; UpdateProperty вызывается только
//! для игрока и только при фактическом удалении этой записи.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::skills::thunder::truncate_original_i64_low;
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const RAGE_BREAK_STATE_ID: u32 = 0x6e;
pub(crate) const RAGE_BREAK_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RageBreakState {
    started_at_ms: u32,
    keep_time_ms: u32,
    attack_gain_percent: i32,
}

impl RageBreakState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32, attack_gain_percent: i32) -> Self {
        Self { started_at_ms, keep_time_ms, attack_gain_percent }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != RAGE_BREAK_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self::new(0, reader.read_u32()?, reader.read_i32()?))
    }

    pub(crate) const fn activate_loaded(mut self, now_ms: u32) -> Self { self.started_at_ms = now_ms; self }
    pub(crate) fn encoded_for_install(self) -> [u8; RAGE_BREAK_STATE_BYTES] { self.encoded_with_remaining(self.keep_time_ms) }
    pub(crate) fn encoded(self, now_milliseconds: impl FnMut() -> u32) -> [u8; RAGE_BREAK_STATE_BYTES] { self.encoded_with_remaining(self.client_time(now_milliseconds) as u32) }
    fn encoded_with_remaining(self, remaining: u32) -> [u8; RAGE_BREAK_STATE_BYTES] {
        let mut bytes = [0; RAGE_BREAK_STATE_BYTES];
        bytes[..4].copy_from_slice(&RAGE_BREAK_STATE_ID.to_le_bytes());
        bytes[4..8].copy_from_slice(&remaining.to_le_bytes());
        bytes[8..].copy_from_slice(&self.attack_gain_percent.to_le_bytes());
        bytes
    }

    pub(crate) const fn skill_id(self) -> u32 { RAGE_BREAK_STATE_ID }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> i32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds) as i32
    }

    fn truncated_gain(self, maximum: u32) -> i32 {
        truncate_original_i64_low(
            f64::from(self.attack_gain_percent)
                * f64::from(0.01_f32)
                * f64::from(maximum),
        )
    }

    pub(crate) fn apply_to_player_maximum_attack(self, maximum: u32) -> u32 {
        let mut gain = self.truncated_gain(maximum) as u16 as u32;
        if maximum.wrapping_add(gain) > u16::MAX as u32 {
            gain = (u16::MAX as u32).wrapping_sub(maximum);
        }
        maximum.wrapping_add(gain).min(i32::MAX as u32)
    }
}

pub(crate) fn end_player_rage_break_state(game: &mut CGame, player_id: i32, _now_ms: u32) -> bool {
    let Some((region_id, holder, key)) = game.find_player(player_id).and_then(|player| {
        Some((player.shape().get_region_id(), player.shape().identity(),
            player.move_shape().applied_state_key::<RageBreakState>()?))
    }) else { return false };
    end_rage_break_state_key(game, region_id, holder, key)
}

fn end_rage_break_state_key(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<RageBreakState>(key)).copied()
        else { return false };
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state.skill_id() as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.remove_applied_state_record::<RageBreakState>(key, RAGE_BREAK_STATE_BYTES))
        .is_some();
    if removed && holder.object_type == 400 {
        let _ = game.update_player_properties(holder.id);
    }
    true
}

pub(crate) fn update_rage_break_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    if !resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<RageBreakState>(key))
        .is_some_and(|state| state.expired(now_ms)) {
        return false;
    }
    end_rage_break_state_key(game, region_id, holder, key)
}

pub(crate) fn send_rage_break_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: RageBreakState,
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
