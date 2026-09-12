//! Обратный рубящий удар CInverseChopped (0x8A).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/inversechopped.cpp.
//! В frontcellswordcast требует живого игрока при каждом AI, допускает MP0
//! в Check и не повторяет оружие после списания MP. Категория 2, отказ GS0292.
//! frontcellsword обходит все допустимые CMoveShape лицевой клетки, исключая
//! U; энергия усиливает первый Calculate и снимается даже без попадания.
//! Успех и отказ различают End(1)/End(0), общий End сбрасывает фазу и движение.

pub(crate) const INVERSE_CHOPPED_SKILL_ID: u32 = 0x8a;
