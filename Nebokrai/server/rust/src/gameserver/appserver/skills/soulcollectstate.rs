//! Каноническое состояние сбора душ `CSoulCollectState` (`0x13B`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/soulcollectstate.cpp`. Состояние без таймера хранит
//! коэффициент и не более `skill_level` душ. Каждое успешное пополнение
//! публикует окончание прежнего снимка до нового снимка. Удаление публикует
//! только окончание. Три исходные перегрузки `Begin` имели одинаковые
//! последствия привязки и визуального обновления и сведены к созданию
//! канонического состояния.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const SOUL_COLLECT_STATE_ID: u32 = 0x13b;

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
