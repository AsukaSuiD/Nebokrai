//! Неподвижные масочные области огненной стены и инь-ян.
//! Источник: gameserver.exe/GameServer.pdb, firewallphalanx.cpp и
//! yinyangphalanx{,2}.cpp. Маска и часы принадлежат zone/skills/masked_area.rs,
//! снимок атаки и CShape связывает этот адаптер; экземпляры живут в регионе.
//! Стена повторяет окно по трём чтениям часов, инь-ян поражает при истечении
//! срока и завершается после обхода. Пустая маска сама по себе не означает End.
//! Replace транспонирует координаты записи, AI читает обычный X/Y.
//! У стены отклонённая допуском цель тоже входит в дедупликацию окна.

use super::elementphalanxattack::ElementPhalanxAttack;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use nebokrai_shared::values::CGuid;
use nebokrai_zone::skills::MaskedArea;
pub(crate) use nebokrai_zone::skills::MaskedAreaPulse;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MaskedElementPhalanx {
    shape: CShape,
    attack: ElementPhalanxAttack,
    scope: MaskedArea,
}

impl MaskedElementPhalanx {
    pub(super) fn new(
        id: i32, attack: ElementPhalanxAttack, started_at_ms: u32, lifetime_ms: u32,
        pulse: MaskedAreaPulse, mask: (i32, i32, &[bool]),
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self { shape, attack,
            scope: MaskedArea::new(started_at_ms, lifetime_ms, pulse, mask) }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.master }
    pub(crate) const fn skill_id(&self) -> u32 { self.attack.skill_id }
    pub(crate) const fn attack_snapshot(&self) -> ElementPhalanxAttack { self.attack }
    pub(crate) const fn dimensions(&self) -> (i32, i32) { self.scope.dimensions() }
    pub(crate) const fn is_periodic(&self) -> bool { self.scope.is_periodic() }
    pub(crate) const fn expired_at(&self, now: u32) -> bool {
        self.scope.expired_at(now)
    }
    pub(crate) const fn attack_due_at(&self, now: u32) -> bool {
        self.scope.attack_due_at(now)
    }
    pub(crate) fn mark_attack_at(&mut self, now: u32) {
        self.scope.mark_attack_at(now);
    }

    pub(crate) fn origin(&self) -> (i32, i32) {
        let x = self.shape.get_tile_x().unwrap_or(i32::MIN);
        let y = self.shape.get_tile_y().unwrap_or(i32::MIN);
        self.scope.origin((x, y))
    }

    pub(crate) fn replace_affect_region(&mut self, level: i32, tile_x: i32, tile_y: i32) {
        let incoming = if self.is_periodic() {
            nebokrai_zone::skills::fire_wall_scope(level)
        } else { super::yinyangphalanx::scope_for_skill(self.skill_id()) };
        let center = (self.shape.get_tile_x().unwrap_or(i32::MIN),
            self.shape.get_tile_y().unwrap_or(i32::MIN));
        self.scope.replace_affect_region(center, (tile_x, tile_y), incoming);
    }

    pub(crate) fn cell_active(&self, x: i32, y: i32) -> bool {
        self.scope.cell_active(x, y)
    }

    pub(crate) fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, self.skill_id() as i32, self.attack.skill_level,
            self.master().master_type, self.master().master_id,
            self.scope.started_at_ms(), self.scope.lifetime_ms(), now,
        )
    }
}
