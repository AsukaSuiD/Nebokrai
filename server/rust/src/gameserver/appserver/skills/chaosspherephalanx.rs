//! Живая фигура движущейся области; путь, часы и клиентский префикс принадлежат Zone.
//! Источник: GameServer/gameserver.exe + GameServer/GameServer.pdb,
//! appserver/skills/chaosspherephalanx.cpp/.h, ctor VA 0x005FEE80,
//! AddToByteArray VA 0x005FECB0, AI VA 0x005FF270.

use nebokrai_zone::skills::{CHAOS_SPHERE_SKILL_ID, ChaosSpherePhalanx, ElementPhalanxAttack};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use nebokrai_shared::values::CGuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CChaosSpherePhalanx {
    shape: CShape,
    area: ChaosSpherePhalanx,
}

impl CChaosSpherePhalanx {
    #[allow(clippy::too_many_arguments, reason = "поля конструктора исходной области")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, frequency_ms: u32, minimum: i32, maximum: i32,
        element: i32, path: Vec<(i32, i32)>, speed_ms: u32, critical_chance: i32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        shape.set_speed((speed_ms as i32) as f32);
        let attack = ElementPhalanxAttack {
            master, skill_id: CHAOS_SPHERE_SKILL_ID, skill_level,
            minimum, maximum, element, critical_chance,
        };
        let area = ChaosSpherePhalanx::new(
            attack, started_at_ms, lifetime_ms, frequency_ms, path, speed_ms,
        );
        Self { shape, area }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.area.master() }
    pub(crate) const fn attack_snapshot(&self) -> ElementPhalanxAttack { self.area.attack_snapshot() }
    pub(crate) const fn expired_at(&self, now: u32) -> bool { self.area.expired_at(now) }
    pub(crate) fn has_path(&self) -> bool { self.area.has_path() }
    pub(crate) fn initial_force_move(&self) -> Option<(i32, i32, u32)> {
        self.area.initial_force_move()
    }
    pub(crate) fn mark_force_moved_at(&mut self, now: u32) { self.area.mark_force_moved_at(now); }
    pub(crate) const fn movement_due_at(&self, now: u32) -> bool { self.area.movement_due_at(now) }
    pub(crate) fn advance_at(&mut self, now: u32) { self.area.advance_at(now); }
    pub(crate) const fn attack_due_at(&self, now: u32) -> bool { self.area.attack_due_at(now) }
    pub(crate) fn mark_attack_at(&mut self, now: u32) { self.area.mark_attack_at(now); }
    pub(crate) fn attack_origin(&self) -> Option<(i32, i32)> { self.area.attack_origin() }
    pub(crate) fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        self.area.write_client_snapshot_fields(&mut payload, now);
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}
