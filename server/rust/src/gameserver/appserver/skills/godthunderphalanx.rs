//! Адаптер CShape и региона для областей GodThunder/GodThunder2.
//! Исходные владельцы: appserver/skills/godthunderphalanx{,2}.cpp/.h.
//! Окна, часы, маска и поля клиентского снимка — в zone/skills/godthunder.rs.
//! Для server decode VA 0x005F5D90 подтверждённого вызывающего пути нет.

use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::summonshape::SUMMON_SHAPE_TYPE;
use nebokrai_shared::values::CGuid;
use nebokrai_zone::skills::{ElementPhalanxAttack, GodThunderParametersError,
    GodThunderPhalanx, GodThunderSummonParameters};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CGodThunderPhalanx {
    shape: CShape,
    area: GodThunderPhalanx,
}

impl CGodThunderPhalanx {
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32,
        parameters: GodThunderSummonParameters,
    ) -> Result<Self, GodThunderParametersError> {
        let area = GodThunderPhalanx::new(
            ElementPhalanxAttack {
                master, skill_id: parameters.skill_id, skill_level: parameters.skill_level,
                minimum: parameters.minimum_attack, maximum: parameters.maximum_attack,
                element: parameters.element_attack, critical_chance: parameters.critical_chance,
            },
            started_at_ms, parameters.lifetime_ms, parameters.frequency_ms,
            parameters.target_count,
        )?;
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity {
            object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID,
        });
        Ok(Self { shape, area })
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.area.attack_snapshot().master }
    pub(crate) const fn attack_snapshot(&self) -> ElementPhalanxAttack {
        self.area.attack_snapshot()
    }
    pub(crate) const fn has_war_soul_pass(&self) -> bool { self.area.has_war_soul_pass() }
    pub(crate) const fn scope_area(&self) -> u32 { self.area.scope_area() }

    pub(crate) fn initialize(&mut self, random: &mut dyn FnMut(i32) -> i32) {
        let center = (self.shape.get_tile_x().unwrap_or(i32::MIN),
            self.shape.get_tile_y().unwrap_or(i32::MIN));
        self.area.initialize(center, random);
    }
    pub(crate) const fn expired_at(&self, now: u32) -> bool { self.area.expired_at(now) }
    pub(crate) const fn attack_due_at(&self, now: u32) -> bool { self.area.attack_due_at(now) }
    pub(crate) fn mark_attack_at(&mut self, now: u32) { self.area.mark_attack_at(now); }
    pub(crate) fn advance_attack_window(&mut self) { self.area.advance_attack_window(); }
    pub(crate) fn current_cell(&self, index: u32) -> Option<(i32, i32)> {
        self.area.current_cell(index)
    }

    pub(crate) fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        let mut payload = Vec::new();
        self.area.write_client_snapshot_fields(&mut payload, now);
        self.shape.add_to_byte_array(&mut payload, true).then_some(payload)
    }
}
