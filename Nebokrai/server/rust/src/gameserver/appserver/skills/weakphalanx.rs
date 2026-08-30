//! Область ослабления `CWeakPhalanx` (`0x12E`).
//!
//! Источник: `gameserver.exe` + `GameServer.pdb`, исходный владелец
//! `appserver/skills/weakphalanx.cpp`. Владелец хранит прямоугольник 1×1,
//! снижение атаки и строгий wrapping-срок жизни. Обход клеток сохраняет
//! исходный порядок X→Y. Разрешение форм и применение состояния остаются у
//! `CGame`, поскольку игроки и монстры принадлежат разным runtime-owner-ам.

use super::weak::WEAK_SKILL_ID;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, SHAPE_CHANGE_DELETE, ShapeIdentity};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use crate::gameserver::gameserver::game::CGame;
use crate::public::guid::CGuid;

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

pub(crate) fn weak_targets(game: &CGame, region_id: i32, phalanx: &CWeakPhalanx) -> Vec<ShapeIdentity> {
    let Some(region) = game.find_region(region_id).map(|owner| owner.base()) else { return Vec::new() };
    let (width, height) = game.area_dimensions();
    let mut targets = Vec::new();
    for (tile_x, tile_y) in phalanx.active_cells() {
        let mut shapes = Vec::new();
        if region.get_shapes(tile_x, tile_y, width, height, game, &mut shapes).is_err() { break }
        for shape in shapes {
            if shape.identity == phalanx.shape().identity()
                || (shape.identity.object_type == phalanx.master().master_type && shape.identity.id == phalanx.master().master_id)
                || !matches!(shape.identity.object_type, 400 | 600)
            { continue }
            targets.push(shape.identity);
        }
    }
    targets
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

    pub(crate) fn tick(&mut self, now_ms: u32) -> WeakPhalanxTick {
        if self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms {
            self.shape.set_change_state(SHAPE_CHANGE_DELETE);
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
        for x in 0..self.length.max(0) {
            for y in 0..self.height.max(0) {
                cells.push((start_x.wrapping_add(x), start_y.wrapping_add(y)));
            }
        }
        cells
    }

    pub(crate) fn encode_client_snapshot(&self) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}
