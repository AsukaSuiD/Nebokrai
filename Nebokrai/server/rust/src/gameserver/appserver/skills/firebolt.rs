//! Огненная стрела CFireBolt (0x132).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/firebolt.cpp.
//! Игрок и монстр используют общий baseprojectilecast: Begin/Check,
//! расход MP, CAN, поворот, задержка, выпуск и End сохраняют порядок оригинала.
//! Check/visual принадлежат baseprojectilecheck, урон — отдельному phalanx.
//! Первый слот SoulCollect снимается через живой End перед конструктором.
//! Время полёта постоянно для экземпляра; S обязателен в AI, но не в Check.
//! End не обнуляет время полёта. Прицельная форма завершает удар без BF504.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime, ServerRegionOwner};

pub(crate) const FIRE_BOLT_SKILL_ID: u32 = 0x132;

pub(crate) fn execute_owned_monster_fire_bolt<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, owner: &mut Option<ServerRegionOwner>, monster_id: i32,
    target: ShapeIdentity, skill_level: u16, runtime: &mut Runtime,
) -> bool {
    super::baseprojectilecast::execute_owned_monster_base_projectile::<FIRE_BOLT_SKILL_ID, Runtime>(
        game, owner, monster_id, target, skill_level, runtime)
}
