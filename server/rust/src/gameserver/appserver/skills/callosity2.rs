//! Вторая закалка CCallosity2 (0x7D).
//! Источник: gameserver.exe/GameServer.pdb, appserver/skills/callosity2.cpp.
//! Исполнение и visual совпадают с первой закалкой; собственный ID сохраняет
//! отдельные свойства, восстановление и вид состояния. Общий caller находится
//! в callosity, взаимное замещение и канонические записи — в callositystate.

pub(crate) use nebokrai_zone::effects::CALLOSITY_2_SKILL_ID;
