//! Каноническое периодическое состояние `CLeafCutState2` (`0x80`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/leafcutstate2.cpp`. Формула, два чтения часов и два
//! вызова MSVCRT RNG совпадают с подтверждённой основой `CLeafCutState`, но
//! состояние имеет отдельную идентичность и lifecycle. В этом классе не
//! подтверждён собственный DB-кодек, поэтому runtime-состояние не выдумывает
//! сериализацию и хранится отдельным каноническим slot-ом.

use super::leafcutstate::{LeafCutState, LeafCutStateTick};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const LEAF_CUT_2_STATE_ID: u32 = 0x80;
const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeafCutState2(LeafCutState);

impl LeafCutState2 {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют состоянию EXE")]
    pub(crate) const fn new(
        master: MasterInfo,
        started_at_ms: u32,
        keep_time_ms: u32,
        frequency_ms: u32,
        damage_factor: f32,
        damage_modifier: f32,
        minimum_attack: u16,
        maximum_attack: u16,
        element_attack: u16,
        soul_attack: u16,
    ) -> Self {
        Self(LeafCutState::new_with_id(
            LEAF_CUT_2_STATE_ID,
            master,
            started_at_ms,
            keep_time_ms,
            frequency_ms,
            damage_factor,
            damage_modifier,
            minimum_attack,
            maximum_attack,
            element_attack,
            soul_attack,
        ))
    }

    pub(crate) const fn skill_id(self) -> u32 { LEAF_CUT_2_STATE_ID }
    pub(crate) const fn master(self) -> MasterInfo { self.0.master() }
    pub(crate) const fn client_time(self, now_ms: u32) -> i32 { self.0.client_time(now_ms) }
    pub(crate) fn tick(
        &mut self,
        lifetime_now_ms: u32,
        frequency_now_ms: u32,
        target_dead: bool,
        critical_chance: u16,
        critical_rate: f32,
        random: &mut dyn FnMut(i32) -> i32,
    ) -> LeafCutStateTick {
        self.0.tick(
            lifetime_now_ms,
            frequency_now_ms,
            target_dead,
            critical_chance,
            critical_rate,
            random,
        )
    }
}

pub(crate) fn send_leaf_cut_2_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: LeafCutState2,
    begin: bool,
    now_ms: u32,
) {
    let mut message = CMessage::new(if begin { STATE_BEGIN_MESSAGE } else { STATE_END_MESSAGE });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(state.skill_id() as i32);
    if begin {
        message.add_long(state.client_time(now_ms));
        message.add_long(0);
    }
    let _ = game.send_shape_position_around(region_id, tile_x, tile_y, &message);
}
