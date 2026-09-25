//! Огненный шар CFireBall (0x13d).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/fireball.cpp.
//! Игрок и монстр используют общий baseprojectilecast: Begin/Check,
//! расход MP, CAN, поворот, задержка, выпуск и End сохраняют порядок оригинала.
//! Check/visual принадлежат baseprojectilecheck, урон — отдельному phalanx.
//! Первый слот SoulCollect снимается через живой End перед конструктором.
//! AI допускает NULL S и не проверяет смерть; путь выпуска использует forced MAX.
//! Move0 находится в Check игрока; форма движется и атакует область до полного End.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, ServerRegionOwner};

pub(crate) use nebokrai_zone::skills::FIRE_BALL_SKILL_ID;

pub(crate) fn execute_owned_monster_fire_ball<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    super::baseprojectilecast::execute_owned_monster_base_projectile::<FIRE_BALL_SKILL_ID, Runtime>(
        game, owner, monster_id, target, skill_level, runtime)
}
