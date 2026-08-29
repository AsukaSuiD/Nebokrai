//! Каноническое накопительное состояние `CEnergyHoldingState` (`0x89`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/energyholdingstate.cpp`. Число зарядов увеличивается не
//! выше уровня навыка; каждое успешное увеличение публикует `End → Begin`.
//! Параметр процента хранится вместе с зарядом и применяется
//! `CInverseChopped` при первом расчёте атаки. Создание, увеличение и снятие
//! канонического player-state выполняются этим owner-ом.
//!
//! DB-запись содержит ID, уровень навыка и число зарядов. Не сохраняемый
//! процент восстанавливается из `CSkillFactory` usage `20020` того же уровня.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const ENERGY_HOLDING_STATE_ID: u32 = 0x89;
pub(crate) const ENERGY_HOLDING_STATE_BYTES: usize = 12;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct EnergyHoldingState {
    skill_level: u32,
    energy_count: u32,
    parameter_percent: u32,
}

impl EnergyHoldingState {
    pub(crate) const fn new(skill_level: u32, parameter_percent: u32) -> Self {
        Self { skill_level, energy_count: 0, parameter_percent }
    }

    pub(crate) const fn skill_id(self) -> u32 { ENERGY_HOLDING_STATE_ID }
    pub(crate) const fn energy_count(self) -> u32 { self.energy_count }
    pub(crate) const fn parameter_percent(self) -> u32 { self.parameter_percent }

    pub(crate) const fn add_energy(&mut self) -> bool {
        if self.energy_count >= self.skill_level { return false }
        self.energy_count = self.energy_count.wrapping_add(1);
        true
    }

    pub(crate) fn decode(
        payload: &[u8],
        offset: usize,
        parameter_percent: u32,
    ) -> Result<Self, LegacyReadBlock> {
        let mut reader = LegacyReader::at(payload, offset)?;
        if reader.read_u32()? != ENERGY_HOLDING_STATE_ID {
            return Err(LegacyReadBlock { offset, needed: 4, available: payload.len().saturating_sub(offset) });
        }
        Ok(Self {
            skill_level: reader.read_u32()?,
            energy_count: reader.read_u32()?,
            parameter_percent,
        })
    }

    pub(crate) fn encoded(self) -> [u8; ENERGY_HOLDING_STATE_BYTES] {
        let mut bytes = Vec::with_capacity(ENERGY_HOLDING_STATE_BYTES);
        let mut writer = LegacyWriter::new(&mut bytes);
        writer.write_u32(ENERGY_HOLDING_STATE_ID);
        writer.write_u32(self.skill_level);
        writer.write_u32(self.energy_count);
        bytes.try_into().expect("размер состояния накопления энергии фиксирован")
    }
}

pub(crate) fn send_energy_holding_state_visual(
    game: &mut CGame,
    region_id: i32,
    identity: ShapeIdentity,
    x: i32,
    y: i32,
    state: EnergyHoldingState,
    begin: bool,
) {
    let mut message = CMessage::new(if begin { 0x000b_fe03 } else { 0x000b_fe04 });
    message.add_long(identity.object_type);
    message.add_long(identity.id);
    message.add_long(ENERGY_HOLDING_STATE_ID as i32);
    if begin {
        message.add_long(0);
        message.add_long(state.parameter_percent() as i32);
    }
    let _ = game.send_shape_position_around(region_id, x, y, &message);
}

pub(crate) fn add_player_energy_holding(game: &mut CGame, player_id: i32, skill_level: u32, parameter_percent: u32) -> bool {
    let installed = game.find_player_mut(player_id).and_then(|player| {
        let region_id = player.server_region_id()?; let identity = player.shape().identity(); let x = player.shape().get_tile_x().ok()?; let y = player.shape().get_tile_y().ok()?;
        if player.energy_holding_state().is_none() { player.begin_energy_holding_state(EnergyHoldingState::new(skill_level, parameter_percent)); }
        let state = player.energy_holding_state_mut().expect("состояние создано выше или существовало");
        Some(state.add_energy().then_some((region_id, identity, x, y, *state)))
    });
    match installed { None => false, Some(None) => true, Some(Some((region_id, identity, x, y, state))) => { send_energy_holding_state_visual(game, region_id, identity, x, y, state, false); send_energy_holding_state_visual(game, region_id, identity, x, y, state, true); true } }
}

pub(crate) fn take_player_energy_holding(game: &mut CGame, player_id: i32) -> Option<EnergyHoldingState> {
    let ended = game.find_player_mut(player_id).and_then(|player| { let region_id = player.server_region_id()?; let identity = player.shape().identity(); let x = player.shape().get_tile_x().ok()?; let y = player.shape().get_tile_y().ok()?; Some((region_id, identity, x, y, player.take_energy_holding_state()?)) });
    let (region_id, identity, x, y, state) = ended?; send_energy_holding_state_visual(game, region_id, identity, x, y, state, false); Some(state)
}
