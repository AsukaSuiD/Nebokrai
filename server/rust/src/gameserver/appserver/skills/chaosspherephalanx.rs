//! Движущаяся область сферы хаоса.
//! Источник: gameserver.exe/GameServer.pdb, chaosspherephalanx.cpp.
//! Первый ForceMove отправляет форму в конец пути; серверный центр затем
//! сдвигается по одной клетке за строгий speed-интервал. Часы и флаг первого
//! движения записываются после ForceMove. Периодический обход квадрата 3×3
//! делает WarSoul перед обычными фигурами; body-dedup начинается заново
//! каждый период и пополняется после Attack. Пустой путь ждёт срока.
//! Числа атаки хранятся единственным Copy-снимком, путь — принадлежащим Vec.
//! Wire содержит skill/level/master type/id/remaining и CShape, без пути.
//! Для server decode 0x005FED20 подтверждённого вызывающего пути нет.

use super::chaossphere::CHAOS_SPHERE_SKILL_ID;
use super::elementphalanxattack::ElementPhalanxAttack;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use nebokrai_shared::values::CGuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CChaosSpherePhalanx {
    shape: CShape,
    attack: ElementPhalanxAttack,
    started_at_ms: u32,
    lifetime_ms: u32,
    frequency_ms: u32,
    path: Vec<(i32, i32)>,
    speed_ms: u32,
    last_attack_ms: u32,
    last_move_ms: u32,
    current_cell: u32,
    force_moved: bool,
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
        Self {
            shape,
            attack: ElementPhalanxAttack {
                master, skill_id: CHAOS_SPHERE_SKILL_ID, skill_level,
                minimum, maximum, element, critical_chance,
            },
            started_at_ms, lifetime_ms, frequency_ms, path, speed_ms,
            last_attack_ms: 0, last_move_ms: 0, current_cell: 0, force_moved: false,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.master }
    pub(crate) const fn attack_snapshot(&self) -> ElementPhalanxAttack { self.attack }

    pub(crate) const fn expired_at(&self, now: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now
    }
    pub(crate) fn has_path(&self) -> bool { !self.path.is_empty() }
    pub(crate) fn initial_force_move(&self) -> Option<(i32, i32, u32)> {
        if self.force_moved { return None; }
        let &(x, y) = self.path.last()?;
        Some((x, y, (self.path.len() as u32).wrapping_mul(self.speed_ms)))
    }
    pub(crate) fn mark_force_moved_at(&mut self, now: u32) {
        self.last_move_ms = now;
        self.force_moved = true;
    }
    pub(crate) const fn movement_due_at(&self, now: u32) -> bool {
        self.speed_ms.wrapping_add(self.last_move_ms) < now
    }
    pub(crate) fn advance_at(&mut self, now: u32) {
        self.last_move_ms = now;
        if self.current_cell < (self.path.len() as u32).wrapping_sub(1) {
            self.current_cell = self.current_cell.wrapping_add(1);
        }
    }
    pub(crate) const fn attack_due_at(&self, now: u32) -> bool {
        self.frequency_ms.wrapping_add(self.last_attack_ms) < now
    }
    pub(crate) fn mark_attack_at(&mut self, now: u32) { self.last_attack_ms = now; }
    pub(crate) fn attack_origin(&self) -> Option<(i32, i32)> {
        self.path.get(self.current_cell as usize)
            .map(|&(x, y)| (x.wrapping_sub(1), y.wrapping_sub(1)))
    }
    pub(crate) fn encode_client_snapshot(&self, now: impl FnMut() -> u32) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, self.attack.skill_id as i32, self.attack.skill_level,
            self.attack.master.master_type, self.attack.master.master_id,
            self.started_at_ms, self.lifetime_ms, now,
        )
    }
}
