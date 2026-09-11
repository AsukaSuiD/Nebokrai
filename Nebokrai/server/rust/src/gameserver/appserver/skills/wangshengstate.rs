//! Сохранённое состояние `CWangshengState` (`0x221`).
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, исходный owner
//! `appserver/skills/wangshengstate.cpp`. Общий codec `CFuryState` сохраняет
//! ID, остаток времени и знаковую прибавку в 12 байтах. Состояние независимо
//! от active-skill owner-а `wangsheng.rs`: exact `CWangsheng::AI` лечит
//! напрямую и его не создаёт. Загруженная legacy-запись использует timed AI
//! `CPobingState`; её `OnUpdateProperties` ставит HP в максимум только когда
//! `current + gain >= max`, а при меньшем результате не меняет HP.
//! Достигнутый AI получает один поколенческий ключ общей арены;
//! порядок вызовов и границу прохода задаёт общий CMoveShape::UpdateAbnormality.
//! Любое удаление адресует тот же экземпляр, а не первый дубль.
//! AI/End разрешают общий CMoveShape по region/type/id; правила свойств
//! игрока не запрещают жизненный цикл региональных держателей.
//! Exact vtable 0x0066208C: timed AI 0x005E6E20 вызывает End(false)
//! из slot +0x48 (0x005E7310): visual → базовый End → GetUser → RemoveState.
//! Это не прямой slot +0x1C (0x005DBCE0), который сам visual не отправляет.
//! UpdateProperty вызывается только для игрока при фактическом удалении.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::states::state::timed_client_state_time;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const WANGSHENG_STATE_ID: u32 = 0x221;
pub(crate) const WANGSHENG_STATE_BYTES: usize = 12;
const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct WangshengState {
    started_at_ms: u32,
    keep_time_ms: u32,
    gain: i32,
}

impl WangshengState {
    pub(crate) const fn new(started_at_ms: u32, keep_time_ms: u32, gain: i32) -> Self {
        Self { started_at_ms, keep_time_ms, gain }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != WANGSHENG_STATE_ID {
            return Err(LegacyReadBlock {
                offset,
                needed: 4,
                available: payload.len().saturating_sub(offset),
            });
        }
        Ok(Self::new(0, reader.read_u32()?, reader.read_i32()?))
    }

    pub(crate) const fn state_id(self) -> u32 { WANGSHENG_STATE_ID }

    pub(crate) const fn activate_loaded(mut self, now_ms: u32) -> Self {
        self.started_at_ms = now_ms;
        self
    }

    pub(crate) const fn expired(self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.keep_time_ms) < now_ms
    }

    pub(crate) fn client_time(self, now_milliseconds: impl FnMut() -> u32) -> u32 {
        timed_client_state_time(self.started_at_ms, self.keep_time_ms, now_milliseconds)
    }

    pub(crate) const fn capped_health(self, current: u32, maximum: u32) -> Option<u32> {
        let actual = current.wrapping_add(self.gain as u32);
        if maximum <= actual { Some(maximum) } else { None }
    }

    pub(crate) fn encoded(
        self,
        now_milliseconds: impl FnMut() -> u32,
    ) -> [u8; WANGSHENG_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(WANGSHENG_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(self.state_id());
        writer.write_u32(self.client_time(now_milliseconds));
        writer.write_i32(self.gain);
        bytes.try_into().expect("размер состояния восстановления фиксирован")
    }
}

pub(crate) fn send_wangsheng_state_visual(
    game: &mut CGame,
    player_id: i32,
    state: WangshengState,
    begin: bool,
    now_milliseconds: impl FnMut() -> u32,
) {
    let Some(player) = game.find_player(player_id) else {
        return;
    };
    let identity = player.shape().identity();
    let mut message = CMessage::new(if begin { STATE_BEGIN_MESSAGE } else { STATE_END_MESSAGE });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.state_id() as i32);
    if begin {
        message.add_long(state.client_time(now_milliseconds) as i32);
        message.add_long(0);
    }
    let _ = game.send_player_shape_around(player_id, None, &message);
}

pub(crate) fn update_wangsheng_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    now_ms: u32,
) -> bool {
    let Some(state) = resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<WangshengState>(key))
        .filter(|state| state.expired(now_ms)).copied()
        else { return false };
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(state.state_id() as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder)
        .and_then(|shape| shape.remove_applied_state_record::<WangshengState>(key, WANGSHENG_STATE_BYTES))
        .is_some();
    if removed && holder.object_type == 400 {
        let _ = game.update_player_properties(holder.id);
    }
    true
}
