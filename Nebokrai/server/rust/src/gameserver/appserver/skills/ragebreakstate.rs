//! Каноническое состояние подготовки яростного удара `CRageBreakState` (`0x6E`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/ragebreakstate.cpp`. Состояние заменяет предыдущий
//! экземпляр, строго истекает после `started + keep`, увеличивает только
//! максимальную атаку и сохраняет исходное округление с границей дробной
//! части `> 0.5`. Для игрока прибавка сужается до `WORD` и ограничивается
//! суммой `0xFFFF`; начало и завершение публикуются как `0xBFE03/04`.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const RAGE_BREAK_STATE_ID: u32 = 0x6e;

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

    pub(crate) const fn skill_id(self) -> u32 { RAGE_BREAK_STATE_ID }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) const fn remaining_ms(self, now_ms: u32) -> i32 {
        let deadline = self.started_at_ms.wrapping_add(self.keep_time_ms);
        if deadline <= now_ms { 0 } else { deadline.wrapping_sub(now_ms) as i32 }
    }

    fn rounded_gain(self, maximum: u32) -> i32 {
        let scaled = self.attack_gain_percent as f32 * 0.01 * maximum as f32;
        let truncated = scaled.trunc() as i32;
        if scaled - truncated as f32 > 0.5 { truncated.wrapping_add(1) } else { truncated }
    }

    pub(crate) fn apply_to_player_maximum_attack(self, maximum: u32) -> u32 {
        let mut gain = self.rounded_gain(maximum) as u16 as u32;
        if maximum.wrapping_add(gain) > u16::MAX as u32 {
            gain = (u16::MAX as u32).wrapping_sub(maximum);
        }
        maximum.wrapping_add(gain).min(i32::MAX as u32)
    }
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
        message.add_long(state.remaining_ms(now_ms));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}
