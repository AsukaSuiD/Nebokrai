//! Движущийся площадной снаряд FireBall.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/fireballphalanx.cpp.
//! Срок жизни и срок очередной клетки читают разные часы. На допущенной
//! клетке живого региона сначала сохраняется endpoint, затем BLOCK3 допускает
//! область 3×3 X→Y. Обычные цели локально дедуплицируются после Attack;
//! боевые феи проверяются перед фигурами каждой клетки и в список не входят.
//! Результат области означает наличие обычных попыток, даже по мёртвой цели.
//! После области возможен полный End с BF504, затем растёт текущая позиция.
//! ForceMove и его флаг идут в хвосте AI, даже после такого End; флаг ставится
//! после callback. NULL регион не завершает путь и не увеличивает позицию.
//! Формула и снимок душ общие с FireBolt в elementprojectileattack.
//! Vec сохраняет независимый путь; статическая маска заменяет CScope.
//! Клиентский encoder пишет skill/level/master/remaining, не endpoint.

use super::fireball::FIRE_BALL_SKILL_ID;
use super::elementprojectileattack::{ElementProjectileAttack, SoulProjectileAmplification};
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::{CShape, ShapeIdentity};
use crate::gameserver::appserver::summonshape::{SUMMON_SHAPE_TYPE, encode_related_phalanx_snapshot};
use nebokrai_shared::values::CGuid;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CFireBallPhalanx {
    shape: CShape,
    attack: ElementProjectileAttack,
    started_at_ms: u32,
    lifetime_ms: u32,
    path: Vec<(i32, i32)>,
    speed_ms: u32,
    current_position: u32,
    destination: (i32, i32),
    force_moved: bool,
}

impl CFireBallPhalanx {
    #[allow(clippy::too_many_arguments, reason = "снимок конструктора FireBallPhalanx")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32,
        element_modifier: i32, path: Vec<(i32, i32)>, speed_ms: u32,
        soul_count: i32, soul_variable: u32,
    ) -> Self {
        let mut shape = CShape::with_constructor_defaults();
        shape.set_identity(ShapeIdentity { object_type: SUMMON_SHAPE_TYPE, id, ex_id: CGuid::GUID_INVALID });
        Self {
            shape,
            attack: ElementProjectileAttack::new(
                master, FIRE_BALL_SKILL_ID, skill_level, minimum_attack, maximum_attack,
                element_modifier, Some(SoulProjectileAmplification::new(soul_count, soul_variable as i32)),
            ),
            started_at_ms, lifetime_ms, path, speed_ms,
            current_position: 0, destination: (0, 0), force_moved: false,
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { &self.shape }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { &mut self.shape }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.master() }
    pub(crate) const fn attack_snapshot(&self) -> ElementProjectileAttack { self.attack }
    pub(crate) fn path_is_empty(&self) -> bool { self.path.is_empty() }

    pub(crate) fn expired_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.lifetime_ms) < now_ms
    }

    pub(crate) fn cell_due_at(&self, now_ms: u32) -> bool {
        self.started_at_ms.wrapping_add(self.current_position.wrapping_mul(self.speed_ms)) <= now_ms
    }

    pub(crate) fn current_cell(&self) -> Option<(i32, i32)> {
        self.path.get(self.current_position as usize).copied()
    }

    pub(crate) fn set_cell_destination(&mut self, cell: (i32, i32)) { self.destination = cell; }
    pub(crate) fn advance(&mut self) { self.current_position = self.current_position.wrapping_add(1); }

    pub(crate) fn pending_force_move(&self) -> Option<(i32, i32, u32)> {
        if self.force_moved { return None; }
        let &(x, y) = self.path.last()?;
        Some((x, y, (self.path.len() as u32).wrapping_mul(self.speed_ms)))
    }

    pub(crate) fn mark_force_moved(&mut self) { self.force_moved = true; }

    pub(crate) fn scope_cells(center_x: i32, center_y: i32) -> impl Iterator<Item = (i32, i32)> {
        (0..3).flat_map(move |x| (0..3).map(move |y| {
            (center_x.wrapping_add(x - 1), center_y.wrapping_add(y - 1))
        }))
    }

    pub(crate) fn encode_client_snapshot(&self, now_milliseconds: impl FnMut() -> u32) -> Option<Vec<u8>> {
        encode_related_phalanx_snapshot(
            &self.shape, FIRE_BALL_SKILL_ID as i32, self.attack.skill_level(),
            self.attack.master().master_type, self.attack.master().master_id,
            self.started_at_ms, self.lifetime_ms, now_milliseconds,
        )
    }
}

// Неподключённый серверный decoder FireBall/FireBolt начинает отсчёт заново.
// Клиентский encoder не заменяет отсутствующий caller декодирования.
// FUNCTION: CFireBallPhalanx::DecordFromByteArray
// SOURCE: appserver/skills/fireballphalanx.cpp:317
// RVA: 0x001FBFA0
//
// Полный декомпилят сохранён в локальном исследовательском корпусе.
