//! Тонкий путь к стационарной форме громового удара `CThunderBlowPhalanx`
//! (`0x13F`) в Zone. Источник: gameserver.exe/GameServer.pdb,
//! appserver/skills/thunderblowphalanx.cpp. Данные формы, тики, клиентский
//! снимок и живая формула (2 RNG, крит в расширенной точности x87) перенесены
//! буквально в `nebokrai_zone::skills::thunderblow` (пара навыка и области
//! слита в один файл, основание и статусы см. там). Здесь — реэкспорт прежних
//! типов и делегация формулы с прежней сигнатурой; владельцы рантайм-флоу
//! области и потребители не меняются.

use crate::gameserver::appserver::states::attackpower::AttackInformation;
use crate::gameserver::appserver::player::PlayerCombatProperties;
use crate::gameserver::gameserver::game::CGame;

pub(crate) use nebokrai_zone::skills::thunderblow::{CThunderBlowPhalanx, ThunderBlowPhalanxTick};

pub(crate) fn calculate_owned_thunder_blow_attack(
    game: &mut CGame,
    phalanx: &CThunderBlowPhalanx,
    target_level: u8,
) -> Option<(AttackInformation, PlayerCombatProperties, u8, u8)> {
    nebokrai_zone::skills::thunderblow::calculate_owned_thunder_blow_attack(game, phalanx, target_level)
}
