//! Канонический запас метеорных стрел `CMeteorArrowState` (`0xCC`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/meteorarrowstate.cpp`. Состояние хранит текущий и
//! предельный запас, сериализуется тремя little-endian DWORD и при каждом
//! успешном пополнении публикует обновление `0xBFE03`. Удаление всего запаса
//! публикует `0xBFE04`. Сырой `ex_states` остаётся только кодеком вокруг этого
//! единственного типизированного владельца.

use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};

pub(crate) const METEOR_ARROW_MASS_SKILL_ID: u32 = 0xcc;
pub(crate) const METEOR_ARROW_STATE_BYTES: usize = 12;

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
    pub(crate) fn shift_serialized_offset_after(&mut self, removed_offset: usize, amount: usize) {
        if self.serialized_offset.is_some_and(|offset| removed_offset < offset) {
            self.serialized_offset = self.serialized_offset.map(|offset| offset - amount);
        }
    }
}
