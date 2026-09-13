//! Область ядовитого тумана: маска покрытия и параметры накладываемого состояния.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/poisonfogphalanx.cpp.
//!
//! Все маски RU-варианта равны одной активной клетке, поэтому вместо CScope
//! хранится один бит. Перекрытие очищает маску, но не сокращает срок жизни.
//! Каждый AI получает отдельный список фигур клетки; допуск, замена состояний
//! и callbacks выполняются по этому порядку при опубликованном владельце.

use super::poisonfogstate::{PoisonFogState, POISON_FOG_STATE_ID};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PoisonFogPhalanxTick {
    Scan,
    Expired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CPoisonFogPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    state_keep_time_ms: u32,
    defense_loss: u32,
    defense_loss_coefficient: u32,
    dodge_loss: u32,
    element_resistance_loss: u32,
    element_resistance_loss_coefficient: u32,
    weapon_damage_level: u32,
    cell_active: bool,
}

pub(crate) fn poison_fog_targets(game: &CGame, region_id: i32, phalanx: &CPoisonFogPhalanx) -> Vec<ShapeIdentity> {
    if !phalanx.cell_active { return Vec::new(); }
    let (Ok(x), Ok(y)) = (phalanx.shape().get_tile_x(), phalanx.shape().get_tile_y()) else { return Vec::new() };
    super::flash::cell_views(game, region_id, x, y).into_iter().map(|shape| shape.identity).collect()
}

impl CPoisonFogPhalanx {
    #[allow(clippy::too_many_arguments, reason = "независимые параметры области и ослабления цели")]
    pub(crate) fn new(
        id: i32,
        master: MasterInfo,
        started_at_ms: u32,
        lifetime_ms: u32,
        skill_level: i32,
        state_keep_time_ms: u32,
        defense_loss: u32,
        defense_loss_coefficient: u32,
        dodge_loss: u32,
        element_resistance_loss: u32,
        element_resistance_loss_coefficient: u32,
        weapon_damage_level: u32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self {
            shape, master, started_at_ms, lifetime_ms, skill_level, state_keep_time_ms,
            defense_loss, defense_loss_coefficient, dodge_loss, element_resistance_loss,
            element_resistance_loss_coefficient, weapon_damage_level, cell_active: true,
        }
    }
    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn skill_level(&self) -> i32 { self.skill_level }
    pub(crate) fn set_center(&mut self, x: i32, y: i32) {
        let y = (f64::from(y) + 0.5) as f32;
        let x = (f64::from(x) + 0.5) as f32;
        self.shape.set_pos_xy_base(x, y);
    }

    pub(crate) fn replace_affect_region(&mut self, _level: i32, tile_x: i32, tile_y: i32) {
        if self.shape.get_tile_x() == Ok(tile_x) && self.shape.get_tile_y() == Ok(tile_y) {
            self.cell_active = false;
        }
    }

    pub(crate) fn tick(&mut self, now_ms: u32) -> PoisonFogPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
            PoisonFogPhalanxTick::Expired
        } else {
            PoisonFogPhalanxTick::Scan
        }
    }

    pub(crate) fn state(&self) -> PoisonFogState {
        PoisonFogState::new(
            self.skill_level, self.state_keep_time_ms, self.defense_loss,
            self.defense_loss_coefficient, self.dodge_loss, self.element_resistance_loss,
            self.element_resistance_loss_coefficient, self.weapon_damage_level,
        )
    }

    pub(crate) fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, POISON_FOG_STATE_ID as i32, self.skill_level,
            self.master.master_type, self.master.master_id,
            self.started_at_ms, self.lifetime_ms, now,
        )
    }
}
