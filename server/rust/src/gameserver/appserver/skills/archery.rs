//! Идентификатор и монстровый адаптер базовой стрельбы CArchery.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/archery.cpp.
//! Подтверждённое совместимое исполнение находится в baseprojectilecast,
//! проверки — в baseprojectilecheck, отдельный урон — в archeryphalanx.
//! Постоянное время полёта принадлежит зарегистрированному экземпляру.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, ServerRegionOwner};

pub(crate) const ARCHERY_SKILL_ID: u32 = 2;

pub(crate) fn execute_owned_monster_archery<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    super::baseprojectilecast::execute_owned_monster_base_projectile::<ARCHERY_SKILL_ID, Runtime>(
        game, owner, monster_id, target, skill_level, runtime)
}
