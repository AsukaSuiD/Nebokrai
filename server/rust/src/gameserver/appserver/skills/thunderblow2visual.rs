//! Тонкий путь к визуальному ресурсу `CThunderBlow2` в Zone.
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/thunderblow2.cpp.
//! Wire-visual `0x000BFE01` (modes 0/1/3 с разрешением S у mode 1 и
//! personal-таблица ошибок) перенесён буквально в
//! `nebokrai_zone::skills::thunderblow2` — исходный `thunderblow2.cpp`
//! владеет обеими частями, поэтому файл слит с навыком (основание и статусы
//! см. там). Здесь — делегация с прежней сигнатурой; швы доставки реализованы
//! над `CGame` в `skills/thunderblow2.rs`; потребители не меняются.

use crate::gameserver::appserver::moveshape::MoveShapeSkill;
use crate::gameserver::gameserver::game::CGame;

pub(crate) fn publish_thunder_blow_2_visual(game: &CGame, skill: &MoveShapeSkill, mode: u32) {
    nebokrai_zone::skills::thunderblow2::publish_thunder_blow_2_visual(game, skill, mode)
}
