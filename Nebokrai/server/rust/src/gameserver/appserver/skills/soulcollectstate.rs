//! Каноническое состояние сбора душ `CSoulCollectState` (`0x13B`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/soulcollectstate.cpp`. Состояние без таймера хранит
//! коэффициент и не более `skill_level` душ. Каждое успешное пополнение
//! публикует окончание прежнего снимка до нового снимка. Удаление публикует
//! только окончание. Три исходные перегрузки `Begin` имели одинаковые
//! последствия привязки и визуального обновления и сведены к созданию
//! канонического состояния. Exact persisted-запись содержит ID, skill level
//! и число душ, но теряет `variable_percent`; после загрузки он нулевой.

//! Vtable 0x0065EFE4: End +0x1C→0x005E1D20 публикует visual phase2,
//! пишет IsEnded=1, затем GetSufferer и RemoveState. AI при этом пустой;
//! это не отменяет прямой End. Payload остаётся живым до доставки visual.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::states::state::{resolve_state_move_shape, resolve_state_move_shape_mut};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const SOUL_COLLECT_STATE_ID: u32 = 0x13b;
pub(crate) const SOUL_COLLECT_STATE_BYTES: usize = 12;

pub(crate) fn end_soul_collect_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<SoulCollectState>(key)).is_none()
    {
        return false;
    }
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(SOUL_COLLECT_STATE_ID as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder).and_then(|shape| {
        shape.applied_state::<SoulCollectState>(key)?;
        let _ = shape.mark_applied_state_ended(key);
        shape.remove_applied_state_record::<SoulCollectState>(key, SOUL_COLLECT_STATE_BYTES)
    }).is_some();
    if removed && holder.object_type == 400 {
        let _ = game.update_player_properties(holder.id);
    }
    removed
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct SoulCollectState {
    skill_level: i32,
    variable_percent: u32,
    souls: i32,
}

impl SoulCollectState {
    pub(crate) const fn new(skill_level: i32, variable_percent: u32) -> Self {
        Self { skill_level, variable_percent, souls: 0 }
    }

    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != SOUL_COLLECT_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self { skill_level: reader.read_i32()?, variable_percent: 0, souls: reader.read_i32()? })
    }

    pub(crate) fn encoded(self) -> [u8; SOUL_COLLECT_STATE_BYTES] {
        let mut bytes = [0; SOUL_COLLECT_STATE_BYTES];
        for (index, value) in [SOUL_COLLECT_STATE_ID as i32, self.skill_level, self.souls].into_iter().enumerate() {
            bytes[index * 4..index * 4 + 4].copy_from_slice(&value.to_le_bytes());
        }
        bytes
    }

    pub(crate) const fn skill_id(self) -> u32 { SOUL_COLLECT_STATE_ID }
    pub(crate) const fn variable_percent(self) -> u32 { self.variable_percent }
    pub(crate) const fn souls(self) -> i32 { self.souls }

    /// Возвращает `true` только когда исходный `AddSoul` действительно меняет
    /// состояние и поэтому требует пары визуальных событий `2 -> 1`.
    pub(crate) fn add_soul(&mut self) -> bool {
        if self.souls >= self.skill_level { return false; }
        self.souls = self.souls.wrapping_add(1);
        true
    }
}

pub(crate) fn send_soul_collect_state_visual(
    game: &mut CGame,
    region_id: i32,
    target: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: SoulCollectState,
    begin: bool,
) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(target.object_type);
    message.add_long(target.id);
    message.add_long(SOUL_COLLECT_STATE_ID as i32);
    if begin {
        message.add_long(state.variable_percent() as i32);
        message.add_long(state.souls());
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}
