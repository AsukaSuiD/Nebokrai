//! Движущийся площадной снаряд огненного шара FireBall — шов к Zone.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/fireballphalanx.cpp.
//! Композит (общий полёт, элементный контакт с усилителем душами, движение
//! пути и клиентский снимок) перенесён буквально в
//! `nebokrai_zone::skills::projectile` (основание и статусы см. там); здесь
//! остаётся только адаптер снимка атаки: live-разрешение полей источника и
//! доставка контакта принадлежат `CGame` и выполняются обёрткой
//! `elementprojectileattack`. Души приходят снимком из cast-а, как у FireBolt.

use super::elementprojectileattack::ElementProjectileAttack;
use crate::gameserver::appserver::masterinfo::MasterInfo;
use crate::gameserver::appserver::shape::CShape;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CFireBallPhalanx(nebokrai_zone::skills::CFireBallPhalanx);

impl CFireBallPhalanx {
    #[allow(clippy::too_many_arguments, reason = "снимок конструктора FireBallPhalanx")]
    pub(crate) fn new(
        id: i32, master: MasterInfo, started_at_ms: u32, lifetime_ms: u32,
        skill_level: i32, minimum_attack: i32, maximum_attack: i32,
        element_modifier: i32, path: Vec<(i32, i32)>, speed_ms: u32,
        soul_count: i32, soul_variable: u32,
    ) -> Self {
        Self(nebokrai_zone::skills::CFireBallPhalanx::new(
            id, master, started_at_ms, lifetime_ms, skill_level, minimum_attack, maximum_attack,
            element_modifier, path, speed_ms, soul_count, soul_variable,
        ))
    }

    pub(crate) const fn shape(&self) -> &CShape { self.0.shape() }
    pub(crate) const fn shape_mut(&mut self) -> &mut CShape { self.0.shape_mut() }
    pub(crate) const fn master(&self) -> MasterInfo { self.0.master() }
    pub(crate) const fn attack_snapshot(&self) -> ElementProjectileAttack {
        ElementProjectileAttack::from_rule(self.0.attack_snapshot())
    }
    pub(crate) fn path_is_empty(&self) -> bool { self.0.path_is_empty() }

    pub(crate) fn expired_at(&self, now_ms: u32) -> bool { self.0.expired_at(now_ms) }

    pub(crate) fn cell_due_at(&self, now_ms: u32) -> bool { self.0.cell_due_at(now_ms) }

    pub(crate) fn current_cell(&self) -> Option<(i32, i32)> { self.0.current_cell() }
    pub(crate) fn set_cell_destination(&mut self, cell: (i32, i32)) {
        self.0.set_cell_destination(cell);
    }
    pub(crate) fn advance(&mut self) { self.0.advance(); }
    pub(crate) fn pending_force_move(&self) -> Option<(i32, i32, u32)> {
        self.0.pending_force_move()
    }
    pub(crate) fn mark_force_moved(&mut self) { self.0.mark_force_moved(); }

    pub(crate) fn scope_cells(center_x: i32, center_y: i32) -> impl Iterator<Item = (i32, i32)> {
        nebokrai_zone::skills::CFireBallPhalanx::scope_cells(center_x, center_y)
    }

    pub(crate) fn encode_client_snapshot(
        &self, now_milliseconds: impl FnMut() -> u32,
    ) -> Option<Vec<u8>> {
        self.0.encode_client_snapshot(now_milliseconds)
    }
}

// Серверный `DecordFromByteArray` FireBall (pub `1:001fafa0`, RVA `0x1FBFA0`)
// перенесён в `nebokrai_zone::skills::BaseProjectileFlight::
// decode_server_snapshot` (основание и статусы см. там), достижимого caller-а
// у оригинала нет.
