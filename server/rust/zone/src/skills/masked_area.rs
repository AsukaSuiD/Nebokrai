//! Маска и живая форма неподвижных областей FireWall и YinYang.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! appserver/skills/firewallphalanx.cpp и yinyangphalanx{,2}.cpp.
//! Опорные адреса ReplaceAffectRegion/SetInScope/AI —
//! docs/reconstruction/gameserver-skills.md#zonalcast-скелет-областных-призывов.
//! Композит `MaskedElementPhalanx` (CShape + снимок атаки + маска) перенесён
//! из старого адаптера буквально; новых машинных оснований он не добавляет.

use nebokrai_shared::values::CGuid;
use crate::combat::MasterInfo;
use crate::regions::ShapeIdentity;
use crate::regions::shape::CShape;
use super::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use super::ElementPhalanxAttack;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MaskedAreaPulse {
    Once,
    Periodic { frequency_ms: u32, last_attack_ms: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaskedArea {
    started_at_ms: u32,
    lifetime_ms: u32,
    pulse: MaskedAreaPulse,
    length: i32,
    height: i32,
    cells: Vec<bool>,
}

impl MaskedArea {
    pub fn new(
        started_at_ms: u32, lifetime_ms: u32, pulse: MaskedAreaPulse,
        (length, height, mask): (i32, i32, &[bool]),
    ) -> Self {
        Self { started_at_ms, lifetime_ms, pulse, length, height, cells: mask.to_vec() }
    }

    pub const fn started_at_ms(&self) -> u32 { self.started_at_ms }
    pub const fn lifetime_ms(&self) -> u32 { self.lifetime_ms }
    pub const fn dimensions(&self) -> (i32, i32) { (self.length, self.height) }
    pub const fn is_periodic(&self) -> bool {
        matches!(self.pulse, MaskedAreaPulse::Periodic { .. })
    }
    pub const fn expired_at(&self, now: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now
    }
    pub const fn attack_due_at(&self, now: u32) -> bool {
        match self.pulse {
            MaskedAreaPulse::Once => self.expired_at(now),
            MaskedAreaPulse::Periodic { frequency_ms, last_attack_ms } =>
                frequency_ms.wrapping_add(last_attack_ms) < now,
        }
    }
    pub fn mark_attack_at(&mut self, now: u32) {
        if let MaskedAreaPulse::Periodic { last_attack_ms, .. } = &mut self.pulse {
            *last_attack_ms = now;
        }
    }

    pub fn origin(&self, (center_x, center_y): (i32, i32)) -> (i32, i32) {
        (center_x.wrapping_sub(self.length >> 1), center_y.wrapping_sub(self.height >> 1))
    }

    pub fn replace_affect_region(
        &mut self, center: (i32, i32), incoming_center: (i32, i32),
        (new_length, new_height, incoming_mask): (i32, i32, &[bool]),
    ) {
        let (left, top) = self.origin(center);
        let new_left = incoming_center.0.wrapping_sub(new_length >> 1);
        let new_top = incoming_center.1.wrapping_sub(new_height >> 1);
        for x in 0..self.length {
            for y in 0..self.height {
                let new_x = left.wrapping_add(x).wrapping_sub(new_left);
                let new_y = top.wrapping_add(y).wrapping_sub(new_top);
                if new_x < 0 || new_y < 0 || new_x >= new_length || new_y >= new_height { continue; }
                if incoming_mask.get(new_y.wrapping_mul(new_length).wrapping_add(new_x) as usize)
                    .copied().unwrap_or(false)
                {
                    // Native передаёт смещения в SetInScope в переставленном порядке.
                    let index = x.wrapping_mul(self.length).wrapping_add(y) as usize;
                    if let Some(cell) = self.cells.get_mut(index) { *cell = false; }
                }
            }
        }
    }

    pub fn cell_active(&self, x: i32, y: i32) -> bool {
        self.cells.get(y.wrapping_mul(self.length).wrapping_add(x) as usize)
            .copied().unwrap_or(false)
    }
}

/// Живая форма неподвижных масочных областей огненной стены и инь-ян:
/// связка CShape, снимка атаки и маски. Стена повторяет окно по трём чтениям
/// часов, инь-ян поражает при истечении срока и завершается после обхода.
/// Replace транспонирует координаты записи, AI читает обычный X/Y.
/// Состав и порядок соответствуют адаптеру старого пакета.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MaskedElementPhalanx {
    shape: CShape,
    attack: ElementPhalanxAttack,
    scope: MaskedArea,
}

impl MaskedElementPhalanx {
    pub fn new(
        id: i32, attack: ElementPhalanxAttack, started_at_ms: u32, lifetime_ms: u32,
        pulse: MaskedAreaPulse, mask: (i32, i32, &[bool]),
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self { shape, attack,
            scope: MaskedArea::new(started_at_ms, lifetime_ms, pulse, mask) }
    }

    pub const fn shape(&self) -> &CShape { &self.shape }
    pub const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub const fn master(&self) -> MasterInfo { self.attack.master }
    pub const fn skill_id(&self) -> u32 { self.attack.skill_id }
    pub const fn attack_snapshot(&self) -> ElementPhalanxAttack { self.attack }
    pub const fn dimensions(&self) -> (i32, i32) { self.scope.dimensions() }
    pub const fn is_periodic(&self) -> bool { self.scope.is_periodic() }
    pub const fn expired_at(&self, now: u32) -> bool {
        self.scope.expired_at(now)
    }
    pub const fn attack_due_at(&self, now: u32) -> bool {
        self.scope.attack_due_at(now)
    }
    pub fn mark_attack_at(&mut self, now: u32) {
        self.scope.mark_attack_at(now);
    }

    pub fn origin(&self) -> (i32, i32) {
        let x = self.shape.get_tile_x().unwrap_or(i32::MIN);
        let y = self.shape.get_tile_y().unwrap_or(i32::MIN);
        self.scope.origin((x, y))
    }

    pub fn replace_affect_region(&mut self, level: i32, tile_x: i32, tile_y: i32) {
        let incoming = if self.is_periodic() {
            super::firewall::fire_wall_scope(level)
        } else { super::yinyang::yin_yang_scope(self.skill_id()) };
        let center = (self.shape.get_tile_x().unwrap_or(i32::MIN),
            self.shape.get_tile_y().unwrap_or(i32::MIN));
        self.scope.replace_affect_region(center, (tile_x, tile_y), incoming);
    }

    pub fn cell_active(&self, x: i32, y: i32) -> bool {
        self.scope.cell_active(x, y)
    }

    pub fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, self.skill_id() as i32, self.attack.skill_level,
            self.master().master_type, self.master().master_id,
            self.scope.started_at_ms(), self.scope.lifetime_ms(), now,
        )
    }
}
