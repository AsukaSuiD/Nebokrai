//! Канонический запас метеорных стрел `CMeteorArrowState` (`0xCC`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/meteorarrowstate.cpp`. Состояние хранит текущий и
//! предельный запас, сериализуется тремя little-endian DWORD и при каждом
//! успешном пополнении публикует обновление `0xBFE03`. Удаление всего запаса
//! публикует `0xBFE04`. Сырой `ex_states` остаётся только кодеком вокруг этого
//! единственного типизированного владельца.
//! Vtable 0x00660AC4: End +0x1C→0x005F6AD0 сначала публикует visual
//! phase2 (0x005F6BA0, BF E04 на GetSufferer), затем вызывает CState::End
//! 0x005DBCE0: mark ended и RemoveState у GetUser, не у sufferer.
//! Runtime Begin в MeteorArrowMass0x00588628 получает self,self; загрузочный
//! StartAllStates0x004CE050 передаёт null user. Общая metadata сохраняет
//! различие: отсутствие user не подменяется удалением из holder.
//! Object Begin +0x08→0x005F6B10 не имеет null-guards: base Begin,
//! новый visual(0xC), BeginVisualEffect(1), одноаргументный Update +0x0C
//! 0x005DC1E0 и return 1. Этот Update базовый, при loop=1 пакетов нет;
//! restart не пополняет запас и не сбрасывает timestamp при null user.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state, end_base_applied_state,
    resolve_state_move_shape, update_applied_state_visual_base,
};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const METEOR_ARROW_MASS_SKILL_ID: u32 = 0xcc;
pub(crate) const METEOR_ARROW_STATE_BYTES: usize = 12;

pub(crate) fn restart_meteor_arrow_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<MeteorArrowState>(key)).is_none()
    {
        return false;
    }
    begin_base_applied_state(game, region_id, holder, key)
        && begin_applied_state_visual(game, region_id, holder, key, 1)
        && update_applied_state_visual_base(game, region_id, holder, key)
}

pub(crate) fn end_meteor_arrow_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<MeteorArrowState>(key)).is_none()
    {
        return false;
    }
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(METEOR_ARROW_MASS_SKILL_ID as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    end_base_applied_state(game, region_id, holder, key, METEOR_ARROW_STATE_BYTES)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MeteorArrowState {
    arrows: i32,
    maximum_arrows: u32,
    serialized_offset: Option<usize>,
}

impl MeteorArrowState {
    pub(crate) const fn new(maximum_arrows: u32) -> Self {
        Self { arrows: 0, maximum_arrows, serialized_offset: None }
    }
    pub(crate) const fn skill_id(self) -> u32 { METEOR_ARROW_MASS_SKILL_ID }
    pub(crate) const fn arrows(self) -> i32 { self.arrows }
    pub(crate) const fn additional_data(self) -> i32 { self.arrows }
    pub(crate) const fn maximum_arrows(self) -> u32 { self.maximum_arrows }
    pub(crate) const fn serialized_span(self) -> Option<(usize, usize)> {
        match self.serialized_offset { Some(offset) => Some((offset, METEOR_ARROW_STATE_BYTES)), None => None }
    }
    pub(crate) fn add_arrows(&mut self, amount: u32) -> bool {
        if self.arrows >= self.maximum_arrows as i32 { return false }
        let next = (self.arrows.max(0) as u32).wrapping_add(amount).min(self.maximum_arrows);
        self.arrows = next as i32;
        true
    }
    fn encoded(self) -> [u8; METEOR_ARROW_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(METEOR_ARROW_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(METEOR_ARROW_MASS_SKILL_ID); writer.write_i32(self.arrows); writer.write_u32(self.maximum_arrows);
        bytes.try_into().expect("размер состояния метеорных стрел фиксирован")
    }
    pub(crate) fn decode(payload: &[u8], offset: usize) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != METEOR_ARROW_MASS_SKILL_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self { arrows: reader.read_i32()?, maximum_arrows: reader.read_u32()?, serialized_offset: Some(offset) })
    }
    pub(crate) fn append_serialized(&mut self, payload: &mut Vec<u8>) {
        let offset = payload.len(); payload.extend_from_slice(&self.encoded()); self.serialized_offset = Some(offset);
    }
    pub(crate) fn write_serialized_at(&mut self, payload: &mut [u8], offset: usize) -> bool {
        let Some(destination) = payload.get_mut(offset..offset.saturating_add(METEOR_ARROW_STATE_BYTES)) else { return false };
        destination.copy_from_slice(&self.encoded()); self.serialized_offset = Some(offset); true
    }
    pub(crate) fn update_serialized(self, payload: &mut [u8]) {
        if let Some(offset) = self.serialized_offset { let _ = LegacyWriter::write_i32_at(payload, offset + 4, self.arrows); }
    }
    pub(crate) fn shift_serialized_offset_for_insert(&mut self, inserted_offset: usize, amount: usize) {
        if let Some(offset) = &mut self.serialized_offset {
            if *offset >= inserted_offset {
                *offset += amount;
            }
        }
    }

    pub(crate) fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) {
        if self.serialized_offset.is_some_and(|offset| removed_offset < offset) {
            self.serialized_offset = self.serialized_offset.map(|offset| offset - amount);
        }
    }
}
