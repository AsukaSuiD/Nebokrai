//! Каноническое периодическое состояние `CLeafCutState3` (`0x8F`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/leafcutstate3.cpp`. Класс наследует формулу, два чтения
//! часов и 68-байтовый DB-кодек `CLeafCutState`, но хранится отдельным
//! состоянием и сохраняет собственный ID. Обёртка не создаёт второй источник
//! истины: жизненный цикл принадлежит `CanonicalStateStorage`.

use super::leafcutstate::{LeafCutState, LeafCutStateTick};
use crate::gameserver::appserver::legacycodec::LegacyReadBlock;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const LEAF_CUT_3_STATE_ID: u32 = 0x8f;
pub(crate) const LEAF_CUT_3_STATE_BYTES: usize = 68;
const STATE_BEGIN_MESSAGE: i32 = 0x000b_fe03;
const STATE_END_MESSAGE: i32 = 0x000b_fe04;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LeafCutState3(LeafCutState);

impl LeafCutState3 {
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
            LEAF_CUT_3_STATE_ID,
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

    pub(crate) const fn skill_id(self) -> u32 { LEAF_CUT_3_STATE_ID }
    pub(crate) const fn master(self) -> MasterInfo { self.0.master() }
    pub(crate) const fn client_time(self, now_ms: u32) -> i32 { self.0.client_time(now_ms) }
    pub(crate) const fn serialized_span(self) -> Option<(usize, usize)> { self.0.serialized_span() }
    pub(crate) fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) { self.0.shift_serialized_offset_after(removed_offset, amount); }
    pub(crate) fn activate_loaded(&mut self, now_ms: u32) { self.0.activate_loaded(now_ms); }
    pub(crate) fn decode(payload: &[u8], offset: usize, now_ms: u32) -> Result<Self, LegacyReadBlock> { LeafCutState::decode_with_id(payload, offset, now_ms, LEAF_CUT_3_STATE_ID).map(Self) }
    pub(crate) fn append_serialized(&mut self, payload: &mut Vec<u8>, now_ms: u32) { self.0.append_serialized(payload, now_ms); }
    pub(crate) fn write_serialized_at(&mut self, payload: &mut [u8], offset: usize, now_ms: u32) -> bool { self.0.write_serialized_at(payload, offset, now_ms) }
    pub(crate) fn update_serialized_runtime(self, payload: &mut [u8], now_ms: u32) { self.0.update_serialized_runtime(payload, now_ms); }
    pub(crate) fn tick(&mut self, lifetime_now_ms: u32, frequency_now_ms: u32, target_dead: bool, critical_chance: u16, critical_rate: f32, random: &mut dyn FnMut(i32) -> i32) -> LeafCutStateTick { self.0.tick(lifetime_now_ms, frequency_now_ms, target_dead, critical_chance, critical_rate, random) }
}

pub(crate) fn send_leaf_cut_3_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    tile_x: i32,
    tile_y: i32,
    state: LeafCutState3,
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
