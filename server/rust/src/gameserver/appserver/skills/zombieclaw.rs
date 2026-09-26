//! CZombieClaw (0x1A2) использует общий пошаговый снаряд.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/zombieclaw.cpp.
//! Общий зарегистрированный PathProjectile — Zone `skills/pathprojectile.rs`
//! (кластер B полосы Monster 0x19x); отличие ZombieClaw — ID и безоружный
//! профиль формулы прямой стихии. Нулевая MP-цена этого Check всё равно
//! запрещает движение; первый AI затем выполняет обычное списание MP и
//! публикацию состояния. ID владельца живёт в Zone.

use super::energybolt::execute_owned_path_projectile;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, ServerRegionOwner};

pub(crate) use nebokrai_zone::skills::pathprojectile::ZOMBIE_CLAW_SKILL_ID;

pub(crate) fn execute_owned_monster_zombie_claw<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target: ShapeIdentity,
    skill_level: u16,
    runtime: &mut Runtime,
) -> bool {
    execute_owned_path_projectile::<ZOMBIE_CLAW_SKILL_ID, Runtime>(
        game, owner, monster_id, target, skill_level, runtime,
    )
}
