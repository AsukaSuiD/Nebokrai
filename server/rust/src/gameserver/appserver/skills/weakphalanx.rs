//! Область ослабления: геометрия, срок жизни и параметры состояния.
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/weakphalanx.cpp.
//!
//! RU-caller создаёт область 1×1. Исходный обход использует length для обеих
//! осей, хотя смещение Y и состояние используют height. Истечение лишь
//! выбирает ветвь AI: удаление области следует после обоих обходов состояний.
//! Входной снимок содержит ID/уровень навыка, Master и оставшееся время перед
//! CShape; сам владелец и его часы при сериализации не изменяются.

use super::weak::WEAK_SKILL_ID;
use super::weakstate::WeakState;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use crate::gameserver::gameserver::game::CGame;
use nebokrai_shared::values::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum WeakPhalanxTick { Scan, Expired }

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CWeakPhalanx {
    shape: CShape,
    master: MasterInfo,
    started_at_ms: u32,
    lifetime_ms: u32,
    skill_level: i32,
    length: i32,
    height: i32,
    attack_loss: u32,
}

pub(crate) fn weak_cell_targets(game: &CGame, region_id: i32, x: i32, y: i32) -> Vec<ShapeIdentity> {
    super::flash::cell_views(game, region_id, x, y).into_iter().map(|shape| shape.identity).collect()
}

impl CWeakPhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля буквально соответствуют конструктору EXE")]
    pub(crate) fn new(id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32, skill_level: i32, length: i32, height: i32, attack_loss: u32) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self { shape, master, started_at_ms, lifetime_ms, skill_level, length, height, attack_loss }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.master }
    pub(crate) const fn skill_level(&self) -> i32 { self.skill_level }
    pub(crate) const fn attack_loss(&self) -> u32 { self.attack_loss }
    pub(crate) const fn length(&self) -> i32 { self.length }
    pub(crate) const fn height(&self) -> i32 { self.height }
    pub(crate) const fn skill_id(&self) -> u32 { WEAK_SKILL_ID }

    pub(crate) fn set_center(&mut self, x: i32, y: i32) {
        let y = (f64::from(y) + 0.5) as f32;
        let x = (f64::from(x) + 0.5) as f32;
        self.shape.set_pos_xy_base(x, y);
    }

    pub(crate) fn state(&self) -> Option<WeakState> {
        let y = self.shape.get_tile_y().ok()?;
        let x = self.shape.get_tile_x().ok()?;
        Some(WeakState::new(self.attack_loss, x, y, self.length, self.height))
    }

    pub(crate) fn mark_ended(&mut self) {
        self.shape.set_change_state(SHAPE_CHANGE_DELETE);
    }

    pub(crate) fn tick(&mut self, now_ms: u32) -> WeakPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            WeakPhalanxTick::Expired
        } else {
            WeakPhalanxTick::Scan
        }
    }

    pub(crate) fn active_cells(&self) -> Vec<(i32, i32)> {
        let Ok(center_x) = self.shape.get_tile_x() else { return Vec::new() };
        let Ok(center_y) = self.shape.get_tile_y() else { return Vec::new() };
        let start_x = center_x.wrapping_sub(self.length / 2);
        let start_y = center_y.wrapping_sub(self.height / 2);
        let mut cells = Vec::new();
        for x in start_x..start_x.wrapping_add(self.length) {
            for y in start_y..start_y.wrapping_add(self.length) {
                cells.push((x, y));
            }
        }
        cells
    }

    pub(crate) fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, WEAK_SKILL_ID as i32, self.skill_level,
            self.master.master_type, self.master.master_id,
            self.started_at_ms, self.lifetime_ms, now,
        )
    }
}
