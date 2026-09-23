//! Неподвижные масочные области огненной стены и инь-ян.
//! Источник: gameserver.exe/GameServer.pdb, firewallphalanx.cpp и
//! yinyangphalanx{,2}.cpp. У владельца одна маска Vec и один снимок атаки;
//! каждый экземпляр независимо хранится в региональной арене.
//! Стена повторяет окно по трём чтениям часов, инь-ян поражает при истечении
//! срока и завершается после обхода. Пустая маска сама по себе не означает End.
//! Replace транспонирует координаты записи, AI читает обычный X/Y.
//! У стены отклонённая допуском цель тоже входит в дедупликацию окна.

use super::elementphalanxattack::ElementPhalanxAttack;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use nebokrai_shared::values::CGuid;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum MaskedAreaPulse {
    Once,
    Periodic { frequency_ms: u32, last_attack_ms: u32 },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MaskedElementPhalanx {
    shape: CShape,
    attack: ElementPhalanxAttack,
    started_at_ms: u32,
    lifetime_ms: u32,
    pulse: MaskedAreaPulse,
    length: i32,
    height: i32,
    scope: Vec<bool>,
}

impl MaskedElementPhalanx {
    pub(super) fn new(
        id: i32, attack: ElementPhalanxAttack, started_at_ms: u32, lifetime_ms: u32,
        pulse: MaskedAreaPulse, mask: (i32, i32, &[bool]),
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self { shape, attack, started_at_ms, lifetime_ms, pulse,
            length: mask.0, height: mask.1, scope: mask.2.to_vec() }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.master }
    pub(crate) const fn skill_id(&self) -> u32 { self.attack.skill_id }
    pub(crate) const fn attack_snapshot(&self) -> ElementPhalanxAttack { self.attack }
    pub(crate) const fn dimensions(&self) -> (i32, i32) { (self.length, self.height) }
    pub(crate) const fn is_periodic(&self) -> bool { matches!(self.pulse, MaskedAreaPulse::Periodic { .. }) }
    pub(crate) const fn expired_at(&self, now: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now
    }
    pub(crate) const fn attack_due_at(&self, now: u32) -> bool {
        match self.pulse {
            MaskedAreaPulse::Once => self.expired_at(now),
            MaskedAreaPulse::Periodic { frequency_ms, last_attack_ms } => frequency_ms.wrapping_add(last_attack_ms) < now,
        }
    }
    pub(crate) fn mark_attack_at(&mut self, now: u32) {
        if let MaskedAreaPulse::Periodic { last_attack_ms, .. } = &mut self.pulse { *last_attack_ms = now; }
    }

    pub(crate) fn origin(&self) -> (i32, i32) {
        let x = self.shape.get_tile_x().unwrap_or(i32::MIN);
        let y = self.shape.get_tile_y().unwrap_or(i32::MIN);
        (x.wrapping_sub(self.length >> 1), y.wrapping_sub(self.height >> 1))
    }

    pub(crate) fn replace_affect_region(&mut self, level: i32, tile_x: i32, tile_y: i32) {
        let (left, top) = self.origin();
        let (new_length, new_height, mask) = if self.is_periodic() {
            nebokrai_zone::skills::fire_wall_scope(level)
        } else { super::yinyangphalanx::scope_for_skill(self.skill_id()) };
        let new_left = tile_x.wrapping_sub(new_length >> 1);
        let new_top = tile_y.wrapping_sub(new_height >> 1);
        for x in 0..self.length {
            for y in 0..self.height {
                let new_x = left.wrapping_add(x).wrapping_sub(new_left);
                let new_y = top.wrapping_add(y).wrapping_sub(new_top);
                if new_x < 0 || new_y < 0 || new_x >= new_length || new_y >= new_height { continue; }
                if mask.get(new_y.wrapping_mul(new_length).wrapping_add(new_x) as usize).copied().unwrap_or(false) {
                    let index = x.wrapping_mul(self.length).wrapping_add(y) as usize;
                    if let Some(cell) = self.scope.get_mut(index) { *cell = false; }
                }
            }
        }
    }

    pub(crate) fn cell_active(&self, x: i32, y: i32) -> bool {
        self.scope.get(y.wrapping_mul(self.length).wrapping_add(x) as usize).copied().unwrap_or(false)
    }

    pub(crate) fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, self.skill_id() as i32, self.attack.skill_level,
            self.master().master_type, self.master().master_id,
            self.started_at_ms, self.lifetime_ms, now,
        )
    }
}
