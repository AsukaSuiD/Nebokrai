//! Область ядовитого тумана `CPoisonFogPhalanx` (`0xC9`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/poisonfogphalanx.cpp`. В RU-варианте все достигнутые
//! уровни используют одну клетку. Каждый проход `AI` заново заменяет состояние
//! подходящих целей; `CGame` сохраняет порядок `GetShape`, правила PK и
//! координацию заимствований.

use super::poisonfogstate::PoisonFogState;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PoisonFogPhalanxTick { Scan, Expired }

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CPoisonFogPhalanx {
    shape: CShape, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
    skill_level: i32, state_keep_time_ms: u32, defense_loss: u32,
    defense_loss_coefficient: u32, dodge_loss: u32, element_resistance_loss: u32,
    element_resistance_loss_coefficient: u32, weapon_damage_level: u32,
    scope_active: bool,
}

pub(crate) fn poison_fog_targets(game: &CGame, region_id: i32, phalanx: &CPoisonFogPhalanx) -> Vec<ShapeIdentity> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() };
    let (Ok(x), Ok(y)) = (phalanx.shape().get_tile_x(), phalanx.shape().get_tile_y()) else { return Vec::new() };
    let (width, height) = game.area_dimensions();
    let mut shapes = Vec::new();
    if region.get_shapes(x, y, width, height, game, &mut shapes).is_err() { return Vec::new() }
    shapes.into_iter().map(|shape| shape.identity).filter(|identity| {
        *identity != phalanx.shape().identity()
            && !(identity.object_type == phalanx.master().master_type && identity.id == phalanx.master().master_id)
            && matches!(identity.object_type, 400 | 600)
    }).collect()
}

impl CPoisonFogPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32, skill_level: i32, state_keep_time_ms: u32, defense_loss: u32, defense_loss_coefficient: u32, dodge_loss: u32, element_resistance_loss: u32, element_resistance_loss_coefficient: u32, weapon_damage_level: u32) -> Self { let mut shape = CShape::with_constructor_defaults(); shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID }); Self { shape, master, started_at_ms, lifetime_ms, skill_level, state_keep_time_ms, defense_loss, defense_loss_coefficient, dodge_loss, element_resistance_loss, element_resistance_loss_coefficient, weapon_damage_level, scope_active: true } }
    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) fn replace_affect_region(&mut self, tile_x: i32, tile_y: i32) { if self.shape.get_tile_x() == Ok(tile_x) && self.shape.get_tile_y() == Ok(tile_y) { self.scope_active = false; } }
    pub(crate) fn tick(&mut self, now_ms: u32) -> PoisonFogPhalanxTick { if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms || !self.scope_active { self.shape.set_change_state(SHAPE_CHANGE_DELETE); PoisonFogPhalanxTick::Expired } else { PoisonFogPhalanxTick::Scan } }
    pub(crate) fn state(&self, now_ms: u32) -> PoisonFogState { PoisonFogState::new(self.skill_level, now_ms, self.state_keep_time_ms, self.defense_loss, self.defense_loss_coefficient, self.dodge_loss, self.element_resistance_loss, self.element_resistance_loss_coefficient, self.weapon_damage_level) }
    pub(crate) fn encode_client_snapshot(&self) -> Option<Vec<u8>> { let mut payload = Vec::new(); self.shape.add_to_byte_array(&mut payload, true).then_some(payload) }
}
