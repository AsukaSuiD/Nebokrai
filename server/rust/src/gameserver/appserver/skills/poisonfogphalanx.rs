//! Адаптер области ядовитого тумана к CShape и живому региону.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/poisonfogphalanx.cpp/.h.
//! Время, активность клетки и шаблон состояния находятся в zone/skills/poisonfog.rs.
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
use nebokrai_shared::values::CGuid;
use nebokrai_zone::skills::PoisonFogPhalanx;

pub(crate) use nebokrai_zone::skills::PoisonFogPhalanxTick;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CPoisonFogPhalanx {
    shape: CShape,
    rule: PoisonFogPhalanx,
}

pub(crate) fn poison_fog_targets(game: &CGame, region_id: i32, phalanx: &CPoisonFogPhalanx) -> Vec<ShapeIdentity> {
    if !phalanx.rule.cell_active() { return Vec::new(); }
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
        let state = PoisonFogState::new(
            skill_level, state_keep_time_ms, defense_loss, defense_loss_coefficient,
            dodge_loss, element_resistance_loss, element_resistance_loss_coefficient,
            weapon_damage_level,
        );
        Self { shape, rule: PoisonFogPhalanx::new(master, started_at_ms, lifetime_ms, state) }
    }
    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.rule.master() }
    pub(crate) const fn skill_level(&self) -> i32 { self.rule.skill_level() }
    pub(crate) fn set_center(&mut self, x: i32, y: i32) {
        let y = (f64::from(y) + 0.5) as f32;
        let x = (f64::from(x) + 0.5) as f32;
        self.shape.set_pos_xy_base(x, y);
    }

    pub(crate) fn replace_affect_region(&mut self, _level: i32, tile_x: i32, tile_y: i32) {
        if self.shape.get_tile_x() == Ok(tile_x) && self.shape.get_tile_y() == Ok(tile_y) {
            self.rule.clear_cell();
        }
    }

    pub(crate) fn tick(&mut self, now_ms: u32) -> PoisonFogPhalanxTick {
        let tick = self.rule.tick(now_ms);
        if tick == PoisonFogPhalanxTick::Expired {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
        }
        tick
    }

    pub(crate) fn state(&self) -> PoisonFogState {
        self.rule.state()
    }

    pub(crate) fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, POISON_FOG_STATE_ID as i32, self.rule.skill_level(),
            self.rule.master().master_type, self.rule.master().master_id,
            self.rule.started_at_ms(), self.rule.lifetime_ms(), now,
        )
    }
}
