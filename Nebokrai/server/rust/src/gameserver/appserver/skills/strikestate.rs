//! Сохранённая блокировка StrikeState (0xDD).
//! Источник: gameserver.exe + GameServer.pdb, appserver/skills/strikestate.cpp.
//!
//! Сохранённое состояние проходит Load → Begin(NULL, holder) → AI/End → Save.
//! Сам CStrike создаёт Rush2, поэтому отдельного primary-caller для 0xDD нет.
//! Нулевые поля конструктора, кодек ID/остаток без копирования состояния,
//! строгий wrapping deadline, visual loop1, блокировки и End общие с Blind. Load читает
//! часы после ID перед remaining, Begin сохраняет этот timestamp, Save читает
//! живой остаток после ID. OnAction пуст; попадание не снимает состояние.
//!
//! Объектный Begin проверяет S до base и сохраняет параметр S через visual.

pub(crate) const STRIKE_STATE_ID: u32 = 0xdd;
pub(crate) const STRIKE_STATE_BYTES: usize = super::blindstate::BLIND_STATE_BYTES;

pub(crate) type StrikeState = super::blindstate::BlindState<STRIKE_STATE_ID>;
