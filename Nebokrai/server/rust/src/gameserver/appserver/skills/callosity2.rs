//! Вторая закалка `CCallosity2`.
//!
//! Источник: точная пара `gameserver.exe + GameServer.pdb`, владельцы
//! `callosity2.cpp` и `callositystate2.cpp`. Исполнение разделяет с первой
//! закалкой проверки, необратимый порядок расхода MP/RP, задержку и сообщения,
//! но имеет отдельный идентификатор и время восстановления. Этот владелец
//! создаёт конкретное состояние `0x7d`; взаимное замещение выполняет общий
//! семейный путь через единственный канонический slot.

use super::callositystate2::CallosityState2;

pub(crate) const CALLOSITY_2_SKILL_ID: u32 = 0x7d;

pub(crate) const fn create_callosity_2_state(
    blast_factor: u16,
    time_to_keep: i32,
) -> CallosityState2 {
    CallosityState2::new(blast_factor, time_to_keep)
}
