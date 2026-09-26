//! Призыв снежной бури CSnowStorm.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/snowstorm.cpp.
//! Тело Summon и живая форма области перенесены буквально в
//! `nebokrai_zone::skills::snowstorm` порцией T5 «zonalcast-хаб» (порядок
//! запросов, диагностика параметров и статусы — там). Здесь — делегация с
//! прежней сигнатурой; потребитель (hub zonalcast) не меняется.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::appserver::states::skill::RegisteredSkill;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) use nebokrai_zone::skills::SNOW_STORM_SKILL_ID;

pub(super) fn summon_snow_storm<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, instance: RegisteredSkill, source: (i32, ShapeIdentity),
    destination: (i32, i32), runtime: &mut Runtime,
) {
    nebokrai_zone::skills::summon_snow_storm(
        game, instance, source, destination, runtime,
        &mut |runtime: &mut Runtime| runtime.now_milliseconds(),
    );
}
