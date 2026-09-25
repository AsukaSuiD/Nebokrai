//! Движущийся площадной снаряд огненного шара FireBall.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/fireballphalanx.cpp.
//! Снимок конструктора, часы и клиентский префикс разделяют общий полёт,
//! элементный контакт с усилителем душами и движение пути —
//! `nebokrai_zone::skills::projectile` (основание и статусы см. там);
//! обёртка компонует полёт и контакт, сохраняя прежние имена и интерфейс
//! для владельцев game/. Души приходят снимком из cast-а, как у FireBolt.
//! Литерал навыка 0x13D кладётся в `+0xB8`, клиентский encoder пишет
//! skill/level/master/remaining, не endpoint. Срок жизни и дедлайн
//! очередной клетки читают разные часы.

use super::baseprojectilephalanx::BaseProjectileFlight;
use super::elementprojectileattack::{ElementProjectileAttack, SoulProjectileAmplification};
use super::fireball::FIRE_BALL_SKILL_ID;
use nebokrai_zone::skills::FireBallPath;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::CShape;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CFireBallPhalanx {
    flight: BaseProjectileFlight,
    attack: ElementProjectileAttack,
    movement: FireBallPath,
}

impl CFireBallPhalanx {
    #[allow(clippy::too_many_arguments, reason = "снимок конструктора FireBallPhalanx")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32,
        element_modifier: i32, path: Vec<(i32, i32)>, speed_ms: u32,
        soul_count: i32, soul_variable: u32,
    ) -> Self {
        Self {
            flight: BaseProjectileFlight::new_untargeted(id, started_at_ms, lifetime_ms),
            attack: ElementProjectileAttack::new(
                master, FIRE_BALL_SKILL_ID, skill_level, minimum_attack, maximum_attack,
                element_modifier,
                Some(SoulProjectileAmplification::new(soul_count, soul_variable as i32)),
            ),
            movement: FireBallPath::new(path, speed_ms),
        }
    }

    pub(crate) const fn shape(&self) -> &CShape { self.flight.shape() }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { self.flight.shape_mut() }
    pub(crate) const fn master(&self) -> MasterInfo { self.attack.master() }
    pub(crate) const fn attack_snapshot(&self) -> ElementProjectileAttack { self.attack }
    pub(crate) fn path_is_empty(&self) -> bool { self.movement.is_empty() }

    pub(crate) fn expired_at(&self, now_ms: u32) -> bool { self.flight.expired_at(now_ms) }

    pub(crate) fn cell_due_at(&self, now_ms: u32) -> bool {
        self.movement.cell_due_at(self.flight.started_at_ms(), now_ms)
    }

    pub(crate) fn current_cell(&self) -> Option<(i32, i32)> { self.movement.current_cell() }
    pub(crate) fn set_cell_destination(&mut self, cell: (i32, i32)) {
        self.movement.set_destination(cell);
    }
    pub(crate) fn advance(&mut self) { self.movement.advance(); }
    pub(crate) fn pending_force_move(&self) -> Option<(i32, i32, u32)> {
        self.movement.pending_force_move()
    }
    pub(crate) fn mark_force_moved(&mut self) { self.movement.mark_force_moved(); }

    pub(crate) fn scope_cells(center_x: i32, center_y: i32) -> impl Iterator<Item = (i32, i32)> {
        FireBallPath::scope_cells(center_x, center_y)
    }

    pub(crate) fn encode_client_snapshot(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        self.flight.encode_client_snapshot(
            FIRE_BALL_SKILL_ID, self.attack.skill_level(), self.attack.master(),
            now_milliseconds,
        )
    }
}

// Серверный `DecordFromByteArray` FireBall (pub `1:001fafa0`, RVA `0x1FBFA0`)
// повторяет общую форму прицельных фаланг с уровнем в слоте профиля `+0xD4`;
// общий decoder перенесён в `nebokrai_zone::skills::BaseProjectileFlight::
// decode_server_snapshot` (основание см. там), достижимого caller-а у
// оригинала нет.
