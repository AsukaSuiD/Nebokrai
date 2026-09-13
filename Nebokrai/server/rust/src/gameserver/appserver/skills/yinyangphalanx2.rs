//! Маска второй области инь-ян.
//! Источник: gameserver.exe/GameServer.pdb, yinyangphalanx2.cpp.
//! Все уровни используют 1×1. Независимый экземпляр создаётся фабрикой
//! yinyangphalanx; маска, часы и обход принадлежат maskedelementphalanx.

pub(crate) const YIN_YANG_2_SCOPE: (i32, i32, &[bool]) = (1, 1, &[true]);
