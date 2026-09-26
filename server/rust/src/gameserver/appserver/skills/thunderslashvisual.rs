//! Тонкий путь visual громового рассечения (0x72) в Zone.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/thunderslash.cpp.
//! Тело `UpdateVisualEffect@CThunderSlashEffect` (VA `0x57A550` по точной
//! паре) перенесено буквально в `nebokrai_zone::skills::thunderslash`
//! (visual слит в файл навыка — один исходный cpp; статусы и швы см. там).
//! Здесь — делегация с прежней сигнатурой; диспетчер visual не меняется.

use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::gameserver::game::CGame;

pub(crate) fn publish_thunder_slash_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    nebokrai_zone::skills::thunderslash::publish_thunder_slash_visual(game, skill, mode);
}
