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

//! Vtable 0x0065FDA4: End +0x1C→0x005E1D20 публикует visual phase2,
//! пишет IsEnded=1, затем GetSufferer и RemoveState. AI при этом пустой;
//! это не отменяет прямой End. Payload остаётся живым до доставки visual.
//! Object Begin +0x08→0x005EC610: без null-guards вызывает base Begin,
//! создаёт visual(0xC) и BeginVisualEffect(1), затем возвращает 1. Update
//! не вызывается: restart не публикует пакет и не меняет число зарядов.

use crate::gameserver::appserver::moveshape::StateKey;
use crate::gameserver::appserver::states::state::{
    begin_applied_state_visual, begin_base_applied_state,
    resolve_state_move_shape, resolve_state_move_shape_mut,
};
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::legacycodec::{LegacyReadBlock, LegacyReader, LegacyWriter};
use crate::gameserver::gameserver::game::CGame;
use crate::nets::netserver::message::CMessage;

pub(crate) const ENERGY_HOLDING_STATE_ID: u32 = 0x89;
pub(crate) const ENERGY_HOLDING_STATE_BYTES: usize = 12;

pub(crate) fn restart_energy_holding_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
    _changing_region: bool,
    _now: &mut dyn FnMut() -> u32,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<EnergyHoldingState>(key)).is_none()
    {
        return false;
    }
    begin_base_applied_state(game, region_id, holder, key)
        && begin_applied_state_visual(game, region_id, holder, key, 1)
}

pub(crate) fn end_energy_holding_state(
    game: &mut CGame,
    region_id: i32,
    holder: ShapeIdentity,
    key: StateKey,
) -> bool {
    if resolve_state_move_shape(game, region_id, holder)
        .and_then(|shape| shape.applied_state::<EnergyHoldingState>(key)).is_none()
    {
        return false;
    }
    let mut message = CMessage::new(0x000b_fe04);
    message.add_long(holder.object_type);
    message.add_long(holder.id);
    message.add_long(ENERGY_HOLDING_STATE_ID as i32);
    let _ = game.send_move_shape_around(region_id, holder, &message);
    let removed = resolve_state_move_shape_mut(game, region_id, holder).and_then(|shape| {
        shape.applied_state::<EnergyHoldingState>(key)?;
        let _ = shape.mark_applied_state_ended(key);
        shape.remove_applied_state_record::<EnergyHoldingState>(key, ENERGY_HOLDING_STATE_BYTES)
    }).is_some();
    if removed {
        let _ = game.update_move_shape_properties(region_id, holder);
    }
    removed
}

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
    pub(crate) const fn skill_level(self) -> u32 { self.skill_level }
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
