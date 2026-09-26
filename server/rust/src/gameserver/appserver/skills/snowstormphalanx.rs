//! Путь к живой форме снежной бури в Zone.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/snowstormphalanx.cpp/.h.
//! Композит `CSnowStormPhalanx` (CShape + область) и применение окна
//! перенесены буквально в `nebokrai_zone::skills::snowstorm` порцией T5
//! «zonalcast-хаб» (основание и статусы — там). Здесь — реэкспорт и
//! делегация с прежней сигнатурой; потребитель (обход `game/snowstorm.rs`)
//! не меняется.

use crate::gameserver::appserver::shape::ShapeIdentity;
use crate::gameserver::gameserver::game::{CGame, GameMainLoopRuntime};

pub(crate) use nebokrai_zone::skills::{CSnowStormPhalanx, SNOW_STORM_SCOPE_AREA, SnowStormAttack};

pub(crate) fn apply_snow_storm_attack<Runtime: GameMainLoopRuntime>(
    game: &mut CGame, snapshot: SnowStormAttack, region: i32,
    target: ShapeIdentity, runtime: &mut Runtime,
) {
    nebokrai_zone::skills::apply_snow_storm_attack(game, snapshot, region, target, runtime);
}
