//! CSnakeBolt (0x1A5) использует общий пошаговый снаряд.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/snakebolt.cpp.
//! Общий зарегистрированный PathProjectile сохраняет его Begin/Check/AI/End;
//! прямой стихийный урон учитывает оружейный множитель.
//! Нулевая MP-цена этого Check всё равно запрещает движение; первый AI затем
//! выполняет обычное списание MP и публикацию состояния.

use super::energybolt::execute_owned_path_projectile;
use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, ServerRegionOwner};

pub(crate) const SNAKE_BOLT_SKILL_ID: u32 = 0x1a5;

pub(crate) fn execute_owned_monster_snake_bolt<Runtime: GameMainLoopRuntime>(
    game: &mut CGame,
    owner: &mut Option<ServerRegionOwner>,
    monster_id: i32,
    target: ShapeIdentity,
    skill_level: u16,
    runtime: &mut Runtime,
) -> bool {
    execute_owned_path_projectile::<SNAKE_BOLT_SKILL_ID, Runtime>(
        game, owner, monster_id, target, skill_level, runtime,
    )
}
